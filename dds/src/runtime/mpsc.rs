use std::{
    collections::VecDeque,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

use crate::{
    dcps::runtime::{ChannelReceive, ChannelSend},
    infrastructure::error::{DdsError, DdsResult},
};

pub fn mpsc_channel<T>() -> (MpscSender<T>, MpscReceiver<T>) {
    let inner = Arc::new(Mutex::new(MpscInner {
        data: VecDeque::with_capacity(64),
        waker: None,
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
    is_closed: bool,
}

impl<T> std::fmt::Debug for MpscInner<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MpscInner")
            .field("data_length", &self.data.len())
            .field("waker", &self.waker)
            .finish()
    }
}

#[derive(Debug)]
pub enum MpscSenderError {
    Closed,
}

impl From<MpscSenderError> for DdsError {
    fn from(_value: MpscSenderError) -> Self {
        DdsError::AlreadyDeleted
    }
}

pub struct MpscSender<T> {
    inner: Arc<Mutex<MpscInner<T>>>,
}

impl<T> Clone for MpscSender<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> ChannelSend<T> for MpscSender<T>
where
    T: Send,
{
    async fn send(&self, value: T) -> DdsResult<()> {
        MpscSender::send(self, value).map_err(|_| DdsError::AlreadyDeleted)
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
}

impl<T> ChannelReceive<T> for MpscReceiver<T>
where
    T: Send,
{
    async fn receive(&mut self) -> Option<T> {
        self.recv().await
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
