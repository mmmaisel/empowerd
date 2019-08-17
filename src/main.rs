use std::env;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::panic;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

extern crate config;
extern crate daemonize;
extern crate influx_derive;
extern crate libc;
#[macro_use] extern crate serde;
extern crate signal_hook;
#[macro_use] extern crate slog;
extern crate slog_term;

extern crate slog_daemon;

use daemonize::Daemonize;
use slog::Logger;

mod csvimport;
mod miner;
mod models;
mod scheduler;
mod settings;

use csvimport::*;
use miner::*;
use scheduler::*;
use settings::*;

// TODO: ALL TIMES MUST BE UTC FOR INFLUX LOGGING
// TODO: use logger everywhere, sma, dachs

fn setup_logging(settings: &Settings) -> slog::Logger
{
    let daemon_drain = slog_daemon::logger(settings.log_filename.clone());
    let term_decorator = slog_term::PlainSyncDecorator::new(std::io::stdout());
    let term_drain = slog_term::FullFormat::new(term_decorator).build();

    let duplication = slog::Duplicate(daemon_drain, term_drain);
    // TODO: implement level filtering via settings
    //let filter = slog::Filter(duplication);

    return slog::Logger::root(slog::Fuse(duplication), o!());
}

fn load_settings() -> Result<Settings, ()>
{
    let mut cfg_path = "/etc/stromd/stromd.conf".to_string();
    let mut found_cfg: bool = false;
    for arg in env::args()
    {
        if found_cfg
        {
            cfg_path = arg;
            break;
        }

        if arg == "-c"
        {
            found_cfg = true;
        }
    }

    let settings = match Settings::load_config(cfg_path)
    {
        Ok(x) => x,
        Err(e) =>
        {
            println!("💩️ Could not load config, {}", e);
            return Err(());
        }
    };
    if cfg!(debug_assertions)
    {
        println!("{:?}", settings);
    }
    return Ok(settings);
}

fn main()
{
    let settings = match load_settings()
    {
        Ok(x) => x,
        Err(_) => return
    };

    let root_logger = setup_logging(&settings);
    info!(root_logger, "⚡️ Starting stromd");
    let logger = root_logger.new(o!());

    if settings.import != "none"
    {
        csvimport_main(settings, root_logger.new(o!()));
        return;
    }

    if settings.daemonize
    {
        let daemon = Daemonize::new().
            pid_file(&settings.pid_file).
            chown_pid_file(true).
            working_directory(&settings.wrk_dir);

        match daemon.start()
        {
            Ok(_) => debug!(logger, "Daemonized"),
            Err(e) => error!(logger, "💩️ Error {}", e)
        }
    }
    // TODO: oneshot mode
    daemon_main(settings, root_logger.new(o!()));
    info!(logger, "terminated");
}

fn csvimport_main(settings: Settings, logger: Logger)
{
    let miner = match
        StromMiner::new(settings.clone(), logger.new(o!()))
    {
        Ok(x) => x,
        Err(e) =>
        {
            error!(logger, "Could not create miner, error: {}", e);
            return;
        }
    };

    info!(logger, "🔧️ Importing {} data", settings.import);
    let result = if settings.import == "solar"
    {
        import_solar(&miner)
    }
    else if settings.import == "dachs"
    {
        import_dachs(&miner)
    }
    else if settings.import == "meter"
    {
        import_meter(&miner)
    }
    else if settings.import == "water"
    {
        import_water(&miner)
    }
    else if settings.import == "gas"
    {
        import_gas(&miner)
    }
    else
    {
        Err(format!("Invalid import option '{}'", settings.import))
    };

    if let Err(e) = result
    {
        error!(logger, "{}", e);
    }
}

fn daemon_main(settings: Settings, logger: Logger)
{
    const DACHS_TASK_ID: u32 = 1;
    const SMA_TASK_ID: u32 = 2;
    const METER_TASK_ID: u32 = 3;

    let (mut intpipe_r, mut intpipe_w) = match UnixStream::pair()
    {
        Ok((read, write)) => (read, write),
        Err(e) =>
        {
            error!(logger, "💩️ Open signal pipe failed, {}", e);
            return;
        }
    };

    let intpipe_w2 = match intpipe_w.try_clone()
    {
        Ok(pipe) => pipe,
        Err(e) =>
        {
            error!(logger, "💩️ Unable to clone signal pipe, {}", e);
            return;
        }
    };
    if let Err(e) = signal_hook::pipe::register(
        signal_hook::SIGINT, intpipe_w2)
    {
        error!(logger, "💩️ Unable to register SIGINT, {}", e);
        return;
    }
    let mut dummy = [0];

    let condition_parent = Arc::new((Mutex::new(true), Condvar::new()));
    let condition_child = condition_parent.clone();
    let child_logger = logger.new(o!());

    let child = thread::spawn(move ||
    {
        panic::set_hook(Box::new(|panic_info|
        {
            if let Some(s) = panic_info.payload().downcast_ref::<&str>()
            {
                println!("panic occurred: {:?}", s);
            }
            else
            {
                // TODO: panics sometimes on SIGNIT
                //  after dachs data was written???
                println!("panic occurred");
            }
            unsafe { libc::kill(libc::getpid(), signal_hook::SIGINT) };
        }));
        let mut miner = match
            StromMiner::new(settings.clone(), child_logger.new(o!()))
        {
            Ok(x) => x,
            Err(e) =>
            {
                error!(child_logger, "Could not create miner, error: {}", e);
                intpipe_w.write(&[55]).unwrap();
                return;
            }
        };

        let mut scheduler = Scheduler::new();
        scheduler.add_task(DACHS_TASK_ID, settings.dachs_poll_interval);
        scheduler.add_task(SMA_TASK_ID, settings.sma_poll_interval);
        scheduler.add_task(METER_TASK_ID, settings.meter_poll_interval);
        let result = scheduler.run(condition_child, &child_logger,
            |id, now, _interval|
        {
            match id
            {
                DACHS_TASK_ID =>
                {
                    miner.mine_dachs_data(now);
                }
                SMA_TASK_ID =>
                {
                    miner.mine_solar_data(now);
                }
                METER_TASK_ID =>
                {
                    miner.mine_meter_data(now);
                }
                _ =>
                {
                    error!(child_logger, "unexpected id {} found", id);
                    return false;
                }
            }
            return true;
        });
        if let Err(e) = result
        {
            error!(child_logger, "💩️ Scheduler failed, {}", e);
        }
        intpipe_w.write(&[55]).unwrap();
    });

    if let Err(e) = intpipe_r.read_exact(&mut dummy)
    {
        error!(logger, "💩️ could not read signal pipe, {}", e);
    }
    debug!(logger, "received {}, stopping main", dummy[0]);
    {
        let mut running = condition_parent.0.lock().unwrap();
        *running = false;
    }
    condition_parent.1.notify_all();

    info!(logger, "terminating");
    if let Err(e) = child.join()
    {
        error!(logger, "💩️ Miner thread returned an error, {:?}", e);
    }
}
