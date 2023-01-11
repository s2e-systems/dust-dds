use std::sync::{Arc, Condvar, Mutex};

use crate::infrastructure::{
    error::{DdsError, DdsResult},
    time::Duration,
};

#[derive(Clone)]
pub struct DdsCondvar(Arc<(Condvar, Mutex<bool>)>);

impl Default for DdsCondvar {
    fn default() -> Self {
        Self::new()
    }
}

impl DdsCondvar {
    pub fn new() -> Self {
        Self(Arc::new((Condvar::new(), Mutex::new(false))))
    }

    pub fn notify_all(&self) {
        let mut started = self.0 .1.lock().unwrap();
        *started = true;
        self.0 .0.notify_all()
    }

    pub fn wait_timeout(&self, timeout: Duration) -> DdsResult<()> {
        let cvar = &self.0 .0;
        loop {
            let started = self.0 .1.lock().unwrap();
            let dur = std::time::Duration::new(timeout.sec() as u64, timeout.nanosec());
            let (mut started, result) = cvar.wait_timeout(started, dur).unwrap();
            if result.timed_out() {
                return Err(DdsError::Timeout);
            } else if *started {
                // Put the value back to false for the next round
                *started = false;
                return Ok(());
            }
        }
    }
}
