use std::cmp;
use std::sync::{Arc, Condvar, Mutex};
use std::time;

use slog::Logger;

struct ScheduledTask
{
    pending: bool,
    id: u32,
    next_wake: i64,
    interval: i64
}

impl ScheduledTask
{
    fn new(id: u32, interval: i64) -> ScheduledTask
    {
        return ScheduledTask
        {
            pending: false,
            id: id,
            next_wake: 0,
            interval: interval
        };
    }
}

pub struct Scheduler
{
    now: i64,
    next_wake: i64,
    tasks: Vec<ScheduledTask>
}

impl Scheduler
{
    pub fn new() -> Scheduler
    {
        return Scheduler
        {
            now: 0,
            next_wake: 0,
            tasks: Vec::new()
        };
    }

    pub fn add_task(&mut self, id: u32, interval: i64)
    {
        self.tasks.push(ScheduledTask::new(id, interval));
    }

    fn check_pending_tasks(&mut self, shall_exec: bool)
    {
        self.next_wake = std::i64::MAX;
        self.now = time::SystemTime::now().
            duration_since(time::UNIX_EPOCH).
            expect("💩️ Time outside of epoch").as_secs() as i64;

        for task in self.tasks.iter_mut()
        {
            if task.next_wake <= self.now
            {
                task.next_wake =
                    ((self.now / task.interval) + 1) * task.interval;
                task.pending = shall_exec;
            }
            self.next_wake = cmp::min(task.next_wake, self.next_wake);
        }
    }

    pub fn run<F>(&mut self, condition: Arc<(Mutex<bool>, Condvar)>,
        logger: &Logger, mut callback: F)
        -> Result<(), String>
        where F: FnMut(u32, i64, i64) -> ()
    {
        debug!(logger, "starting scheduler");
        let &(ref mutex, ref cvar) = &*condition;
        let mut running = mutex.lock().unwrap();

        self.check_pending_tasks(false);
        while !*running
        {
            self.check_pending_tasks(true);
            for task in self.tasks.iter_mut()
            {
                if task.pending
                {
                    // TODO: add error handling for tasks?!
                    callback(task.id, self.now, task.interval);
                    task.pending = false;
                }
            }

            self.now = time::SystemTime::now().
                duration_since(time::UNIX_EPOCH).
                expect("💩️ Time outside of epoch").as_secs() as i64;
            let delay = (self.next_wake - self.now) as u64;
            debug!(logger, "Sleep {} sec", delay);
            let result = cvar.wait_timeout(running,
                time::Duration::new(delay, 0)).expect("💩️ poisened mutex");
            running = result.0;
        }
        debug!(logger, "stopping scheduler");
        return Ok(());
    }
}
