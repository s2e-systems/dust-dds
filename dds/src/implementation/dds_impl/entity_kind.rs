use super::{
    builtin_data_reader::BuiltinDataReader, builtin_subscriber::BuiltinSubscriber,
    listener_data_reader::ListenerDataReader, listener_data_writer::ListenerDataWriter,
    listener_subscriber::ListenerSubscriber, user_defined_data_reader::UserDefinedDataReader,
    user_defined_data_writer::UserDefinedDataWriter,
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

#[derive(PartialEq, Debug)]
pub enum DataReaderKind {
    Builtin(BuiltinDataReader),
    UserDefined(UserDefinedDataReader),
    Listener(ListenerDataReader),
}
