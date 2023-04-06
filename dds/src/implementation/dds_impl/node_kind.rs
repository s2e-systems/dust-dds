use super::{
    listener_subscriber::ListenerSubscriberNode,
    node_builtin_data_reader_stateful::BuiltinDataReaderStatefulNode,
    node_builtin_data_reader_stateless::BuiltinDataReaderStatelessNode,
    node_builtin_subscriber::BuiltinSubscriberNode,
    node_listener_data_reader::ListenerDataReaderNode,
    node_listener_data_writer::ListenerDataWriterNode,
    node_user_defined_data_reader::UserDefinedDataReaderNode,
    node_user_defined_data_writer::UserDefinedDataWriterNode,
    node_user_defined_subscriber::UserDefinedSubscriberNode,
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
pub enum DataReaderNodeKind {
    BuiltinStateful(BuiltinDataReaderStatefulNode),
    BuiltinStateless(BuiltinDataReaderStatelessNode),
    UserDefined(UserDefinedDataReaderNode),
    Listener(ListenerDataReaderNode),
}
