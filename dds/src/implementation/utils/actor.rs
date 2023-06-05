use std::{future::poll_fn, task::Poll};

use tokio::{runtime, sync};

use lazy_static::lazy_static;

use crate::infrastructure::error::{DdsError, DdsResult};

lazy_static! {
    pub static ref THE_RUNTIME: runtime::Runtime = runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime");
}

pub trait Mail {
    type Result;
}

pub trait MailHandler<M>
where
    M: Mail,
    Self: Sized,
{
    fn handle(&mut self, mail: M) -> M::Result;
}

#[derive(Debug)]
pub struct ActorAddress<A> {
    sender: sync::mpsc::Sender<Box<dyn GenericHandler<A> + Send>>,
}

impl<A> Clone for ActorAddress<A> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<A> ActorAddress<A> {
    pub fn send_blocking<M>(&self, mail: M) -> DdsResult<M::Result>
    where
        A: MailHandler<M>,
        M: Mail + Send + 'static,
        M::Result: Send,
    {
        let (response_sender, response_receiver) = sync::oneshot::channel();

        self.sender
            .blocking_send(Box::new(SyncMail::new(mail, response_sender)))
            .map_err(|_| DdsError::AlreadyDeleted)?;
        response_receiver
            .blocking_recv()
            .map_err(|_| DdsError::AlreadyDeleted)
    }
}

trait GenericHandler<A> {
    fn handle(&mut self, actor: &mut A) -> Result<(), ()>;
}

struct SyncMail<M>
where
    M: Mail,
{
    // Both fields have to be inside an option because later on the contents
    // have to be moved out and the struct. Because the struct is passed as a Boxed
    // trait object this is only feasible by using the Option fields.
    mail: Option<M>,
    sender: Option<sync::oneshot::Sender<M::Result>>,
}

impl<A, M> GenericHandler<A> for SyncMail<M>
where
    A: MailHandler<M>,
    M: Mail,
{
    fn handle(&mut self, actor: &mut A) -> Result<(), ()> {
        let result = <A as MailHandler<M>>::handle(
            actor,
            self.mail
                .take()
                .expect("Mail should be processed only once"),
        );
        self.sender
            .take()
            .expect("Mail should be processed only once")
            .send(result)
            .map_err(|_| ())
    }
}

pub struct Actor<A> {
    address: ActorAddress<A>,
    join_handle: tokio::task::JoinHandle<()>,
}

impl<A> Actor<A> {
    pub fn address(&self) -> ActorAddress<A> {
        self.address.clone()
    }
}

impl<A> Drop for Actor<A> {
    fn drop(&mut self) {
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
    let (sender, mailbox) = sync::mpsc::channel(10);

    let address = ActorAddress { sender };

    let mut actor_obj = SpawnedActor {
        value: actor,
        mailbox,
    };

    let join_handle = THE_RUNTIME.spawn(poll_fn(move |cx| {
        while let Poll::Ready(val) = actor_obj.mailbox.poll_recv(cx) {
            if let Some(mut m) = val {
                // Handling mail can be synchronous and allowed to call blocking code
                // (e.g. send mail to other actors). It is not allowed to be an async function
                // otherwise multiple mails could interrupt each other and modify the object in between.
                // To allow calling blocking code inside the block_in_place method is used to prevent
                // the runtime from crashing
                tokio::task::block_in_place(|| m.handle(&mut actor_obj.value).ok());
            }
        }

        Poll::Pending
    }));

    Actor {
        address,
        join_handle,
    }
}

impl<M> SyncMail<M>
where
    M: Mail,
{
    fn new(message: M, sender: sync::oneshot::Sender<M::Result>) -> Self {
        Self {
            mail: Some(message),
            sender: Some(sender),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub struct MyData {
        data: u8,
    }

    impl MyData {
        fn increment(&mut self, value: u8) -> u8 {
            self.data += value;
            self.data
        }

        fn decrement(&mut self) {
            self.data -= 1;
        }
    }

    pub struct IncrementMail {
        pub value: u8,
    }

    impl Mail for IncrementMail {
        type Result = u8;
    }

    pub struct DecrementMail;

    impl Mail for DecrementMail {
        type Result = ();
    }

    impl MailHandler<IncrementMail> for MyData {
        fn handle(&mut self, mail: IncrementMail) -> <IncrementMail as Mail>::Result {
            self.increment(mail.value)
        }
    }

    impl MailHandler<DecrementMail> for MyData {
        fn handle(&mut self, _: DecrementMail) -> <DecrementMail as Mail>::Result {
            self.decrement()
        }
    }

    pub struct DataInterface(ActorAddress<MyData>);

    impl DataInterface {
        pub fn increment(&self, value: u8) -> DdsResult<u8> {
            self.0.send_blocking(IncrementMail { value })
        }
    }

    #[test]
    fn actor_increment() {
        let my_data = MyData { data: 0 };
        let actor = spawn_actor(my_data);
        let data_interface = DataInterface(actor.address());
        assert_eq!(data_interface.increment(10).unwrap(), 10)
    }

    #[test]
    fn actor_already_deleted() {
        let my_data = MyData { data: 0 };
        let actor = spawn_actor(my_data);
        let data_interface = DataInterface(actor.address());
        std::mem::drop(actor);
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert_eq!(data_interface.increment(10), Err(DdsError::AlreadyDeleted))
    }
}
