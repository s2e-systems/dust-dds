use crate::infrastructure::error::{DdsError, DdsResult};

use super::runtime::{
    executor::ExecutorHandle,
    mpsc::{mpsc_channel, MpscSender},
    oneshot::{oneshot, OneshotReceiver, OneshotSender},
};

pub trait Mail {
    type Result;
}

pub trait MailHandler<M>
where
    M: Mail,
{
    fn handle(&mut self, message: M) -> M::Result;
}

struct ReplyMail<M>
where
    M: Mail,
{
    mail: Option<M>,
    reply_sender: Option<OneshotSender<M::Result>>,
}

pub struct ReplyReceiver<M>
where
    M: Mail,
{
    reply_receiver: OneshotReceiver<M::Result>,
}

impl<M> ReplyReceiver<M>
where
    M: Mail,
{
    pub async fn receive_reply(self) -> M::Result {
        self.reply_receiver
            .await
            .expect("The mail reply sender is never dropped")
    }
}

pub trait GenericHandler<A> {
    fn handle(&mut self, actor: &mut A);
}

impl<A, M> GenericHandler<A> for ReplyMail<M>
where
    A: MailHandler<M> + Send,
    M: Mail + Send,
    <M as Mail>::Result: Send,
{
    fn handle(&mut self, actor: &mut A) {
        let result =
            <A as MailHandler<M>>::handle(actor, self.mail.take().expect("Must have a message"));
        self.reply_sender
            .take()
            .expect("Must have a sender")
            .send(result);
    }
}

pub struct ActorAddress<A> {
    mail_sender: MpscSender<Box<dyn GenericHandler<A> + Send>>,
}

impl<A> Clone for ActorAddress<A> {
    fn clone(&self) -> Self {
        Self {
            mail_sender: self.mail_sender.clone(),
        }
    }
}

impl<A> ActorAddress<A> {
    pub fn is_closed(&self) -> bool {
        self.mail_sender.is_closed()
    }

    pub fn send_actor_mail<M>(&self, mail: M) -> DdsResult<ReplyReceiver<M>>
    where
        A: MailHandler<M> + Send,
        M: Mail + Send + 'static,
        M::Result: Send,
    {
        let (reply_sender, reply_receiver) = oneshot();
        self.mail_sender
            .send(Box::new(ReplyMail {
                mail: Some(mail),
                reply_sender: Some(reply_sender),
            }))
            .map_err(|_| DdsError::AlreadyDeleted)?;
        Ok(ReplyReceiver { reply_receiver })
    }
}

pub struct Actor<A> {
    mail_sender: MpscSender<Box<dyn GenericHandler<A> + Send>>,
    // join_handle: tokio::task::JoinHandle<()>,
}

impl<A> Actor<A>
where
    A: Send + 'static,
{
    pub fn spawn(mut actor: A, runtime: &ExecutorHandle) -> Self {
        let (mail_sender, mailbox_recv) = mpsc_channel::<Box<dyn GenericHandler<A> + Send>>();

        runtime.spawn(async move {
            while let Some(mut m) = mailbox_recv.recv().await {
                m.handle(&mut actor);
            }
        });
        Actor { mail_sender }
    }

    pub fn address(&self) -> ActorAddress<A> {
        ActorAddress {
            mail_sender: self.mail_sender.clone(),
        }
    }

    pub async fn stop(self) {
        self.mail_sender.close();
    }

    pub fn send_actor_mail<M>(&self, mail: M) -> ReplyReceiver<M>
    where
        A: MailHandler<M>,
        M: Mail + Send + 'static,
        M::Result: Send,
    {
        let (reply_sender, reply_receiver) = oneshot();
        self.mail_sender
            .send(Box::new(ReplyMail {
                mail: Some(mail),
                reply_sender: Some(reply_sender),
            }))
            .expect("Message will always be sent when actor exists");
        ReplyReceiver { reply_receiver }
    }
}

#[cfg(test)]
mod tests {
    use crate::implementation::runtime::executor::{block_on, Executor};

    use super::*;

    pub struct MyActor {
        data: u8,
    }

    pub struct Increment {
        value: u8,
    }
    impl Mail for Increment {
        type Result = u8;
    }
    impl MailHandler<Increment> for MyActor {
        fn handle(&mut self, message: Increment) -> <Increment as Mail>::Result {
            self.data += message.value;
            self.data
        }
    }

    #[test]
    fn actor_increment() {
        let executor = Executor::new();
        let my_data = MyActor { data: 0 };
        let actor = Actor::spawn(my_data, &executor.handle());

        assert_eq!(
            block_on(async {
                actor
                    .send_actor_mail(Increment { value: 10 })
                    .receive_reply()
                    .await
            }),
            10
        )
    }

    #[test]
    fn actor_stop_should_not_block() {
        let executor = Executor::new();
        let my_data = MyActor { data: 0 };
        let actor = Actor::spawn(my_data, &executor.handle());

        assert_eq!(
            block_on(async {
                actor
                    .send_actor_mail(Increment { value: 10 })
                    .receive_reply()
                    .await
            }),
            10
        );

        block_on(actor.stop());
    }

    #[test]
    fn actor_send_message_after_stop_should_return_error() {
        let executor = Executor::new();
        let my_data = MyActor { data: 0 };
        let actor = Actor::spawn(my_data, &executor.handle());
        let actor_address = actor.address();

        assert_eq!(
            block_on(async {
                actor
                    .send_actor_mail(Increment { value: 10 })
                    .receive_reply()
                    .await
            }),
            10
        );

        block_on(actor.stop());

        assert!(actor_address
            .send_actor_mail(Increment { value: 10 })
            .is_err());
    }
}
