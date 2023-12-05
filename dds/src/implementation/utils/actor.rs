use std::sync::{
    atomic::{self, AtomicBool},
    Arc, Once,
};

use lazy_static::lazy_static;

use crate::infrastructure::error::{DdsError, DdsResult};

static INITIALIZE_EXECUTOR: Once = Once::new();
const NUM_THREADS: usize = 8;

lazy_static! {
    pub static ref THE_RUNTIME: smol::Executor<'static> = smol::Executor::new();
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
    sender: smol::channel::Sender<Box<dyn GenericHandler<A> + Send>>,
}

impl<A> Clone for ActorAddress<A> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<A> ActorAddress<A> {
    pub async fn send_mail_and_await_reply<M>(&self, mail: M) -> DdsResult<M::Result>
    where
        A: MailHandler<M> + Send,
        M: Mail + Send + 'static,
        M::Result: Send + Sync,
    {
        let (response_sender, response_receiver) = async_oneshot::oneshot();

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
        M::Result: Send + Sync,
    {
        let (response_sender, response_receiver) = async_oneshot::oneshot();

        let mut send_result = self
            .sender
            .try_send(Box::new(ReplyMail::new(mail, response_sender)));
        // Try sending the mail until it succeeds. This is done instead of calling a tokio::task::block_in_place because this solution
        // would be only valid when the runtime is multithreaded. For single threaded runtimes this would still cause a panic.
        while let Err(receive_error) = send_result {
            match receive_error {
                smol::channel::TrySendError::Full(mail) => {
                    send_result = self.sender.try_send(mail);
                }
                smol::channel::TrySendError::Closed(_) => return Err(DdsError::AlreadyDeleted),
            }
        }

        // Receive on a try_recv() loop checking for error instead of a call to recv() to avoid blocking the thread. This would not cause
        // a Tokio runtime panic since it is using an std channel but it could cause a single-threaded runtime to hang and further tasks not
        // being executed.
        let mut receive_result = response_receiver.try_recv();
        while let Err(receive_error) = receive_result {
            match receive_error {
                async_oneshot::TryRecvError::Empty(response_receiver) => {
                    receive_result = response_receiver.try_recv();
                }
                async_oneshot::TryRecvError::Closed => return Err(DdsError::AlreadyDeleted),
            }
        }
        Ok(receive_result
            .map_err(|_| ())
            .expect("Receive result should be Ok"))
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
                smol::channel::TrySendError::Full(mail) => {
                    send_result = self.sender.try_send(mail);
                }
                smol::channel::TrySendError::Closed(_) => return Err(DdsError::AlreadyDeleted),
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
    sender: Option<async_oneshot::Sender<M::Result>>,
}

impl<M> ReplyMail<M>
where
    M: Mail,
{
    fn new(message: M, sender: async_oneshot::Sender<M::Result>) -> Self {
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
    M::Result: Send + Sync,
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
    sender: smol::channel::Sender<Box<dyn GenericHandler<A> + Send>>,
    _join_handle: smol::Task<()>,
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
        M::Result: Send + Sync,
    {
        let (response_sender, response_receiver) = async_oneshot::oneshot();

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
        self.sender.close();
        // self.join_handle.cancel();
    }
}

struct SpawnedActor<A> {
    value: A,
    mailbox: smol::channel::Receiver<Box<dyn GenericHandler<A> + Send>>,
}

pub fn spawn_actor<A>(actor: A) -> Actor<A>
where
    A: Send + 'static,
{
    INITIALIZE_EXECUTOR.call_once(|| {

        // Create an executor thread pool.
        for n in 0..NUM_THREADS {
            // A pending future is one that simply yields forever.
            std::thread::Builder::new()
                .name(format!("dust-dds-worker-thread-{}", n))
                .spawn(|| smol::future::block_on(THE_RUNTIME.run(std::future::pending::<()>())))
                .expect("cannot spawn executor thread");
        }
    });

    let (sender, mailbox) = smol::channel::bounded(16);

    let mut actor_obj = SpawnedActor {
        value: actor,
        mailbox,
    };
    let cancellation_token = Arc::new(AtomicBool::new(false));
    let cancellation_token_cloned = cancellation_token.clone();

    let join_handle = THE_RUNTIME.spawn(async move {
        while let Ok(mut m) = actor_obj.mailbox.recv().await {
            if !cancellation_token_cloned.load(atomic::Ordering::Acquire) {
                m.handle(&mut actor_obj.value).await;
            } else {
                break;
            }
        }
    });

    Actor {
        sender,
        _join_handle: join_handle,
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
