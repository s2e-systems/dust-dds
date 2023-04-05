use super::{
    builtin_subscriber::BuiltinSubscriber, listener_data_writer::ListenerDataWriter,
    listener_subscriber::ListenerSubscriber, user_defined_data_writer::UserDefinedDataWriter,
    user_defined_subscriber::UserDefinedSubscriber,
};

#[derive(PartialEq, Debug)]
pub enum SubscriberKind {
    BuiltIn(BuiltinSubscriber),
    UserDefined(UserDefinedSubscriber),
    Listener(ListenerSubscriber),
}

#[derive(PartialEq, Debug)]
pub enum DataWriterKind {
    UserDefined(UserDefinedDataWriter),
    Listener(ListenerDataWriter),
}
