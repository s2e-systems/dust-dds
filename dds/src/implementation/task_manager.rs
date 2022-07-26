use std::{
    sync::{
        atomic::{self, AtomicBool},
        Arc,
    },
    thread::JoinHandle,
};

pub struct TaskManager {
    enabled: Arc<AtomicBool>,
    quit: Arc<AtomicBool>,
    threads: Vec<JoinHandle<()>>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            enabled: Arc::new(AtomicBool::new(false)),
            quit: Arc::new(AtomicBool::new(false)),
            threads: vec![],
        }
    }

    pub fn spawn_enabled_periodic_task(
        &mut self,
        name: &'static str,
        mut task: impl FnMut() + Send + Sync + 'static,
        period: std::time::Duration,
    ) {
        let task_enabled = self.enabled.clone();
        let task_quit = self.quit.clone();

        self.threads.push(std::thread::spawn(move || loop {
            if task_quit.load(atomic::Ordering::SeqCst) {
                break;
            }

            if task_enabled.load(atomic::Ordering::SeqCst) {
                task();
            } else {
                println!("Task not enabled: {}", name);
            }

            std::thread::sleep(period);
        }));
    }

    pub fn enable_tasks(&mut self) {
        self.enabled.store(true, atomic::Ordering::SeqCst);
    }

    pub fn _disable_tasks(&mut self) {
        self.enabled.store(false, atomic::Ordering::SeqCst);
    }

    pub fn shutdown_tasks(&mut self) {
        self.quit.store(true, atomic::Ordering::SeqCst);

        while let Some(thread) = self.threads.pop() {
            thread.join().unwrap();
        }
    }
}

impl Drop for TaskManager {
    fn drop(&mut self) {
        self.shutdown_tasks();
    }
}
