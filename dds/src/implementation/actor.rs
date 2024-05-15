use std::future::Future;

use crate::infrastructure::error::{DdsError, DdsResult};

pub const DEFAULT_ACTOR_BUFFER_SIZE: usize = 16;

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
    A::Message: Send,
{
    pub fn upgrade(&self) -> DdsResult<Actor<A>> {
        if let Some(sender) = self.sender.upgrade() {
            Ok(Actor { sender })
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
}

impl<A> Actor<A>
where
    A: ActorHandler + Send + 'static,
    A::Message: Send,
{
    pub fn spawn(mut actor: A, runtime: &tokio::runtime::Handle, buffer_size: usize) -> Self {
        let (sender, mut mailbox) = tokio::sync::mpsc::channel::<A::Message>(buffer_size);

        runtime.spawn(async move {
            while let Some(m) = mailbox.recv().await {
                actor.handle_message(m).await;
            }
        });
        Actor { sender }
    }

    pub fn address(&self) -> ActorWeakAddress<A> {
        ActorWeakAddress {
            sender: self.sender.downgrade(),
        }
    }

    pub async fn send_actor_message(&self, message: A::Message) {
        self.sender.send(message).await.expect(
            "Receiver is guaranteed to exist while actor object is alive. Sending must succeed",
        );
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

        assert_eq!(runtime.block_on(actor.increment(10)), 10)
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
