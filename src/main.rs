use std::env;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::panic;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

extern crate config;
extern crate daemonize;
extern crate libc;
#[macro_use] extern crate serde;
extern crate signal_hook;
#[macro_use] extern crate slog;
extern crate slog_term;

extern crate slog_daemon;

use daemonize::Daemonize;
use slog::Logger;

// TODO: csvimport later
//mod csvimport;
//mod influx;
mod miner;
mod models;
mod scheduler;
mod settings;

//use csvimport::*;
//use influx::*;
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
    let mut cfg_path = "/tmp/test.conf".to_string();
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

    let settings = match Settings::load_config(
        "/tmp/test.conf".to_string())
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

    if settings.daemonize
    {
        let daemon = Daemonize::new().
            pid_file("/tmp/test.pid").
            chown_pid_file(true).
            working_directory("/tmp");

        match daemon.start()
        {
            Ok(_) => debug!(logger, "Daemonized"),
            Err(e) => error!(logger, "💩️ Error {}", e)
        }
    }
    // TODO: oneshot, csv mode
    daemon_main(settings, root_logger.new(o!()));
    info!(logger, "terminated");
}

fn daemon_main(settings: Settings, logger: Logger)
{
    const DACHS_TASK_ID: u32 = 1;
    const SMA_TASK_ID: u32 = 2;

    let (mut intpipe_r, mut intpipe_w) = match UnixStream::pair()
    {
        Ok((read, write)) => (read, write),
        Err(e) =>
        {
            error!(logger, "💩️ Open signal pipe failed, {}", e);
            return;
        }
    };
    if let Err(e) = signal_hook::pipe::register(
        signal_hook::SIGINT, intpipe_w.try_clone().unwrap()) // TODO: dont unwrap
    {
        error!(logger, "💩️ Unable to register SIGINT, {}", e);
        return;
    }
    let mut dummy = [0];

    let condition_parent = Arc::new((Mutex::new(false), Condvar::new()));
    let condition_child = condition_parent.clone();
    let child_logger = logger.new(o!());

    info!(logger, "🚀️ Starting main");
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
                //  after dachs data was written!!!
                println!("panic occurred");
            }
            unsafe { libc::kill(libc::getpid(), signal_hook::SIGINT) };
        }));
        let mut miner = StromMiner::new(
            settings.db_url, settings.db_name,
            settings.dachs_addr, settings.dachs_pw,
            settings.sma_addr, settings.sma_pw);

        let mut scheduler = Scheduler::new();
        scheduler.add_task(DACHS_TASK_ID, settings.dachs_poll_interval);
        scheduler.add_task(SMA_TASK_ID, settings.sma_poll_interval);
        let result = scheduler.run(condition_child, &child_logger, |id, now|
        {
            match id
            {
                DACHS_TASK_ID =>
                {
                    miner.mine_dachs_data(now);
                }
                SMA_TASK_ID =>
                {
                    miner.mine_sma_data();
                }
                _ => panic!("unexpected id found")
            }
        });
        if let Err(e) = result
        {
            error!(child_logger, "💩️ Scheduler failed, {}", e);
        }
        intpipe_w.write(&[55]);
    });

    if let Err(e) = intpipe_r.read_exact(&mut dummy)
    {
        error!(logger, "💩️ could not read signal pipe, {}", e);
    }
    debug!(logger, "received {}, stopping main", dummy[0]);
    {
        let mut guarded = condition_parent.0.lock().unwrap();
        *guarded = true;
    }
    condition_parent.1.notify_all();

    info!(logger, "terminating");
    if let Err(e) = child.join()
    {
        error!(logger, "💩️ Miner thread returned an error, {:?}", e);
    }
}
