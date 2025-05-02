use super::{
    executor::ExecutorHandle,
    mpsc::{mpsc_channel, MpscReceiver, MpscSender},
};
use crate::infrastructure::error::{DdsError, DdsResult};

pub trait MailHandler {
    type Mail;

    fn handle(&mut self, message: Self::Mail);
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
    pub fn spawn(mut actor: A, runtime: &ExecutorHandle) -> Self {
        let (mail_sender, mailbox_recv) = mpsc_channel::<A::Mail>();

        runtime.spawn(async move {
            while let Some(m) = mailbox_recv.recv().await {
                actor.handle(m);
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

    pub fn send_actor_mail(&self, mail: A::Mail) {
        self.mail_sender
            .send(mail)
            .expect("Message will always be sent when actor exists");
    }
}

pub struct ActorBuilder<A>
where
    A: MailHandler,
{
    mail_sender: MpscSender<A::Mail>,
    mailbox_recv: MpscReceiver<A::Mail>,
}

impl<A> ActorBuilder<A>
where
    A: MailHandler + Send + 'static,
    A::Mail: Send,
{
    pub fn new() -> Self {
        let (mail_sender, mailbox_recv) = mpsc_channel::<A::Mail>();

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
            while let Some(m) = mailbox_recv.recv().await {
                actor.handle(m);
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
    impl MailHandler for MyActor {
        type Mail = Increment;
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
