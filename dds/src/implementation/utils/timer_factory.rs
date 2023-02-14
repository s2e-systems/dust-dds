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

    fn reset(&mut self) {
        self.instant = std::time::Instant::now();
    }
}

pub struct TimerProvider {
    instance_timers: HashMap<InstanceHandle, Timer>,
}

impl TimerProvider {
    fn new() -> Self {
        Self {
            instance_timers: HashMap::new(),
        }
    }

    pub fn start_timer(
        &mut self,
        duration: std::time::Duration,
        id: InstanceHandle,
        func: impl Fn() + 'static + Send + Sync,
    ) {
        self.instance_timers.insert(id, Timer::new(duration, func));
    }

    pub fn cancel_timers(&mut self) {
        self.instance_timers.clear();
    }
}

pub struct TimerFactory {
    timer_provider_list: Arc<DdsRwLock<Vec<Arc<DdsRwLock<TimerProvider>>>>>,
    thread_handle: Option<JoinHandle<()>>,
    should_stop: Arc<std::sync::atomic::AtomicBool>,
}

impl TimerFactory {
    pub fn new() -> Self {
        let timer_provider_list =
            Arc::new(DdsRwLock::new(Vec::<Arc<DdsRwLock<TimerProvider>>>::new()));
        let should_stop = Arc::new(AtomicBool::new(false));

        let timer_provider_list_clone = timer_provider_list.clone();
        let should_stop_clone = should_stop.clone();
        let thread_handle = std::thread::spawn(move || loop {
            if should_stop_clone.load(Ordering::Relaxed) {
                break;
            } else {
                std::thread::sleep(std::time::Duration::from_millis(1));
                for timer_provider in timer_provider_list_clone.read_lock().iter() {
                    for (_k, v) in &mut timer_provider.write_lock().instance_timers {
                        if v.is_elapsed() {
                            v.reset();
                            (v.func)()
                        }
                    }
                }
            }
        });
        Self {
            timer_provider_list,
            thread_handle: Some(thread_handle),
            should_stop,
        }
    }

    pub fn create_timer_provider(&self) -> Arc<DdsRwLock<TimerProvider>> {
        let timer_provider = Arc::new(DdsRwLock::new(TimerProvider::new()));

        self.timer_provider_list
            .write_lock()
            .push(timer_provider.clone());

        timer_provider
    }

    pub fn delete_timer_provider(&mut self, timer_provider: &Arc<DdsRwLock<TimerProvider>>) {
        self.timer_provider_list
            .write_lock()
            .retain(|x| !Arc::ptr_eq(x, timer_provider));
    }
}

impl Drop for TimerFactory {
    fn drop(&mut self) {
        self.timer_provider_list.write_lock().clear();
        self.should_stop.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread_handle.take() {
            t.join().ok();
        }
    }
}

#[cfg(test)]
mod tests {
    use mockall::mock;

    use super::*;

    mock! {
        pub Task {
            fn run(&self);
        }
    }

    #[test]
    fn task_executed_right_number_of_times() {
        let timer_factory = TimerFactory::new();
        let tp1 = timer_factory.create_timer_provider();

        let mut mock_task1 = MockTask::new();
        mock_task1.expect_run().times(2).return_const(());

        tp1.write_lock().start_timer(
            std::time::Duration::from_secs(1),
            InstanceHandle::new([1u8; 16]),
            move || mock_task1.run(),
        );

        let mut mock_task2 = MockTask::new();
        mock_task2.expect_run().times(4).return_const(());
        tp1.write_lock().start_timer(
            std::time::Duration::from_millis(600),
            InstanceHandle::new([2u8; 16]),
            move || mock_task2.run(),
        );

        std::thread::sleep(std::time::Duration::from_millis(2500));
    }

    #[test]
    fn rewritten_timer_gets_updated() {
        let timer_factory = TimerFactory::new();
        let tp1 = timer_factory.create_timer_provider();

        let mut mock_task1 = MockTask::new();
        mock_task1.expect_run().times(0).return_const(());

        tp1.write_lock().start_timer(
            std::time::Duration::from_secs(1),
            InstanceHandle::new([1u8; 16]),
            move || mock_task1.run(),
        );

        std::thread::sleep(std::time::Duration::from_millis(800));

        let mut mock_task1 = MockTask::new();
        mock_task1.expect_run().times(0).return_const(());
        tp1.write_lock().start_timer(
            std::time::Duration::from_secs(1),
            InstanceHandle::new([1u8; 16]),
            move || mock_task1.run(),
        );

        std::thread::sleep(std::time::Duration::from_millis(800));
    }

    #[test]
    fn two_timer_providers() {
        let timer_factory = TimerFactory::new();
        let tp1 = timer_factory.create_timer_provider();
        let tp2 = timer_factory.create_timer_provider();

        let mut mock_task1 = MockTask::new();
        mock_task1.expect_run().times(2).return_const(());

        tp1.write_lock().start_timer(
            std::time::Duration::from_secs(1),
            InstanceHandle::new([1u8; 16]),
            move || mock_task1.run(),
        );

        let mut mock_task2 = MockTask::new();
        mock_task2.expect_run().times(4).return_const(());
        tp2.write_lock().start_timer(
            std::time::Duration::from_millis(600),
            InstanceHandle::new([1u8; 16]),
            move || mock_task2.run(),
        );

        std::thread::sleep(std::time::Duration::from_millis(2500));
    }

    #[test]
    fn deleted_timer_provider() {
        let mut timer_factory = TimerFactory::new();
        let tp1 = timer_factory.create_timer_provider();
        let tp2 = timer_factory.create_timer_provider();

        let mut mock_task1 = MockTask::new();
        mock_task1.expect_run().times(2).return_const(());

        tp1.write_lock().start_timer(
            std::time::Duration::from_secs(1),
            InstanceHandle::new([1u8; 16]),
            move || mock_task1.run(),
        );

        let mut mock_task2 = MockTask::new();
        mock_task2.expect_run().times(0).return_const(());
        tp2.write_lock().start_timer(
            std::time::Duration::from_millis(600),
            InstanceHandle::new([1u8; 16]),
            move || mock_task2.run(),
        );

        timer_factory.delete_timer_provider(&tp2);

        std::thread::sleep(std::time::Duration::from_millis(2500));
    }

    #[test]
    fn dropped_timer_provider() {
        let mut mock_task1 = MockTask::new();
        mock_task1.expect_run().times(0).return_const(());

        let mut mock_task2 = MockTask::new();
        mock_task2.expect_run().times(0).return_const(());

        {
            let mut timer_factory = TimerFactory::new();
            let tp1 = timer_factory.create_timer_provider();
            let tp2 = timer_factory.create_timer_provider();

            tp1.write_lock().start_timer(
                std::time::Duration::from_secs(1),
                InstanceHandle::new([1u8; 16]),
                move || mock_task1.run(),
            );

            tp2.write_lock().start_timer(
                std::time::Duration::from_millis(600),
                InstanceHandle::new([1u8; 16]),
                move || mock_task2.run(),
            );

            timer_factory.delete_timer_provider(&tp2);
        }
        std::thread::sleep(std::time::Duration::from_millis(2500));
    }
}
