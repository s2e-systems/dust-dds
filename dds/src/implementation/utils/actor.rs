use std::{future::poll_fn, marker::PhantomData, task::Poll};

use tokio::{runtime, sync};

use lazy_static::lazy_static;

use crate::infrastructure::error::{DdsError, DdsResult};

lazy_static! {
    pub static ref THE_RUNTIME: runtime::Runtime = runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime");
}

pub trait Message {
    type Result;
}

pub trait Handler<M>
where
    M: Message,
    Self: Sized,
{
    fn handle(&mut self, mail: M) -> M::Result;
}

#[derive(Debug)]
pub struct ActorAddress<A> {
    actor: PhantomData<A>,
    sender: sync::mpsc::Sender<Box<dyn GenericHandler<A> + Send>>,
}

impl<A> Clone for ActorAddress<A> {
    fn clone(&self) -> Self {
        Self {
            actor: PhantomData,
            sender: self.sender.clone(),
        }
    }
}

impl<A> ActorAddress<A> {
    pub fn send_blocking<M>(&self, mail: M) -> DdsResult<M::Result>
    where
        A: Handler<M>,
        M: Message + Send + 'static,
        M::Result: Send,
    {
        let (response_sender, response_receiver) = sync::oneshot::channel();

        self.sender
            .blocking_send(Box::new(SyncMessage::new(mail, response_sender)))
            .map_err(|_| DdsError::AlreadyDeleted)?;
        response_receiver
            .blocking_recv()
            .map_err(|_| DdsError::AlreadyDeleted)
    }

    pub async fn send<M>(&self, mail: M) -> DdsResult<M::Result>
    where
        A: Handler<M>,
        M: Message + Send + 'static,
        M::Result: Send,
    {
        let (response_sender, response_receiver) = sync::oneshot::channel();

        self.sender
            .send(Box::new(SyncMessage::new(mail, response_sender)))
            .await
            .map_err(|_| DdsError::AlreadyDeleted)?;
        response_receiver
            .await
            .map_err(|_| DdsError::AlreadyDeleted)
    }
}

pub struct ActorJoinHandle {
    join_handle: tokio::task::JoinHandle<()>,
}

impl Drop for ActorJoinHandle {
    fn drop(&mut self) {
        self.join_handle.abort();
    }
}

trait GenericHandler<A> {
    fn handle(&mut self, actor: &mut A) -> Result<(), ()>;
}

struct SyncMessage<M>
where
    M: Message,
{
    // Both fields have to be inside an option because later on the contents
    // have to be moved out and the struct. Because the struct is passed as a Boxed
    // trait object this is only feasible by using the Option fields.
    message: Option<M>,
    sender: Option<sync::oneshot::Sender<M::Result>>,
}

impl<A, M> GenericHandler<A> for SyncMessage<M>
where
    A: Handler<M>,
    M: Message,
{
    fn handle(&mut self, actor: &mut A) -> Result<(), ()> {
        let result = <A as Handler<M>>::handle(
            actor,
            self.message
                .take()
                .expect("Message should be processed only once"),
        );
        self.sender
            .take()
            .expect("Message should be processed only once")
            .send(result)
            .map_err(|_| ())
    }
}

struct SpawnedActor<A> {
    value: A,
    mailbox: tokio::sync::mpsc::Receiver<Box<dyn GenericHandler<A> + Send>>,
}

pub fn spawn_actor<A>(actor: A) -> (ActorAddress<A>, ActorJoinHandle)
where
    A: Send + 'static,
{
    let (sender, mailbox) = sync::mpsc::channel(10);

    let actor_address = ActorAddress {
        actor: PhantomData,
        sender,
    };
    let mut actor_obj = SpawnedActor {
        value: actor,
        mailbox,
    };

    let actor_handle = ActorJoinHandle {
        join_handle: THE_RUNTIME.spawn(poll_fn(move |cx| {
            while let Poll::Ready(val) = actor_obj.mailbox.poll_recv(cx) {
                if let Some(mut m) = val {
                    // Handling a message must be synchronous and allowed to call blocking code
                    // (e.g. send message to other actors). It is not allowed to be an async function
                    // otherwise multiple messages could interrupt each other and modify the object in between.
                    // To allow calling blocking code inside the block_in_place method is used to prevent
                    // the runtime from crashing
                    tokio::task::block_in_place(|| m.handle(&mut actor_obj.value).ok());
                }
            }

            Poll::Pending
        })),
    };

    (actor_address, actor_handle)
}

impl<M> SyncMessage<M>
where
    M: Message,
{
    fn new(message: M, sender: sync::oneshot::Sender<M::Result>) -> Self {
        Self {
            message: Some(message),
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

    pub struct IncrementMessage {
        pub value: u8,
    }

    impl Message for IncrementMessage {
        type Result = u8;
    }

    pub struct DecrementMessage;

    impl Message for DecrementMessage {
        type Result = ();
    }

    impl Handler<IncrementMessage> for MyData {
        fn handle(&mut self, message: IncrementMessage) -> <IncrementMessage as Message>::Result {
            self.increment(message.value)
        }
    }

    impl Handler<DecrementMessage> for MyData {
        fn handle(&mut self, _message: DecrementMessage) -> <DecrementMessage as Message>::Result {
            self.decrement()
        }
    }

    pub struct DataInterface(ActorAddress<MyData>);

    impl DataInterface {
        pub fn increment(&self, value: u8) -> DdsResult<u8> {
            self.0.send_blocking(IncrementMessage { value })
        }
    }

    #[test]
    fn actor_increment() {
        let my_data = MyData { data: 0 };
        let (address, _join_handle) = spawn_actor(my_data);
        let data_interface = DataInterface(address);
        assert_eq!(data_interface.increment(10).unwrap(), 10)
    }

    #[test]
    fn actor_already_deleted() {
        let my_data = MyData { data: 0 };
        let (address, handle) = spawn_actor(my_data);
        std::mem::drop(handle);
        std::thread::sleep(std::time::Duration::from_millis(100));
        let data_interface = DataInterface(address);
        assert_eq!(data_interface.increment(10), Err(DdsError::AlreadyDeleted))
    }
}
