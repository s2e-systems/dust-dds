use std::sync::Arc;

use crate::infrastructure::error::{DdsError, DdsResult};

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
    mail: M,
    reply_sender: tokio::sync::oneshot::Sender<M::Result>,
}

pub struct ReplyReceiver<M>
where
    M: Mail,
{
    reply_receiver: tokio::sync::oneshot::Receiver<M::Result>,
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
    fn handle(self: Box<Self>, actor: &mut A);
}

impl<A, M> GenericHandler<A> for ReplyMail<M>
where
    A: MailHandler<M> + Send,
    M: Mail + Send,
    <M as Mail>::Result: Send,
{
    fn handle(self: Box<Self>, actor: &mut A) {
        let this = *self;
        let result = <A as MailHandler<M>>::handle(actor, this.mail);
        this.reply_sender.send(result).ok();
    }
}

#[derive(Debug)]
pub struct ActorAddress<A> {
    mail_sender: tokio::sync::mpsc::UnboundedSender<Box<dyn GenericHandler<A> + Send>>,
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
        let (reply_sender, reply_receiver) = tokio::sync::oneshot::channel();
        self.mail_sender
            .send(Box::new(ReplyMail { mail, reply_sender }))
            .map_err(|_| DdsError::AlreadyDeleted)?;
        Ok(ReplyReceiver { reply_receiver })
    }
}

pub struct Actor<A> {
    mail_sender: tokio::sync::mpsc::UnboundedSender<Box<dyn GenericHandler<A> + Send>>,
    join_handle: tokio::task::JoinHandle<()>,
    notify_stop: Arc<tokio::sync::Notify>,
}

impl<A> Actor<A>
where
    A: Send + 'static,
{
    pub fn spawn(mut actor: A, runtime: &tokio::runtime::Handle) -> Self {
        let (mail_sender, mut mailbox_recv) =
            tokio::sync::mpsc::unbounded_channel::<Box<dyn GenericHandler<A> + Send>>();
        let notify_stop = Arc::new(tokio::sync::Notify::new());
        let notify_clone = notify_stop.clone();

        let join_handle = runtime.spawn(async move {
            loop {
                tokio::select! {
                    m = mailbox_recv.recv() => {
                        match m {
                            Some(message) => message.handle(&mut actor),
                            None => break,
                        }
                    }
                    _ = notify_clone.notified() => {
                        mailbox_recv.close();
                    }
                }
            }
        });
        Actor {
            mail_sender,
            join_handle,
            notify_stop,
        }
    }

    pub fn address(&self) -> ActorAddress<A> {
        ActorAddress {
            mail_sender: self.mail_sender.clone(),
        }
    }

    pub async fn stop(self) {
        self.notify_stop.notify_one();
        self.join_handle.await.ok();
    }

    pub fn send_actor_mail<M>(&self, mail: M) -> ReplyReceiver<M>
    where
        A: MailHandler<M>,
        M: Mail + Send + 'static,
        M::Result: Send,
    {
        let (reply_sender, reply_receiver) = tokio::sync::oneshot::channel();
        self.mail_sender
            .send(Box::new(ReplyMail { mail, reply_sender }))
            .expect("Message will always be sent when actor exists");
        ReplyReceiver { reply_receiver }
    }
}

#[cfg(test)]
mod tests {
    use tokio::runtime::Runtime;

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
        let runtime = Runtime::new().unwrap();
        let my_data = MyActor { data: 0 };
        let actor = Actor::spawn(my_data, runtime.handle());

        assert_eq!(
            runtime.block_on(async {
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
        let runtime = Runtime::new().unwrap();
        let my_data = MyActor { data: 0 };
        let actor = Actor::spawn(my_data, runtime.handle());

        assert_eq!(
            runtime.block_on(async {
                actor
                    .send_actor_mail(Increment { value: 10 })
                    .receive_reply()
                    .await
            }),
            10
        );

        runtime.block_on(actor.stop());
    }

    #[test]
    fn actor_send_message_after_stop_should_return_error() {
        let runtime = Runtime::new().unwrap();
        let my_data = MyActor { data: 0 };
        let actor = Actor::spawn(my_data, runtime.handle());
        let actor_address = actor.address();

        assert_eq!(
            runtime.block_on(async {
                actor
                    .send_actor_mail(Increment { value: 10 })
                    .receive_reply()
                    .await
            }),
            10
        );

        runtime.block_on(actor.stop());

        assert!(actor_address
            .send_actor_mail(Increment { value: 10 })
            .is_err());
    }
}
