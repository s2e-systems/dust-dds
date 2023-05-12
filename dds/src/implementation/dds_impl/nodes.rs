use crate::{implementation::rtps::types::Guid, subscription::data_reader::AnyDataReader};

use super::{
    node_user_defined_data_writer::UserDefinedDataWriterNode,
    node_user_defined_topic::UserDefinedTopicNode,
};

#[derive(PartialEq, Eq, Debug)]
pub enum SubscriberNodeKind {
    Builtin(SubscriberNode),
    UserDefined(SubscriberNode),
    Listener(SubscriberNode),
}

#[derive(PartialEq, Eq, Debug)]
pub enum DataWriterNodeKind {
    UserDefined(UserDefinedDataWriterNode),
    Listener(UserDefinedDataWriterNode),
}

#[derive(PartialEq, Eq, Debug)]
pub enum DataReaderNodeKind {
    BuiltinStateful(DataReaderNode),
    BuiltinStateless(DataReaderNode),
    UserDefined(DataReaderNode),
    Listener(DataReaderNode),
}

#[derive(PartialEq, Eq, Debug)]
pub enum TopicNodeKind {
    UserDefined(UserDefinedTopicNode),
    Listener(UserDefinedTopicNode),
}

#[derive(PartialEq, Eq, Debug)]
pub struct SubscriberNode {
    this: Guid,
    parent: Guid,
}

impl SubscriberNode {
    pub fn new(this: Guid, parent: Guid) -> Self {
        Self { this, parent }
    }

    pub fn guid(&self) -> Guid {
        self.this
    }

    pub fn parent_participant(&self) -> Guid {
        self.parent
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct DataReaderNode {
    this: Guid,
    parent_subcriber: Guid,
    parent_participant: Guid,
}

impl DataReaderNode {
    pub fn new(this: Guid, parent_subcriber: Guid, parent_participant: Guid) -> Self {
        Self {
            this,
            parent_subcriber,
            parent_participant,
        }
    }

    pub fn guid(&self) -> Guid {
        self.this
    }

    pub fn parent_subscriber(&self) -> Guid {
        self.parent_subcriber
    }

    pub fn parent_participant(&self) -> Guid {
        self.parent_participant
    }
}

impl AnyDataReader for DataReaderNode {}
