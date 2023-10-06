use std::sync::{
    atomic::{self, AtomicBool},
    Arc,
};

use lazy_static::lazy_static;

use crate::infrastructure::error::{DdsError, DdsResult};

lazy_static! {
    pub static ref THE_RUNTIME: tokio::runtime::Runtime =
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_stack_size(4 * 1024 * 1024)
            .build()
            .expect("Failed to create Tokio runtime");
}

pub trait Mail {
    type Result;
}

#[async_trait::async_trait]
pub trait MailHandler<M>
where
    M: Mail,
    Self: Sized,
{
    async fn handle(&mut self, mail: M) -> M::Result;
}

#[derive(Debug)]
pub struct ActorAddress<A> {
    sender: tokio::sync::mpsc::Sender<Box<dyn GenericHandler<A> + Send>>,
}

impl<A> Clone for ActorAddress<A> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<A> PartialEq for ActorAddress<A> {
    fn eq(&self, other: &Self) -> bool {
        self.sender.same_channel(&other.sender)
    }
}

impl<A> Eq for ActorAddress<A> {}

impl<A> ActorAddress<A> {
    pub async fn send_mail_and_await_reply<M>(&self, mail: M) -> DdsResult<M::Result>
    where
        A: MailHandler<M> + Send,
        M: Mail + Send + 'static,
        M::Result: Send,
    {
        let (response_sender, response_receiver) = tokio::sync::oneshot::channel();

        self.sender
            .send(Box::new(ReplyMail::new(mail, response_sender)))
            .await
            .map_err(|_| DdsError::AlreadyDeleted)?;
        response_receiver
            .await
            .map_err(|_| DdsError::AlreadyDeleted)
    }

    pub fn send_mail_and_await_reply_blocking<M>(&self, mail: M) -> DdsResult<M::Result>
    where
        A: MailHandler<M> + Send,
        M: Mail + Send + 'static,
        M::Result: Send,
    {
        let (response_sender, mut response_receiver) = tokio::sync::oneshot::channel();

        let mut send_result = self
            .sender
            .try_send(Box::new(ReplyMail::new(mail, response_sender)));
        // Try sending the mail until it succeeds. This is done instead of calling a tokio::task::block_in_place because this solution
        // would be only valid when the runtime is multithreaded. For single threaded runtimes this would still cause a panic.
        while let Err(receive_error) = send_result {
            match receive_error {
                tokio::sync::mpsc::error::TrySendError::Full(mail) => {
                    send_result = self.sender.try_send(mail);
                }

                tokio::sync::mpsc::error::TrySendError::Closed(_) => {
                    return Err(DdsError::AlreadyDeleted)
                }
            }
        }

        // Receive on a try_recv() loop checking for error instead of a call to recv() to avoid blocking the thread. This would not cause
        // a Tokio runtime panic since it is using an std channel but it could cause a single-threaded runtime to hang and further tasks not
        // being executed.
        let mut receive_result = response_receiver.try_recv();
        while let Err(receive_error) = receive_result {
            match receive_error {
                tokio::sync::oneshot::error::TryRecvError::Empty => {
                    receive_result = response_receiver.try_recv();
                }

                tokio::sync::oneshot::error::TryRecvError::Closed => {
                    return Err(DdsError::AlreadyDeleted)
                }
            }
        }
        Ok(receive_result.expect("Receive result should be Ok"))
    }

    pub async fn send_mail<M>(&self, mail: M) -> DdsResult<()>
    where
        A: MailHandler<M> + Send,
        M: Mail + Send + 'static,
    {
        self.sender
            .send(Box::new(CommandMail::new(mail)))
            .await
            .map_err(|_| DdsError::AlreadyDeleted)
    }

