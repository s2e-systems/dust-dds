use std::{
    collections::VecDeque,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

pub fn mpsc_channel<T>() -> (MpscSender<T>, MpscReceiver<T>) {
    let inner = Arc::new(Mutex::new(MpscInner {
        data: VecDeque::new(),
        waker: None,
        sender_count: 1,
        is_closed: false,
    }));
    (
        MpscSender {
            inner: inner.clone(),
        },
        MpscReceiver { inner },
    )
}

struct MpscInner<T> {
    data: VecDeque<T>,
    waker: Option<Waker>,
    sender_count: usize,
    is_closed: bool,
}

impl<T> std::fmt::Debug for MpscInner<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MpscInner")
            .field("data_length", &self.data.len())
            .field("waker", &self.waker)
            .field("sender_count", &self.sender_count)
            .finish()
    }
}

#[derive(Debug)]
pub enum MpscSenderError {
    Closed,
}

pub struct MpscSender<T> {
    inner: Arc<Mutex<MpscInner<T>>>,
}

impl<T> Clone for MpscSender<T> {
    fn clone(&self) -> Self {
        let mut inner_lock = self.inner.lock().expect("Mutex shouldn't be poisoned");
        inner_lock.sender_count += 1;
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Drop for MpscSender<T> {
    fn drop(&mut self) {
        let mut inner_lock = self.inner.lock().expect("Mutex shouldn't be poisoned");
        inner_lock.sender_count -= 1;
        // When the sender is dropped wake the waiting task
        // so that it can finish knowing it won't get any message
        if inner_lock.sender_count == 0 {
            if let Some(w) = inner_lock.waker.take() {
                w.wake()
            }
        }
    }
}

impl<T> std::fmt::Debug for MpscSender<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MpscSender")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<T> MpscSender<T> {
    pub fn send(&self, value: T) -> Result<(), MpscSenderError> {
        let mut inner_lock = self.inner.lock().expect("Mutex shouldn't be poisoned");
        if inner_lock.is_closed {
            Err(MpscSenderError::Closed)
        } else {
            inner_lock.data.push_back(value);
            if let Some(w) = inner_lock.waker.take() {
                w.wake();
            }
            Ok(())
        }
    }

    pub fn is_closed(&self) -> bool {
        self.inner
            .lock()
            .expect("Mutex shouldn't be poisoned")
            .is_closed
    }
}

pub struct MpscReceiver<T> {
    inner: Arc<Mutex<MpscInner<T>>>,
}

impl<T> MpscReceiver<T> {
    pub async fn recv(&self) -> Option<T> {
        MpscReceiverFuture {
            inner: self.inner.clone(),
        }
        .await
    }

    pub fn close(&self) {
        let mut inner_lock = self.inner.lock().expect("Mutex shouldn't be poisoned");
        inner_lock.is_closed = true;
        if let Some(w) = inner_lock.waker.take() {
            w.wake();
        }
    }
}

struct MpscReceiverFuture<T> {
    inner: Arc<Mutex<MpscInner<T>>>,
}

impl<T> Future for MpscReceiverFuture<T> {
    type Output = Option<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut inner_lock = self.inner.lock().expect("Mutex shouldn't be poisoned");
        if let Some(value) = inner_lock.data.pop_front() {
            Poll::Ready(Some(value))
        } else if inner_lock.is_closed {
            Poll::Ready(None)
        } else {
            inner_lock.waker.replace(cx.waker().clone());
            Poll::Pending
        }
    }
}
