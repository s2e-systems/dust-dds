use core::future::Future;

use crate::{
    dcps::runtime::{ChannelReceive, ChannelSend, DdsRuntime, Spawner},
    infrastructure::error::{DdsError, DdsResult},
};

pub trait MailHandler {
    type Mail;

    fn handle(&mut self, message: Self::Mail) -> impl Future<Output = ()> + Send;
}

pub struct ActorAddress<R, A>
where
    R: DdsRuntime,
    A: MailHandler,
    <A as MailHandler>::Mail: Send,
{
    mail_sender: R::ChannelSender<<A as MailHandler>::Mail>,
}

impl<R, A> Clone for ActorAddress<R, A>
where
    R: DdsRuntime,
    A: MailHandler,
    <A as MailHandler>::Mail: Send,
{
    fn clone(&self) -> Self {
        Self {
            mail_sender: self.mail_sender.clone(),
        }
    }
}

impl<R, A> ActorAddress<R, A>
where
    R: DdsRuntime,
    A: MailHandler,
    <A as MailHandler>::Mail: Send,
{
    pub async fn send_actor_mail(&self, mail: A::Mail) -> DdsResult<()>
    where
        A: MailHandler,
        A::Mail: Send + 'static,
    {
        self.mail_sender
            .send(mail)
            .await
            .map_err(|_| DdsError::AlreadyDeleted)
    }
}

pub struct Actor<R, A>
where
    R: DdsRuntime,
    A: MailHandler,
    A::Mail: Send,
{
    mail_sender: R::ChannelSender<<A as MailHandler>::Mail>,
    // join_handle: tokio::task::JoinHandle<()>,
}

impl<R, A> Actor<R, A>
where
    R: DdsRuntime,
    A: MailHandler + Send + 'static,
    A::Mail: Send + 'static,
{
    pub fn spawn(mut actor: A, spawner_handle: &R::SpawnerHandle) -> Self {
        let (mail_sender, mut mailbox_recv) = R::channel::<A::Mail>();

        spawner_handle.spawn(async move {
            while let Some(m) = mailbox_recv.receive().await {
                actor.handle(m).await;
            }
        });
        Actor { mail_sender }
    }

    pub fn address(&self) -> ActorAddress<R, A> {
        ActorAddress {
            mail_sender: self.mail_sender.clone(),
        }
    }

    pub async fn send_actor_mail(&self, mail: A::Mail) {
        self.mail_sender
            .send(mail)
            .await
            .expect("Message will always be sent when actor exists");
    }
}