    pub fn send_mail_blocking<M>(&self, mail: M) -> DdsResult<()>
    where
        A: MailHandler<M> + Send,
        M: Mail + Send + 'static,
    {
        let mut send_result = self.sender.try_send(Box::new(CommandMail::new(mail)));
        // Try sending the mail until it succeeds. This is done instead of calling a tokio::task::block_in_place because this solution
        // would be only valid when the runtime is multithreaded. For single threaded runtimes this would still cause a panic.
        while let Err(error) = send_result {
            match error {
                tokio::sync::mpsc::error::TrySendError::Full(mail) => {
                    send_result = self.sender.try_send(mail);
                }

                tokio::sync::mpsc::error::TrySendError::Closed(_) => {
                    return Err(DdsError::AlreadyDeleted)
                }
            }
        }
        Ok(())
    }
}

#[async_trait::async_trait]
trait GenericHandler<A> {
    async fn handle(&mut self, actor: &mut A);
}

struct CommandMail<M> {
    mail: Option<M>,
}

impl<M> CommandMail<M> {
    fn new(mail: M) -> Self {
        Self { mail: Some(mail) }
    }
}

#[async_trait::async_trait]
impl<A, M> GenericHandler<A> for CommandMail<M>
where
    A: MailHandler<M> + Send,
    M: Mail + Send,
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

impl<M> ReplyMail<M>
where
    M: Mail,
{
    fn new(message: M, sender: tokio::sync::oneshot::Sender<M::Result>) -> Self {
        Self {
            mail: Some(message),
            sender: Some(sender),
        }
    }
}

#[async_trait::async_trait]
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

pub struct Actor<A> {
    sender: tokio::sync::mpsc::Sender<Box<dyn GenericHandler<A> + Send>>,
    join_handle: tokio::task::JoinHandle<()>,
    cancellation_token: Arc<AtomicBool>,
}

impl<A> Actor<A> {
    pub fn address(&self) -> ActorAddress<A> {
        ActorAddress {
            sender: self.sender.clone(),
        }
    }

    pub async fn send_mail_and_await_reply<M>(&self, mail: M) -> M::Result
    where
        A: MailHandler<M> + Send,
        M: Mail + Send + 'static,
        M::Result: Send,
    {
        let (response_sender, response_receiver) = tokio::sync::oneshot::channel();

        self.sender
            .send(Box::new(ReplyMail::new(mail, response_sender)))
            .await
            .map_err(|_| ())
            .expect(
                "Receiver is guaranteed to exist while actor object is alive. Sending must succeed",
            );

        response_receiver
            .await
            .expect("Message is always processed as long as actor object exists")
    }

    pub async fn send_mail<M>(&self, mail: M)
    where
        A: MailHandler<M> + Send,
        M: Mail + Send + 'static,
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

impl<A> Drop for Actor<A> {
    fn drop(&mut self) {
        self.cancellation_token
            .store(true, atomic::Ordering::Release);
        self.join_handle.abort();
    }
}

struct SpawnedActor<A> {
    value: A,
    mailbox: tokio::sync::mpsc::Receiver<Box<dyn GenericHandler<A> + Send>>,
}

pub fn spawn_actor<A>(actor: A) -> Actor<A>
where
    A: Send + 'static,
{
    let (sender, mailbox) = tokio::sync::mpsc::channel(16);

    let mut actor_obj = SpawnedActor {
        value: actor,
        mailbox,
    };
    let cancellation_token = Arc::new(AtomicBool::new(false));
    let cancellation_token_cloned = cancellation_token.clone();

    let join_handle = THE_RUNTIME.spawn(async move {
        while let Some(mut m) = actor_obj.mailbox.recv().await {
            if !cancellation_token_cloned.load(atomic::Ordering::Acquire) {
                m.handle(&mut actor_obj.value).await;
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

#[cfg(test)]
mod tests {
    use dust_dds_derive::actor_interface;

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
        let my_data = MyData { data: 0 };
        let actor = spawn_actor(my_data);

        assert_eq!(
            actor
                .address()
                .send_mail_and_await_reply_blocking(increment::new(10))
                .unwrap(),
            10
        )
    }

    #[test]
    fn actor_already_deleted() {
        let my_data = MyData { data: 0 };
        let actor = spawn_actor(my_data);
        let actor_address = actor.address().clone();
        std::mem::drop(actor);
        assert_eq!(
            actor_address.send_mail_and_await_reply_blocking(increment::new(10)),
            Err(DdsError::AlreadyDeleted)
        );
    }
}
