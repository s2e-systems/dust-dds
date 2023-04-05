use super::{
    builtin_data_reader::BuiltinDataReaderNode, builtin_subscriber::BuiltinSubscriberNode,
    listener_data_reader::ListenerDataReaderNode, listener_data_writer::ListenerDataWriterNode,
    listener_subscriber::ListenerSubscriberNode, user_defined_data_reader::UserDefinedDataReaderNode,
    user_defined_data_writer::UserDefinedDataWriterNode,
    user_defined_subscriber::UserDefinedSubscriberNode,
};

#[derive(PartialEq, Debug)]
pub enum SubscriberNodeKind {
    Builtin(BuiltinSubscriberNode),
    UserDefined(UserDefinedSubscriberNode),
    Listener(ListenerSubscriberNode),
}

#[derive(PartialEq, Debug)]
pub enum DataWriterNodeKind {
    UserDefined(UserDefinedDataWriterNode),
    Listener(ListenerDataWriterNode),
}

#[derive(PartialEq, Debug)]
pub enum DataReaderKindNode {
    Builtin(BuiltinDataReaderNode),
    UserDefined(UserDefinedDataReaderNode),
    Listener(ListenerDataReaderNode),
}
