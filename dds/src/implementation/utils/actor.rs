use std::sync::{
    atomic::{self, AtomicBool},
    Arc,
};

use lazy_static::lazy_static;

use crate::infrastructure::error::{DdsError, DdsResult};

lazy_static! {
    pub static ref THE_RUNTIME: tokio::runtime::Runtime =
        tokio::runtime::Builder::new_multi_thread()
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

pub trait CommandHandler<M> {
    fn handle(&mut self, mail: M);
}

#[derive(Debug)]
pub struct ActorAddress<A> {
    sender: tokio::sync::mpsc::UnboundedSender<Box<dyn GenericHandler<A> + Send>>,
}

impl<A> Clone for ActorAddress<A> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<A> PartialEq for ActorAddress<A> {
    fn eq(&self, other: &Self) -> bool {
        self.sender.same_channel(&other.sender)
    }
}

impl<A> Eq for ActorAddress<A> {}

impl<A> ActorAddress<A> {
    pub fn send_blocking<M>(&self, mail: M) -> DdsResult<M::Result>
    where
        A: MailHandler<M>,
        M: Mail + Send + 'static,
        M::Result: Send,
    {
        let (response_sender, response_receiver) = tokio::sync::oneshot::channel();

        self.sender
            .send(Box::new(SyncMail::new(mail, response_sender)))
            .map_err(|_| DdsError::AlreadyDeleted)?;
        response_receiver
            .blocking_recv()
            .map_err(|_| DdsError::AlreadyDeleted)
    }

    pub fn send_command<M>(&self, command: M) -> DdsResult<()>
    where
        A: CommandHandler<M>,
        M: Send + 'static,
    {
        self.sender
            .send(Box::new(CommandMail::new(command)))
            .map_err(|_| DdsError::AlreadyDeleted)
    }
}

trait GenericHandler<A> {
    fn handle(&mut self, actor: &mut A);
}

struct SyncMail<M>
where
    M: Mail,
{
    // Both fields have to be inside an option because later on the contents
    // have to be moved out and the struct. Because the struct is passed as a Boxed
    // trait object this is only feasible by using the Option fields.
    mail: Option<M>,
    sender: Option<tokio::sync::oneshot::Sender<M::Result>>,
}

impl<M> SyncMail<M>
where
    M: Mail,
{
    fn new(message: M, sender: tokio::sync::oneshot::Sender<M::Result>) -> Self {
        Self {
            mail: Some(message),
            sender: Some(sender),
        }
    }
}

impl<A, M> GenericHandler<A> for SyncMail<M>
where
    A: MailHandler<M>,
    M: Mail,
{
    fn handle(&mut self, actor: &mut A) {
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
            .map_err(|_| "Failed to send message on type withou Debug")
            .expect("Sending should never fail");
    }
}

struct CommandMail<M> {
    mail: Option<M>,
}

impl<M> CommandMail<M> {
    fn new(mail: M) -> Self {
        Self { mail: Some(mail) }
    }
}

impl<A, M> GenericHandler<A> for CommandMail<M>
where
    A: CommandHandler<M>,
{
    fn handle(&mut self, actor: &mut A) {
        <A as CommandHandler<M>>::handle(
            actor,
            self.mail
                .take()
                .expect("Mail should be processed only once"),
        );
    }
}

pub struct Actor<A> {
    address: ActorAddress<A>,
    join_handle: tokio::task::JoinHandle<()>,
    cancellation_token: Arc<AtomicBool>,
}

impl<A> Actor<A> {
    pub fn address(&self) -> &ActorAddress<A> {
        &self.address
    }
}

impl<A> Drop for Actor<A> {
    fn drop(&mut self) {
        self.cancellation_token
            .store(true, atomic::Ordering::Release);
        self.join_handle.abort();
    }
}

struct SpawnedActor<A> {
    value: A,
    mailbox: tokio::sync::mpsc::UnboundedReceiver<Box<dyn GenericHandler<A> + Send>>,
}

pub fn spawn_actor<A>(actor: A) -> Actor<A>
where
    A: Send + 'static,
{
    let (sender, mailbox) = tokio::sync::mpsc::unbounded_channel();

    let address = ActorAddress { sender };

    let mut actor_obj = SpawnedActor {
        value: actor,
        mailbox,
    };
    let cancellation_token = Arc::new(AtomicBool::new(false));
    let cancellation_token_cloned = cancellation_token.clone();

    let join_handle = THE_RUNTIME.spawn(async move {
        while let Some(mut m) = actor_obj.mailbox.recv().await {
            if !cancellation_token_cloned.load(atomic::Ordering::Acquire) {
                m.handle(&mut actor_obj.value);
            } else {
                break;
            }
        }
    });

    Actor {
        address,
        join_handle,
        cancellation_token,
    }
}

// Macro to create both a function for the method and the equivalent wrapper for the actor
macro_rules! mailbox_function {
    // Match a function definition with return type
    ($type_name:ident, pub fn $fn_name:ident(&$($self_:ident)+ $(, $arg_name:ident:$arg_type:ty)* $(,)?) -> $ret_type:ty $body:block) => {
        impl $type_name {
            #[allow(clippy::too_many_arguments)]
            pub fn $fn_name(&$($self_)+ $(, $arg_name:$arg_type)*) -> $ret_type{
                $body
            }
        }

        impl crate::implementation::utils::actor::ActorAddress<$type_name> {
            #[allow(clippy::too_many_arguments)]
            pub fn $fn_name(&self $(, $arg_name:$arg_type)*) -> crate::infrastructure::error::DdsResult<$ret_type> {
                #[allow(non_camel_case_types)]
                struct $fn_name {
                    $($arg_name:$arg_type,)*
                }

                impl crate::implementation::utils::actor::Mail for $fn_name {
                    type Result = $ret_type;
                }

                impl crate::implementation::utils::actor::MailHandler<$fn_name> for $type_name {
                    #[allow(unused_variables)]
                    fn handle(&mut self, mail: $fn_name) -> $ret_type {
                        self.$fn_name($(mail.$arg_name,)*)
                    }
                }

                self.send_blocking($fn_name{
                    $($arg_name, )*
                })

            }
        }

    };

    // Match a function definition without return type
    ($type_name:ident, pub fn $fn_name:ident(&$($self_:ident)+ $(, $arg_name:ident:$arg_type:ty)* $(,)?) $body:block ) => {
        impl $type_name {
            #[allow(clippy::too_many_arguments)]
            pub fn $fn_name(&$($self_)+ $(, $arg_name:$arg_type)* ) {
                $body
            }
        }

        impl crate::implementation::utils::actor::ActorAddress<$type_name> {
            #[allow(clippy::too_many_arguments)]
            pub fn $fn_name(&self $(, $arg_name:$arg_type)*) -> crate::infrastructure::error::DdsResult<()> {
                #[allow(non_camel_case_types)]
                struct $fn_name {
                    $($arg_name:$arg_type,)*
                }

                impl crate::implementation::utils::actor::Mail for $fn_name {
                    type Result = ();
                }

                impl crate::implementation::utils::actor::MailHandler<$fn_name> for $type_name {
                    #[allow(unused_variables)]
                    fn handle(&mut self, mail: $fn_name) {
                        self.$fn_name($(mail.$arg_name,)*)
                    }
                }

                self.send_blocking($fn_name{
                    $($arg_name, )*
                })

            }
        }
    };
}
pub(crate) use mailbox_function;

// This macro should wrap an impl block and create the actor address wrapper methods with exactly the same interface
// It is kept around the "impl" block because otherwise there is no way to find the type name it refers to ($type_name)
macro_rules! actor_mailbox_interface {
    (impl $type_name:ident {
        $(
        pub fn $fn_name:ident(&$($self_:ident)+ $(, $arg_name:ident:$arg_type:ty)* $(,)?) $(-> $ret_type:ty)?
            $body:block
        )+
    }) => {
        $(crate::implementation::utils::actor::mailbox_function!($type_name, pub fn $fn_name(&$($self_)+ $(, $arg_name:$arg_type)*) $(-> $ret_type)? $body );)+
    };
}
pub(crate) use actor_mailbox_interface;

macro_rules! command_function {
    // Commands are only valid for functions without return
    ($type_name:ident, pub fn $fn_name:ident(&$($self_:ident)+ $(, $arg_name:ident:$arg_type:ty)* $(,)?) $body:block ) => {
        impl $type_name {
            pub fn $fn_name(&$($self_)+ $(, $arg_name:$arg_type)* ) {
                $body
            }
        }

        impl crate::implementation::utils::actor::ActorAddress<$type_name> {
            pub fn $fn_name(&self $(, $arg_name:$arg_type)*) -> crate::infrastructure::error::DdsResult<()> {
                #[allow(non_camel_case_types)]
                struct $fn_name {
                    $($arg_name:$arg_type,)*
                }

                impl crate::implementation::utils::actor::CommandHandler<$fn_name> for $type_name {
                    #[allow(unused_variables)]
                    fn handle(&mut self, mail: $fn_name) {
                        self.$fn_name($(mail.$arg_name,)*)
                    }
                }

                self.send_command($fn_name{
                    $($arg_name, )*
                })

            }
        }
    };
}
pub(crate) use command_function;

macro_rules! actor_command_interface {
    (impl $type_name:ident {
        $(
        pub fn $fn_name:ident(&$($self_:ident)+ $(, $arg_name:ident:$arg_type:ty)* $(,)?)
            $body:block
        )+
    }) => {
        $(crate::implementation::utils::actor::command_function!($type_name, pub fn $fn_name(&$($self_)+ $(, $arg_name:$arg_type)*) $body );)+
    };
}
pub(crate) use actor_command_interface;

#[cfg(test)]
mod tests {
    use super::*;

    pub struct MyData {
        data: u8,
    }
    actor_mailbox_interface!(
    impl MyData {
        pub fn increment(&mut self, value: u8) -> u8 {
            self.data += value;
            self.data
        }

        pub fn decrement(&mut self) {
            self.data -= 1;
        }

        pub fn try_increment(&mut self) -> DdsResult<()> {
            self.data -= 1;
            Ok(())
        }
    }
    );

    pub struct DataInterface(ActorAddress<MyData>);

    impl DataInterface {
        pub fn increment(&self, value: u8) -> DdsResult<u8> {
            self.0.increment(value)
        }
    }

    #[test]
    fn actor_increment() {
        let my_data = MyData { data: 0 };
        let actor = spawn_actor(my_data);
        let data_interface = DataInterface(actor.address().clone());
        assert_eq!(data_interface.increment(10).unwrap(), 10)
    }

    #[test]
    fn actor_already_deleted() {
        let my_data = MyData { data: 0 };
        let actor = spawn_actor(my_data);
        let data_interface = DataInterface(actor.address().clone());
        std::mem::drop(actor);
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert_eq!(data_interface.increment(10), Err(DdsError::AlreadyDeleted));
    }
}
