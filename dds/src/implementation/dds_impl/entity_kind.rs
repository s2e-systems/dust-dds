use super::{
    builtin_data_reader::BuiltinDataReader, builtin_subscriber::BuiltinSubscriberNode,
    listener_data_reader::ListenerDataReader, listener_data_writer::ListenerDataWriter,
    listener_subscriber::ListenerSubscriberNode, user_defined_data_reader::UserDefinedDataReader,
    user_defined_data_writer::UserDefinedDataWriter,
    user_defined_subscriber::UserDefinedSubscriberNode,
};

#[derive(PartialEq, Debug)]
pub enum SubscriberNodeKind {
    Builtin(BuiltinSubscriberNode),
    UserDefined(UserDefinedSubscriberNode),
    Listener(ListenerSubscriberNode),
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
