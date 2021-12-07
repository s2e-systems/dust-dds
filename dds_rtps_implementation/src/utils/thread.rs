use std::thread::JoinHandle;

pub trait Thread {
    type Handle;

    fn spawn<F>(f: F) -> std::io::Result<Self::Handle>
    where
        F: FnOnce(),
        F: Send + 'static;
}

pub struct StdThread;

impl Thread for StdThread {
    type Handle = JoinHandle<()>;

    fn spawn<F>(f: F) -> std::io::Result<JoinHandle<()>>
    where
        F: FnOnce(),
        F: Send + 'static,
    {
        Ok(std::thread::spawn(f))
    }
}
