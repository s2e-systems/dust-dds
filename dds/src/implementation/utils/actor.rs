use std::{
    future::Future,
    pin::Pin,
    sync::{
        atomic::{self, AtomicBool},
        Arc,
    },
};

use crate::infrastructure::error::{DdsError, DdsResult};

pub trait Mail {
    type Result;
}

pub trait MailHandler<M>
where
    M: Mail,
    Self: Sized,
{
    fn handle(&mut self, mail: M) -> impl Future<Output = M::Result> + Send;
}

#[derive(Debug)]
pub struct ActorAddress<A>
where
    A: ActorHandler,
{
    sender: tokio::sync::mpsc::Sender<Box<dyn GenericHandlerDyn<A> + Send>>,
    sender_: tokio::sync::mpsc::Sender<A::Message>,
}

impl<A> Clone for ActorAddress<A>
where
    A: ActorHandler,
{
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            sender_: self.sender_.clone(),
        }
    }
}

impl<A> PartialEq for ActorAddress<A>
where
    A: ActorHandler,
{
    fn eq(&self, other: &Self) -> bool {
        self.sender.same_channel(&other.sender) && self.sender_.same_channel(&other.sender_)
    }
}

impl<A> Eq for ActorAddress<A> where A: ActorHandler {}

impl<A> ActorAddress<A>
where
    A: ActorHandler,
{
    pub async fn send_actor_message(&self, message: A::Message) -> DdsResult<()> {
        self.sender_
            .send(message)
            .await
            .map_err(|_| DdsError::AlreadyDeleted)
    }

    pub async fn send_mail<M>(&self, mail: M) -> DdsResult<()>
    where
        A: MailHandler<M> + Send,
        M: Mail + Send + 'static,
        <M as Mail>::Result: Send,
    {
        self.sender
            .send(Box::new(CommandMail::new(mail)))
            .await
            .map_err(|_| DdsError::AlreadyDeleted)
    }
}

// Workaround for not being able to make a dyn object out of a trait with async
// https://rust-lang.github.io/async-fundamentals-initiative/evaluation/case-studies/builder-provider-api.html#dynamic-dispatch-behind-the-api
trait GenericHandlerDyn<A> {
    fn handle<'a, 'b>(
        &'a mut self,
        actor: &'b mut A,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
        Self: 'b;
}

impl<A, T> GenericHandlerDyn<A> for T
where
    T: GenericHandler<A>,
{
    fn handle<'a, 'b>(
        &'a mut self,
        actor: &'b mut A,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
        Self: 'b,
    {
        Box::pin(<Self as GenericHandler<A>>::handle(self, actor))
    }
}

trait GenericHandler<A> {
    fn handle(&mut self, actor: &mut A) -> impl Future<Output = ()> + Send;
}

struct CommandMail<M> {
    mail: Option<M>,
}

impl<M> CommandMail<M> {
    fn new(mail: M) -> Self {
        Self { mail: Some(mail) }
    }
}

impl<A, M> GenericHandler<A> for CommandMail<M>
where
    A: MailHandler<M> + Send,
    M: Mail + Send,
    M::Result: Send,
{
    async fn handle(&mut self, actor: &mut A) {
        <A as MailHandler<M>>::handle(
            actor,
            self.mail
                .take()
                .expect("Mail should be processed only once"),
        )
        .await;
    }
}

struct ReplyMail<M>
where
    M: Mail,
{
    // Both fields have to be inside an option because later on the contents
    // have to be moved out and the struct. Because the struct is passed as a Boxed
    // trait object this is only feasible by using the Option fields.
    mail: Option<M>,
    sender: Option<tokio::sync::oneshot::Sender<M::Result>>,
}

impl<A, M> GenericHandler<A> for ReplyMail<M>
where
    A: MailHandler<M> + Send,
    M: Mail + Send,
    M::Result: Send,
{
    async fn handle(&mut self, actor: &mut A) {
        let result = <A as MailHandler<M>>::handle(
            actor,
            self.mail
                .take()
                .expect("Mail should be processed only once"),
        )
        .await;
        self.sender
            .take()
            .expect("Mail should be processed only once")
            .send(result)
            .map_err(|_| "Remove need for Debug on message send type")
            .expect("Sending should never fail");
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
    sender_: tokio::sync::mpsc::Sender<A::Message>,
    sender: tokio::sync::mpsc::Sender<Box<dyn GenericHandlerDyn<A> + Send>>,
    join_handle: tokio::task::JoinHandle<()>,
    cancellation_token: Arc<AtomicBool>,
}

impl<A> Actor<A>
where
    A: ActorHandler + Send + 'static,
    A::Message: Send,
{
    pub fn spawn(mut actor: A, runtime: &tokio::runtime::Handle) -> Self {
        let (sender, mut mailbox) =
            tokio::sync::mpsc::channel::<Box<dyn GenericHandlerDyn<A> + Send>>(16);

        let (sender_, mut mailbox_) = tokio::sync::mpsc::channel::<A::Message>(16);

        let cancellation_token = Arc::new(AtomicBool::new(false));
        let cancellation_token_cloned = cancellation_token.clone();

        let join_handle = runtime.spawn(async move {
            loop {
                tokio::select! {
                    Some(mut m) = mailbox.recv() => {
                        if !cancellation_token_cloned.load(atomic::Ordering::Acquire) {
                            m.handle(&mut actor).await;
                        } else {
                            break;
                        }
                    },
                    Some(m) = mailbox_.recv() => {
                        if !cancellation_token_cloned.load(atomic::Ordering::Acquire) {
                            actor.handle_message(m).await;
                        } else {
                            break;
                        }
                    }
                }
            }
        });

        Actor {
            sender_,
            sender,
            join_handle,
            cancellation_token,
        }
    }

    pub fn address(&self) -> ActorAddress<A> {
        ActorAddress {
            sender: self.sender.clone(),
            sender_: self.sender_.clone(),
        }
    }

    pub async fn send_actor_message(&self, message: A::Message) -> () {
        self.sender_.send(message).await.expect(
            "Receiver is guaranteed to exist while actor object is alive. Sending must succeed",
        );
    }

    pub async fn send_mail<M>(&self, mail: M)
    where
        A: MailHandler<M> + Send,
        M: Mail + Send + 'static,
        <M as Mail>::Result: Send,
    {
        self.sender
            .send(Box::new(CommandMail::new(mail)))
            .await
            .map_err(|_| ())
            .expect(
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
