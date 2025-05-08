use std::{
    future::Future,
    pin::{pin, Pin},
    sync::{
        atomic::{self, AtomicBool},
        mpsc::{channel, Sender, TryRecvError},
        Arc, Mutex,
    },
    task::{Context, Poll, Wake, Waker},
    thread::{self, JoinHandle, Thread},
};

use crate::dcps::runtime::Spawner;

pub fn block_on<T>(f: impl Future<Output = T>) -> T {
    struct ThreadWake(Thread);
    impl Wake for ThreadWake {
        fn wake(self: std::sync::Arc<Self>) {
            self.wake_by_ref()
        }

        fn wake_by_ref(self: &Arc<Self>) {
            self.0.unpark()
        }
    }
    let waker = Waker::from(Arc::new(ThreadWake(thread::current())));
    let mut cx = Context::from_waker(&waker);
    let mut pinned_fut = pin!(f);
    loop {
        match pinned_fut.as_mut().poll(&mut cx) {
            Poll::Ready(t) => return t,
            Poll::Pending => thread::park(),
        }
    }
}

pub struct Task {
    future: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
    task_sender: Sender<Arc<Task>>,
    thread_handle: Thread,
    abort: AtomicBool,
}

impl Task {
    fn is_aborted(&self) -> bool {
        self.abort.load(atomic::Ordering::Acquire)
    }
}

impl Wake for Task {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref()
    }

    fn wake_by_ref(self: &Arc<Self>) {
        if !self.is_aborted() {
            self.task_sender.send(self.clone()).unwrap();
            self.thread_handle.unpark();
        }
    }
}

#[derive(Clone)]
pub struct ExecutorHandle {
    task_sender: Sender<Arc<Task>>,
    thread_handle: Thread,
}

impl ExecutorHandle {
    pub fn spawn(&self, f: impl Future<Output = ()> + Send + 'static) {
        let future = Box::pin(f);
        let task = Arc::new(Task {
            future: Mutex::new(future),
            task_sender: self.task_sender.clone(),
            thread_handle: self.thread_handle.clone(),
            abort: AtomicBool::new(false),
        });
        self.task_sender
            .send(task.clone())
            .expect("Should never fail to send");
        self.thread_handle.unpark();
    }
}

impl Spawner for ExecutorHandle {
    fn spawn(&self, f: impl Future<Output = ()> + Send + 'static) {
        self.spawn(f);
    }
}

pub struct Executor {
    task_sender: Sender<Arc<Task>>,
    executor_thread_handle: JoinHandle<()>,
}

impl Executor {
    pub fn new() -> Self {
        let (task_sender, task_receiver) = channel::<Arc<Task>>();
        let executor_thread_handle = std::thread::Builder::new()
            .name("Dust DDS Executor".to_string())
            .spawn(move || loop {
                match task_receiver.try_recv() {
                    Ok(task) => {
                        if !task.is_aborted() {
                            let waker = Waker::from(task.clone());
                            let mut cx = Context::from_waker(&waker);
                            let _ = task
                                .future
                                .try_lock()
                                .expect("Only ever locked here")
                                .as_mut()
                                .poll(&mut cx);
                        }
                    }
                    Err(TryRecvError::Empty) => thread::park(),
                    Err(TryRecvError::Disconnected) => break,
                }
            })
            .expect("failed to spawn thread");

        Self {
            task_sender,
            executor_thread_handle,
        }
    }

    pub fn handle(&self) -> ExecutorHandle {
        ExecutorHandle {
            task_sender: self.task_sender.clone(),
            thread_handle: self.executor_thread_handle.thread().clone(),
        }
    }
}
