use alloc::collections::VecDeque;
use core::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
};
use critical_section::Mutex;

pub struct AsyncMutex<T> {
    inner: Mutex<RefCell<AsyncMutexInner<T>>>,
}

struct AsyncMutexInner<T> {
    _data: T,
    locked: bool,
    waiters: VecDeque<Waker>,
}

impl<T> AsyncMutex<T> {
    pub const fn new(data: T) -> Self {
        Self {
            inner: Mutex::new(RefCell::new(AsyncMutexInner {
                _data: data,
                locked: false,
                waiters: VecDeque::new(),
            })),
        }
    }

    pub fn lock(&'static self) -> AsyncMutexLockFuture<T> {
        AsyncMutexLockFuture { mutex: self }
    }
}

pub struct AsyncMutexLockFuture<T: 'static> {
    mutex: &'static AsyncMutex<T>,
}

impl<T: 'static> Future for AsyncMutexLockFuture<T> {
    type Output = AsyncMutexGuard<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        critical_section::with(|cs| {
            let mut inner = self.mutex.inner.borrow(cs).borrow_mut();
            if !inner.locked {
                inner.locked = true;
                Poll::Ready(AsyncMutexGuard { mutex: self.mutex })
            } else {
                let waker = cx.waker();
                if !inner.waiters.iter().any(|w| w.will_wake(waker)) {
                    inner.waiters.push_back(waker.clone());
                }
                Poll::Pending
            }
        })
    }
}

pub struct AsyncMutexGuard<T: 'static> {
    mutex: &'static AsyncMutex<T>,
}

impl<T: 'static> Drop for AsyncMutexGuard<T> {
    fn drop(&mut self) {
        critical_section::with(|cs| {
            let mut inner = self.mutex.inner.borrow(cs).borrow_mut();
            inner.locked = false;
            if let Some(waker) = inner.waiters.pop_front() {
                waker.wake();
            }
        });
    }
}
