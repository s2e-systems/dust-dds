use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::JoinHandle,
};

use crate::infrastructure::instance::InstanceHandle;

use super::shared_object::DdsRwLock;

struct Timer {
    func: Box<dyn Fn() + Send + Sync>,
    duration: std::time::Duration,
    instant: std::time::Instant,
}

impl Timer {
    fn new(duration: std::time::Duration, func: impl Fn() + 'static + Send + Sync) -> Self {
        Self {
            func: Box::new(func),
            duration,
            instant: std::time::Instant::now(),
        }
    }
    fn is_elapsed(&self) -> bool {
        std::time::Instant::now() - self.instant > self.duration
    }
}
pub struct TimerFactory {
    instance_timers: Arc<DdsRwLock<HashMap<InstanceHandle, Timer>>>,
    thread_handle: Option<JoinHandle<()>>,
    should_stop: Arc<std::sync::atomic::AtomicBool>,
}

impl TimerFactory {
    pub fn new() -> Self {
        let instance_timers = Arc::new(DdsRwLock::new(HashMap::<InstanceHandle, Timer>::new()));
        let t = instance_timers.clone();
        let should_stop = Arc::new(AtomicBool::new(false));
        let should_stop_clone = should_stop.clone();
        let thread_handle = std::thread::spawn(move || loop {
            if should_stop_clone.load(Ordering::Relaxed) {
                break;
            } else {
                std::thread::sleep(std::time::Duration::from_millis(1));
                for (_k, v) in t.read_lock().iter() {
                    if v.is_elapsed() {
                        (v.func)()
                    }
                }
            }
        });
        Self {
            instance_timers,
            thread_handle: Some(thread_handle),
            should_stop,
        }
    }

    pub fn start_timer(
        &self,
        duration: std::time::Duration,
        id: InstanceHandle,
        func: impl Fn() + 'static + Send + Sync,
    ) {
        self.instance_timers
            .write_lock()
            .insert(id, Timer::new(duration, func));
    }

    pub fn cancel_timers(&self) {
        self.instance_timers.write_lock().clear();
    }
}

impl Drop for TimerFactory {
    fn drop(&mut self) {
        self.cancel_timers();
        self.should_stop.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread_handle.take() {
            t.join().ok();
        }
    }
}
