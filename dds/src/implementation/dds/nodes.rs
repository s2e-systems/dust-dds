use crate::{
    implementation::{rtps::types::Guid, utils::actor::ActorAddress},
    publication::data_writer::AnyDataWriter,
    subscription::data_reader::AnyDataReader,
    topic_definition::topic::AnyTopic,
};

use super::{dds_domain_participant::DdsDomainParticipant, dds_topic::DdsTopic};

#[derive(PartialEq, Eq, Debug)]
pub enum SubscriberNodeKind {
    Builtin(SubscriberNode),
    UserDefined(SubscriberNode),
    Listener(SubscriberNode),
}

#[derive(PartialEq, Eq, Debug)]
pub enum DataWriterNodeKind {
    UserDefined(DataWriterNode),
    Listener(DataWriterNode),
}

#[derive(PartialEq, Eq, Debug)]
pub enum DataReaderNodeKind {
    BuiltinStateful(DataReaderNode),
    BuiltinStateless(DataReaderNode),
    UserDefined(DataReaderNode),
    Listener(DataReaderNode),
}

pub enum TopicNodeKind {
    UserDefined(TopicNode),
    Listener(TopicNode),
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

#[derive(Clone)]
pub struct TopicNode {
    this: ActorAddress<DdsTopic>,
    parent: ActorAddress<DdsDomainParticipant>,
}

impl TopicNode {
    pub fn new(this: ActorAddress<DdsTopic>, parent: ActorAddress<DdsDomainParticipant>) -> Self {
        Self { this, parent }
    }

    pub fn address(&self) -> &ActorAddress<DdsTopic> {
        &self.this
    }

    pub fn parent_participant(&self) -> &ActorAddress<DdsDomainParticipant> {
        &self.parent
    }
}

impl AnyTopic for TopicNode {}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub struct DataWriterNode {
    this: Guid,
    parent_publisher: Guid,
    parent_participant: Guid,
}

impl DataWriterNode {
    pub fn new(this: Guid, parent_publisher: Guid, parent_participant: Guid) -> Self {
        Self {
            this,
            parent_publisher,
            parent_participant,
        }
    }

    pub fn guid(&self) -> Guid {
        self.this
    }

    pub fn parent_publisher(&self) -> Guid {
        self.parent_publisher
    }

    pub fn parent_participant(&self) -> Guid {
        self.parent_participant
    }
}

impl AnyDataWriter for DataWriterNode {}

#[derive(Eq, PartialEq, Debug)]
pub struct PublisherNode {
    this: Guid,
    parent: Guid,
}

impl PublisherNode {
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

#[derive(PartialEq, Eq, Debug)]
pub struct DomainParticipantNode(Guid);

impl DomainParticipantNode {
    pub fn new(node: Guid) -> Self {
        Self(node)
    }

    pub fn guid(&self) -> Guid {
        self.0
    }
}
