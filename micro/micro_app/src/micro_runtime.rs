extern crate alloc;

use alloc::{boxed::Box, sync::Arc};
use chrono::NaiveDateTime;
use core::pin::Pin;
use defmt::*;
use dust_dds::runtime::{Clock, DdsRuntime, Spawner, TaskHandle, Timer};
use embassy_executor::SendSpawner;
use embassy_stm32::rtc::RtcTimeProvider;

#[embassy_executor::task(pool_size = 24)]
async fn future_function(f: Pin<Box<dyn Future<Output = ()> + Send>>) {
    f.await
}
#[derive(Clone)]
pub struct MicroClock {
    rtc: Arc<RtcTimeProvider>,
}
impl Clock for MicroClock {
    fn now(&self) -> dust_dds::infrastructure::time::Time {
        let then: NaiveDateTime = self.rtc.now().unwrap().into();
        dust_dds::infrastructure::time::Time::new(then.and_utc().timestamp() as i32, 0)
    }
}

#[derive(Clone)]
pub struct MicroTimer;
impl Timer for MicroTimer {
    fn delay(&mut self, duration: core::time::Duration) -> impl Future<Output = ()> + Send {
        let embassy_duration = embassy_time::Duration::from_micros(duration.as_micros() as u64);
        embassy_time::Timer::after(embassy_duration)
    }
}

pub struct MicrotaskHandle {}
impl TaskHandle for MicrotaskHandle {
    fn join(&self) {}
}

#[derive(Clone)]
pub struct MicroSpawner {
    spawner: SendSpawner,
}
impl Spawner for MicroSpawner {
    type TaskHandle = MicrotaskHandle;

    fn spawn(&self, f: impl Future<Output = ()> + Send + 'static) -> Self::TaskHandle {
        let token = unwrap!(future_function(Box::pin(f)));
        self.spawner.spawn(token);
        MicrotaskHandle {}
    }
}

pub struct MicroRuntime {
    pub rtc_time_provider: Arc<RtcTimeProvider>,
    pub spawner: SendSpawner,
}

impl DdsRuntime for MicroRuntime {
    type ClockHandle = MicroClock;
    type TimerHandle = MicroTimer;
    type SpawnerHandle = MicroSpawner;

    fn timer(&self) -> Self::TimerHandle {
        MicroTimer
    }

    fn clock(&self) -> Self::ClockHandle {
        MicroClock {
            rtc: self.rtc_time_provider.clone(),
        }
    }

    fn spawner(&self) -> Self::SpawnerHandle {
        MicroSpawner {
            spawner: self.spawner,
        }
    }
}
