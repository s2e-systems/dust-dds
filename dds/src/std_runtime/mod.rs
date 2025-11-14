pub mod executor;
pub mod timer;

use std::time::{SystemTime, UNIX_EPOCH};

use executor::{Executor, ExecutorHandle};
use timer::{TimerDriver, TimerHandle};

use crate::{
    infrastructure::time::Time,
    runtime::{Clock, DdsRuntime},
};

#[derive(Clone)]
pub struct StdClock;

impl Clock for StdClock {
    fn now(&self) -> Time {
        let now_system_time = SystemTime::now();
        let unix_time = now_system_time
            .duration_since(UNIX_EPOCH)
            .expect("Clock time is before Unix epoch start");
        Time::new(unix_time.as_secs() as i32, unix_time.subsec_nanos())
    }
}

pub struct StdRuntime {
    timer_driver: TimerDriver,
    executor: Executor,
}

impl StdRuntime {
    pub fn new(executor: Executor, timer_driver: TimerDriver) -> Self {
        Self {
            executor,
            timer_driver,
        }
    }
}

impl DdsRuntime for StdRuntime {
    type ClockHandle = StdClock;
    type TimerHandle = TimerHandle;
    type SpawnerHandle = ExecutorHandle;

    fn timer(&self) -> Self::TimerHandle {
        self.timer_driver.handle()
    }

    fn clock(&self) -> Self::ClockHandle {
        StdClock
    }

    fn spawner(&self) -> Self::SpawnerHandle {
        self.executor.handle()
    }

    fn block_on<T>(f: impl core::future::Future<Output = T>) -> T {
        executor::block_on(f)
    }
}
