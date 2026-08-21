use core::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
};
use critical_section::Mutex;

use crate::{
    dcps::dcps_reply::DcpsReply,
    infrastructure::error::{DdsError, DdsResult},
};

pub struct SharedReplySlot {
    inner: Mutex<RefCell<SharedReplyInner>>,
}

struct SharedReplyInner {
    data: Option<DcpsReply>,
    waker: Option<Waker>,
    closed: bool,
}

impl SharedReplySlot {
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(RefCell::new(SharedReplyInner {
                data: None,
                waker: None,
                closed: false,
            })),
        }
    }

    pub fn send(&self, reply: DcpsReply) {
        critical_section::with(|cs| {
            let mut inner = self.inner.borrow(cs).borrow_mut();
            inner.data = Some(reply);
            if let Some(w) = inner.waker.take() {
                w.wake();
            }
        });
    }

    pub fn close(&self) {
        critical_section::with(|cs| {
            let mut inner = self.inner.borrow(cs).borrow_mut();
            inner.closed = true;
            if let Some(w) = inner.waker.take() {
                w.wake();
            }
        });
    }

    pub fn receive(&'static self) -> SharedReplyFuture {
        SharedReplyFuture { slot: self }
    }
}

pub struct SharedReplyFuture {
    slot: &'static SharedReplySlot,
}

impl Future for SharedReplyFuture {
    type Output = DdsResult<DcpsReply>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        critical_section::with(|cs| {
            let mut inner = self.slot.inner.borrow(cs).borrow_mut();
            if let Some(reply) = inner.data.take() {
                Poll::Ready(Ok(reply))
            } else if inner.closed {
                Poll::Ready(Err(DdsError::AlreadyDeleted))
            } else {
                inner.waker.replace(cx.waker().clone());
                Poll::Pending
            }
        })
    }
}
