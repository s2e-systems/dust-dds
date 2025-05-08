use core::future::Future;

use super::mpsc::{mpsc_channel, MpscSender};
use crate::{
    dcps::runtime::{DdsRuntime, Spawner},
    infrastructure::error::{DdsError, DdsResult},
};

pub trait MailHandler {
    type Mail;

    fn handle(&mut self, message: Self::Mail) -> impl Future<Output = ()> + Send;
}

pub struct ActorAddress<A>
where
    A: MailHandler,
{
    mail_sender: MpscSender<<A as MailHandler>::Mail>,
}

impl<A> Clone for ActorAddress<A>
where
    A: MailHandler,
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
    A::Mail: Send,
{
    pub fn send_actor_mail(&self, mail: A::Mail) -> DdsResult<()>
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
{
    mail_sender: MpscSender<<A as MailHandler>::Mail>,
    // join_handle: tokio::task::JoinHandle<()>,
}

impl<A> Actor<A>
where
    A: MailHandler + Send + 'static,
    A::Mail: Send,
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

    pub fn send_actor_mail(&self, mail: A::Mail) {
        self.mail_sender
            .send(mail)
            .expect("Message will always be sent when actor exists");
    }
}
