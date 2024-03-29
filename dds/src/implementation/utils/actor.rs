use std::{
    future::Future,
    sync::{
        atomic::{self, AtomicBool},
        Arc,
    },
};

use crate::infrastructure::error::{DdsError, DdsResult};

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

impl<A> PartialEq for ActorAddress<A>
where
    A: ActorHandler,
{
    fn eq(&self, other: &Self) -> bool {
        self.sender.same_channel(&other.sender)
    }
}

impl<A> Eq for ActorAddress<A> where A: ActorHandler {}

impl<A> ActorAddress<A>
where
    A: ActorHandler,
    A::Message: Send,
{
    pub async fn send_actor_message(&self, message: A::Message) -> DdsResult<()> {
        self.sender
            .send(message)
            .await
            .map_err(|_| DdsError::AlreadyDeleted)
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
    cancellation_token: Arc<AtomicBool>,
}

impl<A> Actor<A>
where
    A: ActorHandler + Send + 'static,
    A::Message: Send,
{
    pub fn spawn(mut actor: A, runtime: &tokio::runtime::Handle) -> Self {
        let (sender, mut mailbox) = tokio::sync::mpsc::channel::<A::Message>(16);

        let cancellation_token = Arc::new(AtomicBool::new(false));
        let cancellation_token_cloned = cancellation_token.clone();

        let join_handle = runtime.spawn(async move {
            while let Some(m) = mailbox.recv().await {
                if !cancellation_token_cloned.load(atomic::Ordering::Acquire) {
                    actor.handle_message(m).await;
                } else {
                    break;
                }
            }
        });
        Actor {
            sender,
            join_handle,
            cancellation_token,
        }
    }

    pub fn address(&self) -> ActorAddress<A> {
        ActorAddress {
            sender: self.sender.clone(),
        }
    }

    pub async fn send_actor_message(&self, message: A::Message) {
        self.sender.send(message).await.expect(
            "Receiver is guaranteed to exist while actor object is alive. Sending must succeed",
        );
    }
}

impl<A> Drop for Actor<A>
where
    A: ActorHandler,
{
    fn drop(&mut self) {
        self.cancellation_token
            .store(true, atomic::Ordering::Release);
        self.join_handle.abort();
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
        let actor = Actor::spawn(my_data, runtime.handle());

        assert_eq!(runtime.block_on(actor.increment(10)), 10)
    }

    #[test]
    fn actor_already_deleted() {
        let runtime = Runtime::new().unwrap();
        let my_data = MyData { data: 0 };
        let actor = Actor::spawn(my_data, runtime.handle());
        let actor_address = actor.address().clone();
        std::mem::drop(actor);
        assert_eq!(
            runtime.block_on(actor_address.increment(10)),
            Err(DdsError::AlreadyDeleted)
        );
    }
}
