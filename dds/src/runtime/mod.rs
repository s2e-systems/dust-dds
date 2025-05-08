use std::time::{SystemTime, UNIX_EPOCH};

use executor::{Executor, ExecutorHandle};
use oneshot::{oneshot, OneshotReceiver, OneshotSender};
use timer::{TimerDriver, TimerHandle};

use crate::{
    dcps::runtime::{Clock, DdsRuntime},
    infrastructure::time::Time,
};

pub mod actor;
pub mod executor;
pub mod mpsc;
pub mod oneshot;
pub mod timer;

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
    type OneshotSender<T>
        = OneshotSender<T>
    where
        T: Send;
    type OneshotReceiver<T>
        = OneshotReceiver<T>
    where
        T: Send;

    fn timer(&self) -> Self::TimerHandle {
        self.timer_driver.handle()
    }

    fn clock(&self) -> Self::ClockHandle {
        StdClock
    }

    fn spawner(&self) -> Self::SpawnerHandle {
        self.executor.handle()
    }

    fn oneshot<T>() -> (Self::OneshotSender<T>, Self::OneshotReceiver<T>)
    where
        T: Send,
    {
        oneshot()
    }
}
