use core::ops::{Deref, DerefMut};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::mutex::{Mutex, MutexGuard};

/// A zero-copy MPSC channel allowing producers to reserve, populate, and commit data directly.
pub struct ZeroCopyChannel<T: 'static, const N: usize> {
    storage: [Mutex<CriticalSectionRawMutex, T>; N],
    free_queue: Channel<CriticalSectionRawMutex, usize, N>,
    ready_queue: Channel<CriticalSectionRawMutex, usize, N>,
}

impl<T: Default, const N: usize> ZeroCopyChannel<T, N> {
    /// Creates a new channel and pre-fills the free queue.
    pub fn new() -> Self {
        let channel = Self {
            storage: core::array::from_fn(|_| Mutex::new(T::default())),
            free_queue: Channel::new(),
            ready_queue: Channel::new(),
        };

        for i in 0..N {
            channel.free_queue.try_send(i).unwrap();
        }

        channel
    }

    /// Splits the channel into a clonable Sender and a non-clonable Receiver.
    pub fn split(&'static self) -> (ZeroCopySender<T, N>, ZeroCopyReceiver<T, N>) {
        (
            ZeroCopySender { channel: self },
            ZeroCopyReceiver { channel: self },
        )
    }
}

// ==========================================
// PRODUCER SIDE
// ==========================================

/// A clonable sender for the ZeroCopyChannel.
pub struct ZeroCopySender<T: 'static, const N: usize> {
    channel: &'static ZeroCopyChannel<T, N>,
}

impl<T: 'static, const N: usize> Clone for ZeroCopySender<T, N> {
    fn clone(&self) -> Self {
        ZeroCopySender {
            channel: self.channel,
        }
    }
}

impl<T: 'static, const N: usize> ZeroCopySender<T, N> {
    /// Reserves a slot in the channel, returning a permit that allows mutating the data.
    pub async fn reserve(&self) -> SendPermit<'static, T, N> {
        let index = self.channel.free_queue.receive().await;
        // We exclusively own this index from the queue, so try_lock will NEVER fail.
        let guard = self.channel.storage[index].try_lock().unwrap();

        SendPermit {
            channel: self.channel,
            index,
            guard: Some(guard),
        }
    }

    /// Conveniently reserves a slot, writes the provided item to it, and commits it.
    pub async fn send(&self, item: T) {
        let mut permit = self.reserve().await;
        *permit = item;
        // Permit is implicitly dropped and committed here.
    }
}

/// A permit representing exclusive mutable access to a slot in the channel.
pub struct SendPermit<'a, T: 'static, const N: usize> {
    channel: &'a ZeroCopyChannel<T, N>,
    index: usize,
    guard: Option<MutexGuard<'a, CriticalSectionRawMutex, T>>,
}

impl<'a, T, const N: usize> Deref for SendPermit<'a, T, N> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.guard.as_ref().unwrap()
    }
}

impl<'a, T, const N: usize> DerefMut for SendPermit<'a, T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.as_mut().unwrap()
    }
}

impl<'a, T, const N: usize> Drop for SendPermit<'a, T, N> {
    fn drop(&mut self) {
        if self.guard.is_some() {
            // Drop the MutexGuard first to unlock
            self.guard.take();
            // Permit is dropped, commit the data to the ready queue implicitly
            self.channel.ready_queue.try_send(self.index).unwrap();
        }
    }
}

// ==========================================
// CONSUMER SIDE
// ==========================================

/// A non-clonable receiver for the ZeroCopyChannel.
pub struct ZeroCopyReceiver<T: 'static, const N: usize> {
    channel: &'static ZeroCopyChannel<T, N>,
}

impl<T: 'static, const N: usize> ZeroCopyReceiver<T, N> {
    /// Awaits and receives the next ready permit from producers.
    pub async fn receive(&self) -> ReceivePermit<'static, T, N> {
        let index = self.channel.ready_queue.receive().await;
        // We exclusively own this index from the ready queue, so try_lock will NEVER fail.
        let guard = self.channel.storage[index].try_lock().unwrap();

        ReceivePermit {
            channel: self.channel,
            index,
            guard: Some(guard),
        }
    }
}

/// A permit representing exclusive read/write access to a received slot.
pub struct ReceivePermit<'a, T: 'static, const N: usize> {
    channel: &'a ZeroCopyChannel<T, N>,
    index: usize,
    guard: Option<MutexGuard<'a, CriticalSectionRawMutex, T>>,
}

impl<'a, T, const N: usize> Deref for ReceivePermit<'a, T, N> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.guard.as_ref().unwrap()
    }
}

impl<'a, T, const N: usize> DerefMut for ReceivePermit<'a, T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.as_mut().unwrap()
    }
}

impl<'a, T, const N: usize> Drop for ReceivePermit<'a, T, N> {
    fn drop(&mut self) {
        // 1. Drop the MutexGuard first to unlock
        self.guard.take();
        // 2. Return the index to the free pool so producers can use it again
        self.channel.free_queue.try_send(self.index).unwrap();
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::boxed::Box;

    #[tokio::test]
    async fn test_basic_send_receive() {
        // Leak a boxed channel to get a 'static reference for tests
        let channel = Box::leak(Box::new(ZeroCopyChannel::<i32, 4>::new()));
        let (sender, receiver) = channel.split();

        // Reserve a permit
        let mut send_permit = sender.reserve().await;
        *send_permit = 42;
        drop(send_permit); // implicitly commits

        // Alternatively, use the direct send method
        sender.send(100).await;

        // Receive the permits
        let receive_permit = receiver.receive().await;
        assert_eq!(*receive_permit, 42);

        let receive_permit2 = receiver.receive().await;
        assert_eq!(*receive_permit2, 100);
        // implicit drop of receive_permit returns the slot to free queue
    }

    #[tokio::test]
    async fn test_implicit_commit_on_drop() {
        let channel = Box::leak(Box::new(ZeroCopyChannel::<i32, 2>::new()));
        let (sender, receiver) = channel.split();

        {
            let mut p1 = sender.reserve().await;
            let mut p2 = sender.reserve().await;

            *p1 = 100;
            *p2 = 200;

            drop(p1);
            drop(p2);
        }

        let r1 = receiver.receive().await;
        assert_eq!(*r1, 100);
        let r2 = receiver.receive().await;
        assert_eq!(*r2, 200);
    }

    #[tokio::test]
    async fn test_cloned_sender() {
        let channel = Box::leak(Box::new(ZeroCopyChannel::<i32, 2>::new()));
        let (sender1, receiver) = channel.split();
        let sender2 = sender1.clone();

        let mut p1 = sender1.reserve().await;
        *p1 = 1;
        drop(p1);

        let mut p2 = sender2.reserve().await;
        *p2 = 2;
        drop(p2);

        let r1 = receiver.receive().await;
        assert_eq!(*r1, 1);
        drop(r1); // MUST drop to free the slot before we run out of space if it were larger, 
        // but we drop to be explicit.

        let r2 = receiver.receive().await;
        assert_eq!(*r2, 2);
    }
}
