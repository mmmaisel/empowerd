use std::fs::File;
use std::io::{Error, Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Condvar, Mutex};
use std::time;
use std::thread;

extern crate config;
extern crate daemonize;
#[macro_use] extern crate serde;
extern crate signal_hook;
#[macro_use] extern crate slog;
extern crate slog_term;

extern crate slog_daemon;

use daemonize::Daemonize;
use slog::Logger;

// TODO: csvimport later
//mod csvimport;
mod influx;
mod miner;
mod scheduler;
mod settings;

//use csvimport::*;
use influx::*;
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

fn main()
{
    let mut daemonize = false;

    // TODO: config file from args!
    let settings = Settings::load_config("/tmp/test.conf".to_string());
    println!("{:?}", settings);

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
            Ok(_) => info!(logger, "  Daemonized"),
            Err(e) => info!(logger, "💩️ Error {}", e)
        }
    }
    // TODO: oneshot, csv mode
    daemon_main(settings, root_logger.new(o!()));
    info!(logger, "  terminated");
}

fn daemon_main(settings: Settings, logger: Logger)
{
    const DACHS_TASK_ID: u32 = 1;
    const SMA_TASK_ID: u32 = 2;

    // TODO: remove expect everywhere
    // TODO: use logger instead of expect
    let (mut read, write) = UnixStream::pair().expect("💩️");
    signal_hook::pipe::register(signal_hook::SIGINT, write).
        expect("💩️ Unable to register SIGINT");
    let mut dummy = [0];

    let condition_parent = Arc::new((Mutex::new(false), Condvar::new()));
    let condition_child = condition_parent.clone();

    info!(logger, "🚀️ Starting main");
    let child = thread::spawn(move ||
    {
        let mut miner = StromMiner::new(settings.dachs_addr,
            settings.dachs_pw, settings.sma_addr, settings.sma_pw);

        let mut scheduler = Scheduler::new();
        scheduler.add_task(DACHS_TASK_ID, settings.dachs_poll_interval);
        scheduler.add_task(SMA_TASK_ID, settings.sma_poll_interval);
        scheduler.run(condition_child, |id, now|
        {
            match id
            {
                DACHS_TASK_ID =>
                {
                    miner.get_dachs_data();
                    // TODO: do sth with data
                }
                SMA_TASK_ID =>
                {
                    miner.get_sma_data();
                    // TODO: do sth with data
                }
                _ => panic!("unexpected id found")
            }
        });
    });

    read.read_exact(&mut dummy).expect("💩️");
    info!(logger, "🚦️ received {}, stopping main", dummy[0]);
    {
        let mut guarded = condition_parent.0.lock().unwrap();
        *guarded = true;
    }
    condition_parent.1.notify_all();

    info!(logger, "  terminating");
    let thread_result = child.join();
}
