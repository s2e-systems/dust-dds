use std::time::{SystemTime, UNIX_EPOCH};

use timer::{TimerDriver, TimerHandle};

use crate::{
    dcps::clock::Clock,
    implementation::domain_participant_backend::domain_participant_actor::DdsRuntime,
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
}

impl StdRuntime {
    pub fn new(timer_driver: TimerDriver) -> Self {
        Self { timer_driver }
    }
}

impl DdsRuntime for StdRuntime {
    type ClockHandle = StdClock;
    type TimerHandle = TimerHandle;

    fn timer(&mut self) -> Self::TimerHandle {
        self.timer_driver.handle()
    }

    fn clock(&mut self) -> Self::ClockHandle {
        StdClock
    }
}
