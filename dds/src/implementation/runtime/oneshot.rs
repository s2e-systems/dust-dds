use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

#[derive(Debug)]
pub enum OneshotRecvError {
    SenderDropped,
}

pub fn oneshot<T>() -> (OneshotSender<T>, OneshotReceiver<T>) {
    let inner = Arc::new(Mutex::new(OneshotInner {
        data: None,
        waker: None,
        has_sender: true,
    }));
    (
        OneshotSender {
            inner: inner.clone(),
        },
        OneshotReceiver { inner },
    )
}

struct OneshotInner<T> {
    data: Option<T>,
    waker: Option<Waker>,
    has_sender: bool,
}

pub struct OneshotSender<T> {
    inner: Arc<Mutex<OneshotInner<T>>>,
}

impl<T> OneshotSender<T> {
    pub fn send(self, value: T) {
        {
            let mut inner_lock = self.inner.lock().expect("Mutex shouldn't be poisoned");
            inner_lock.data.replace(value);
            if let Some(w) = inner_lock.waker.take() {
                w.wake()
            }
        }

        // Don't run the drop since the channel is already woken up with a value
        std::mem::forget(self)
    }
}

impl<T> Drop for OneshotSender<T> {
    fn drop(&mut self) {
        let mut inner_lock = self.inner.lock().expect("Mutex shouldn't be poisoned");
        inner_lock.has_sender = false;
        // When the sender is dropped wake the waiting task
        // so that it can finish knowing it won't get any message
        if let Some(w) = inner_lock.waker.take() {
            w.wake()
        }
    }
}

pub struct OneshotReceiver<T> {
    inner: Arc<Mutex<OneshotInner<T>>>,
}

impl<T> Future for OneshotReceiver<T> {
    type Output = Result<T, OneshotRecvError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut inner_lock = self.inner.lock().expect("Mutex shouldn't be poisoned");
        if let Some(value) = inner_lock.data.take() {
            Poll::Ready(Ok(value))
        } else if !inner_lock.has_sender {
            Poll::Ready(Err(OneshotRecvError::SenderDropped))
        } else {
            inner_lock.waker.replace(cx.waker().clone());
            Poll::Pending
        }
    }
}
