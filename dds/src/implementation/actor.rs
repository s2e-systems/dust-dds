use std::{future::Future, sync::Arc};

use crate::infrastructure::error::{DdsError, DdsResult};

pub const DEFAULT_ACTOR_BUFFER_SIZE: usize = 16;

#[derive(Debug)]
pub struct ActorAddress<A>
where
    A: ActorHandler,
{
    sender: tokio::sync::mpsc::Sender<A::Message>,
}

impl<A> Clone for ActorAddress<A>
where
    A: ActorHandler,
{
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<A> ActorAddress<A>
where
    A: ActorHandler,
{
    pub async fn send_actor_message(&self, message: A::Message) -> DdsResult<()> {
        match self.sender.send(message).await {
            Ok(_) => Ok(()),
            Err(_) => Err(DdsError::AlreadyDeleted),
        }
    }
}

#[derive(Debug)]
pub struct ActorWeakAddress<A>
where
    A: ActorHandler,
{
    sender: tokio::sync::mpsc::WeakSender<A::Message>,
}

impl<A> Clone for ActorWeakAddress<A>
where
    A: ActorHandler,
{
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<A> ActorWeakAddress<A>
where
    A: ActorHandler,
{
    pub fn upgrade(&self) -> DdsResult<ActorAddress<A>> {
        if let Some(sender) = self.sender.upgrade() {
            Ok(ActorAddress { sender })
        } else {
            Err(DdsError::AlreadyDeleted)
        }
    }
}

pub trait ActorHandler {
    type Message;

    fn handle_message(&mut self, message: Self::Message) -> impl Future<Output = ()> + Send;
}

pub struct Actor<A>
where
    A: ActorHandler,
{
    sender: tokio::sync::mpsc::Sender<A::Message>,
    join_handle: tokio::task::JoinHandle<()>,
    notify_stop: Arc<tokio::sync::Notify>,
}

impl<A> Actor<A>
where
    A: ActorHandler + Send + 'static,
    A::Message: Send,
{
    pub fn spawn(mut actor: A, runtime: &tokio::runtime::Handle, buffer_size: usize) -> Self {
        let (sender, mut mailbox) = tokio::sync::mpsc::channel::<A::Message>(buffer_size);
        let notify_stop = Arc::new(tokio::sync::Notify::new());
        let notify_clone = notify_stop.clone();

        let join_handle = runtime.spawn(async move {
            loop {
                tokio::select! {
                    m = mailbox.recv() => {
                        match m {
                            Some(message) => actor.handle_message(message).await,
                            None => break,
                        }
                    }
                    _ = notify_clone.notified() => {
                        mailbox.close();
                    }
                }
            }
        });
        Actor {
            sender,
            join_handle,
            notify_stop,
        }
    }

    pub fn address(&self) -> ActorWeakAddress<A> {
        ActorWeakAddress {
            sender: self.sender.downgrade(),
        }
    }

    pub async fn stop(self) {
        self.notify_stop.notify_one();
        self.join_handle.await.ok();
    }

    pub async fn send_actor_message(&self, message: A::Message) -> DdsResult<()> {
        match self.sender.send(message).await {
            Ok(_) => Ok(()),
            Err(_) => Err(DdsError::AlreadyDeleted),
        }
    }
}

#[cfg(test)]
mod tests {
    use dust_dds_derive::actor_interface;
    use tokio::runtime::Runtime;

    use super::*;

    pub struct MyData {
        data: u8,
    }

    #[actor_interface]
    impl MyData {
        async fn increment(&mut self, value: u8) -> u8 {
            self.data += value;
            self.data
        }

        async fn decrement(&mut self) {
            self.data -= 1;
        }

        async fn try_increment(&mut self) -> DdsResult<()> {
            self.data -= 1;
            Ok(())
        }
    }

    #[test]
    fn actor_increment() {
        let runtime = Runtime::new().unwrap();
        let my_data = MyData { data: 0 };
        let actor = Actor::spawn(my_data, runtime.handle(), DEFAULT_ACTOR_BUFFER_SIZE);

        assert_eq!(runtime.block_on(actor.increment(10)).unwrap(), 10)
    }

    #[test]
    fn actor_stop_should_not_block() {
        let runtime = Runtime::new().unwrap();
        let my_data = MyData { data: 0 };
        let actor = Actor::spawn(my_data, runtime.handle(), DEFAULT_ACTOR_BUFFER_SIZE);

        assert_eq!(runtime.block_on(actor.increment(10)).unwrap(), 10);

        runtime.block_on(actor.stop());
    }

    #[test]
    fn actor_send_message_after_stop_should_return_error() {
        let runtime = Runtime::new().unwrap();
        let my_data = MyData { data: 0 };
        let actor = Actor::spawn(my_data, runtime.handle(), DEFAULT_ACTOR_BUFFER_SIZE);
        let actor_address = actor.address().upgrade().unwrap();

        assert_eq!(runtime.block_on(actor.increment(10)).unwrap(), 10);

        runtime.block_on(actor.stop());

        assert!(runtime.block_on(actor_address.increment(10)).is_err());
    }

    #[test]
    fn actor_already_deleted() {
        let runtime = Runtime::new().unwrap();
        let my_data = MyData { data: 0 };
        let actor = Actor::spawn(my_data, runtime.handle(), DEFAULT_ACTOR_BUFFER_SIZE);
        let actor_address = actor.address().clone();
        std::mem::drop(actor);
        assert!(actor_address.upgrade().is_err());
    }
}
