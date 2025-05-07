use timer::{TimerDriver, TimerHandle};

use crate::implementation::domain_participant_backend::domain_participant_actor::DdsRuntime;

pub mod actor;
pub mod executor;
pub mod mpsc;
pub mod oneshot;
pub mod timer;

pub struct StdRuntime {
    timer_driver: TimerDriver,
}

impl StdRuntime {
    pub fn new(timer_driver: TimerDriver) -> Self {
        Self { timer_driver }
    }
}

impl DdsRuntime for StdRuntime {
    type TimerHandle = TimerHandle;

    fn timer(&mut self) -> Self::TimerHandle {
        self.timer_driver.handle()
    }
}
