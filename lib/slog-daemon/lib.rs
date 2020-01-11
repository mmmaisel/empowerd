extern crate chrono;
extern crate slog;
extern crate signal_hook;

use slog::{Drain, Record, OwnedKVList};

use std::io::Write;
use std::fs::{File, OpenOptions};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

struct RotatingLogger  // TODO: this is inner struct Logger, not Drain
{
    file: File,
    filename: String,
    reopen_log: Arc<AtomicBool>
}

impl RotatingLogger
{
    pub fn new(filename: String) -> RotatingLogger
    {
        let reopen_log = Arc::new(AtomicBool::new(false));
        match signal_hook::flag::register(
            signal_hook::SIGHUP, Arc::clone(&reopen_log))
        {
            Ok(_x) => (),
            // TODO: print out e
            Err(e) => panic!("💩️ Unable to register SIGHUP")
        }

        let file = OpenOptions::new().append(true).create(true).
            open(&filename).expect("💩️ Can't open logfile");
        return RotatingLogger
        {
            file: file,
            filename: filename,
            reopen_log: reopen_log
        };
    }

    pub fn log(&mut self, msg: String) -> Result<(), String>
    {
        if self.reopen_log.swap(false, Ordering::Relaxed)
        {
            // TODO: don't panic
            self.file = OpenOptions::new().append(true).create(true).
                open(&self.filename).expect("💩️ Can't open logfile");
        }
        // TODO: don't panic
        self.file.write(msg.as_bytes()).unwrap();
        return Ok(());
    }
}

pub struct DaemonDrain
{
    logger: Mutex<RotatingLogger>
}

impl DaemonDrain
{
    fn new(logger: RotatingLogger) -> DaemonDrain
    {
        return DaemonDrain
        {
            logger: Mutex::new(logger)
        };
    }
}

impl Drain for DaemonDrain
{
    type Err = String;
    type Ok = ();

    fn log(&self, info: &Record, logger_values: &OwnedKVList)
        -> Result<(), String>
    {
        let mut logger = self.logger.lock().expect("💩️ poisoned mutex");
        // TODO: improve formatting, don't panic
        logger.log(format!("[{}] {} {:?} {:?}\n",
            chrono::Local::now().format("%H:%M:%S %d %b %Y"),
            info.level(), info.msg(), logger_values)).unwrap();
        return Ok(());
    }
}

pub fn logger(filename: String) -> DaemonDrain
{
    return DaemonDrain::new(RotatingLogger::new(filename));
}
