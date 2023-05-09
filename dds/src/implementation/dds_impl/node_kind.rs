use super::{
    node_builtin_data_reader_stateful::BuiltinDataReaderStatefulNode,
    node_builtin_data_reader_stateless::BuiltinDataReaderStatelessNode,
    node_builtin_subscriber::BuiltinSubscriberNode,
    node_listener_data_writer::ListenerDataWriterNode,
    node_listener_subscriber::ListenerSubscriberNode, node_listener_topic::ListenerTopicNode,
    node_user_defined_data_reader::UserDefinedDataReaderNode,
    node_user_defined_data_writer::UserDefinedDataWriterNode,
    node_user_defined_subscriber::UserDefinedSubscriberNode,
    node_user_defined_topic::UserDefinedTopicNode,
};

#[derive(PartialEq, Eq, Debug)]
pub enum SubscriberNodeKind {
    Builtin(BuiltinSubscriberNode),
    UserDefined(UserDefinedSubscriberNode),
    Listener(ListenerSubscriberNode),
}

#[derive(PartialEq, Eq, Debug)]
pub enum DataWriterNodeKind {
    UserDefined(UserDefinedDataWriterNode),
    Listener(ListenerDataWriterNode),
}

#[derive(PartialEq, Debug)]
pub enum DataReaderNodeKind {
    BuiltinStateful(BuiltinDataReaderStatefulNode),
    BuiltinStateless(BuiltinDataReaderStatelessNode),
    UserDefined(UserDefinedDataReaderNode),
    Listener(UserDefinedDataReaderNode),
}

#[derive(PartialEq, Eq, Debug)]
pub enum TopicNodeKind {
    UserDefined(UserDefinedTopicNode),
    Listener(ListenerTopicNode),
}
