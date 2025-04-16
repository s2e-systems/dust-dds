use super::{
    executor::ExecutorHandle,
    mpsc::{mpsc_channel, MpscReceiver, MpscSender},
};
use crate::infrastructure::error::{DdsError, DdsResult};

pub trait Mail {
    type Result;
}

pub trait MailHandler<M> {
    fn handle(&mut self, message: M);
}

struct ReplyMail<M>
where
    M: Mail,
{
    mail: Option<M>,
}

pub trait GenericHandler<A> {
    fn handle(&mut self, actor: &mut A);
}

impl<A, M> GenericHandler<A> for ReplyMail<M>
where
    A: MailHandler<M> + Send,
    M: Mail + Send,
{
    fn handle(&mut self, actor: &mut A) {
        <A as MailHandler<M>>::handle(actor, self.mail.take().expect("Must have a message"));
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
    pub fn send_actor_mail<M>(&self, mail: M) -> DdsResult<()>
    where
        A: MailHandler<M> + Send,
        M: Mail + Send + 'static,
        M::Result: Send,
    {
        self.mail_sender
            .send(Box::new(ReplyMail { mail: Some(mail) }))
            .map_err(|_| DdsError::AlreadyDeleted)
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

    pub fn send_actor_mail<M>(&self, mail: M)
    where
        A: MailHandler<M>,
        M: Mail + Send + 'static,
        M::Result: Send,
    {
        self.mail_sender
            .send(Box::new(ReplyMail { mail: Some(mail) }))
            .expect("Message will always be sent when actor exists");
    }
}

pub struct ActorBuilder<A> {
    mail_sender: MpscSender<Box<dyn GenericHandler<A> + Send>>,
    mailbox_recv: MpscReceiver<Box<dyn GenericHandler<A> + Send>>,
}

impl<A> ActorBuilder<A>
where
    A: Send + 'static,
{
    pub fn new() -> Self {
        let (mail_sender, mailbox_recv) = mpsc_channel::<Box<dyn GenericHandler<A> + Send>>();

        Self {
            mail_sender,
            mailbox_recv,
        }
    }

    pub fn address(&self) -> ActorAddress<A> {
        ActorAddress {
            mail_sender: self.mail_sender.clone(),
        }
    }

    pub fn build(self, mut actor: A, runtime: &ExecutorHandle) -> Actor<A> {
        let mailbox_recv = self.mailbox_recv;
        runtime.spawn(async move {
            while let Some(mut m) = mailbox_recv.recv().await {
                m.handle(&mut actor);
            }
        });
        Actor {
            mail_sender: self.mail_sender,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::runtime::{
        executor::{block_on, Executor},
        oneshot::{oneshot, OneshotSender},
    };

    use super::*;

    pub struct MyActor {
        data: u8,
    }

    pub struct Increment {
        value: u8,
        result_sender: OneshotSender<u8>,
    }
    impl Mail for Increment {
        type Result = u8;
    }
    impl MailHandler<Increment> for MyActor {
        fn handle(&mut self, message: Increment) {
            self.data += message.value;
            message.result_sender.send(self.data);
        }
    }

    #[test]
    fn actor_increment() {
        let executor = Executor::new();
        let my_data = MyActor { data: 0 };
        let actor = Actor::spawn(my_data, &executor.handle());
        let (result_sender, result_receiver) = oneshot();

        assert_eq!(
            block_on(async {
                actor.send_actor_mail(Increment {
                    value: 10,
                    result_sender,
                });
                result_receiver.await.unwrap()
            }),
            10
        )
    }

    #[test]
    fn actor_stop_should_not_block() {
        let executor = Executor::new();
        let my_data = MyActor { data: 0 };
        let actor = Actor::spawn(my_data, &executor.handle());

        let (result_sender, result_receiver) = oneshot();

        assert_eq!(
            block_on(async {
                actor.send_actor_mail(Increment {
                    value: 10,
                    result_sender,
                });
                result_receiver.await.unwrap()
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

        let (result_sender, result_receiver) = oneshot();
        assert_eq!(
            block_on(async {
                actor.send_actor_mail(Increment {
                    value: 10,
                    result_sender,
                });
                result_receiver.await.unwrap()
            }),
            10
        );

        block_on(actor.stop());
        let (result_sender, _) = oneshot();

        assert!(actor_address
            .send_actor_mail(Increment {
                value: 10,
                result_sender
            })
            .is_err());
    }
}
