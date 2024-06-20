use std::{
    future::Future,
    pin::pin,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
    thread::{self, Thread},
};

pub fn block_on<T>(f: impl Future<Output = T>) -> T {
    struct ThreadWake(Thread);
    impl Wake for ThreadWake {
        fn wake(self: std::sync::Arc<Self>) {
            self.0.unpark();
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
