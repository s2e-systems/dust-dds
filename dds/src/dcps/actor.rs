use core::future::Future;

use crate::{
    dcps::channels::mpsc::{MpscSender, mpsc_channel},
    infrastructure::error::{DdsError, DdsResult},
    runtime::{DdsRuntime, Spawner},
};

pub trait MailHandler {
    type Mail;

    fn handle(&mut self, message: Self::Mail) -> impl Future<Output = ()> + Send;
}

pub struct ActorAddress<A>
where
    A: MailHandler,
    <A as MailHandler>::Mail: Send,
{
    mail_sender: MpscSender<<A as MailHandler>::Mail>,
}

impl<A> Clone for ActorAddress<A>
where
    A: MailHandler,
    <A as MailHandler>::Mail: Send,
{
    fn clone(&self) -> Self {
        Self {
            mail_sender: self.mail_sender.clone(),
        }
    }
}

impl<A> ActorAddress<A>
where
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
            .map_err(|_| DdsError::AlreadyDeleted)
    }
}

pub struct Actor<A>
where
    A: MailHandler,
    A::Mail: Send,
{
    mail_sender: MpscSender<<A as MailHandler>::Mail>,
    // join_handle: tokio::task::JoinHandle<()>,
}

impl<A> Actor<A>
where
    A: MailHandler + Send + 'static,
    A::Mail: Send + 'static,
{
    pub fn spawn<R: DdsRuntime>(mut actor: A, spawner_handle: &R::SpawnerHandle) -> Self {
        let (mail_sender, mailbox_recv) = mpsc_channel::<A::Mail>();

        spawner_handle.spawn(async move {
            while let Some(m) = mailbox_recv.recv().await {
                actor.handle(m).await;
            }
        });
        Actor { mail_sender }
    }

    pub fn address(&self) -> ActorAddress<A> {
        ActorAddress {
            mail_sender: self.mail_sender.clone(),
        }
    }

    pub async fn send_actor_mail(&self, mail: A::Mail) {
        self.mail_sender
            .send(mail)
            .expect("Message will always be sent when actor exists");
    }
}
