use crate::{
    implementation::utils::actor::ActorAddress, publication::data_writer::AnyDataWriter,
    subscription::data_reader::AnyDataReader, topic_definition::topic::AnyTopic,
};

use super::{
    dds_data_reader::DdsDataReader, dds_data_writer::DdsDataWriter,
    dds_domain_participant::DdsDomainParticipant, dds_publisher::DdsPublisher,
    dds_subscriber::DdsSubscriber, dds_topic::DdsTopic,
};

#[derive(Clone, PartialEq, Eq)]
pub enum SubscriberNodeKind {
    Builtin(SubscriberNode),
    UserDefined(SubscriberNode),
    Listener(SubscriberNode),
}

#[derive(Clone, PartialEq, Eq)]
pub enum DataWriterNodeKind {
    UserDefined(DataWriterNode),
    Listener(DataWriterNode),
}

#[derive(Clone, PartialEq, Eq)]
pub enum DataReaderNodeKind {
    _BuiltinStateful(DataReaderNode),
    _BuiltinStateless(DataReaderNode),
    UserDefined(DataReaderNode),
    Listener(DataReaderNode),
}

#[derive(Clone, PartialEq, Eq)]
pub enum TopicNodeKind {
    UserDefined(TopicNode),
    Listener(TopicNode),
}

#[derive(Clone, PartialEq, Eq)]
pub struct SubscriberNode {
    this: ActorAddress<DdsSubscriber>,
    parent: ActorAddress<DdsDomainParticipant>,
}

impl SubscriberNode {
    pub fn new(
        this: ActorAddress<DdsSubscriber>,
        parent: ActorAddress<DdsDomainParticipant>,
    ) -> Self {
        Self { this, parent }
    }

    pub fn address(&self) -> &ActorAddress<DdsSubscriber> {
        &self.this
    }

    pub fn parent_participant(&self) -> &ActorAddress<DdsDomainParticipant> {
        &self.parent
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct DataReaderNode {
    this: ActorAddress<DdsDataReader>,
    parent_subscriber: ActorAddress<DdsSubscriber>,
    parent_participant: ActorAddress<DdsDomainParticipant>,
}

impl DataReaderNode {
    pub fn new(
        this: ActorAddress<DdsDataReader>,
        parent_subscriber: ActorAddress<DdsSubscriber>,
        parent_participant: ActorAddress<DdsDomainParticipant>,
    ) -> Self {
        Self {
            this,
            parent_subscriber,
            parent_participant,
        }
    }

    pub fn address(&self) -> &ActorAddress<DdsDataReader> {
        &self.this
    }

    pub fn parent_subscriber(&self) -> &ActorAddress<DdsSubscriber> {
        &self.parent_subscriber
    }

    pub fn parent_participant(&self) -> &ActorAddress<DdsDomainParticipant> {
        &self.parent_participant
    }

    pub fn topic_address(&self) -> ActorAddress<DdsTopic> {
        self.parent_participant
            .get_user_defined_topic_list()
            .expect("should never fail")
            .iter()
            .find(|t| {
                t.get_type_name() == self.this.get_type_name()
                    && t.get_name() == self.this.get_topic_name()
            })
            .expect("should always exist")
            .clone()
    }
}

impl AnyDataReader for DataReaderNode {}

#[derive(Clone, PartialEq, Eq)]
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

#[derive(Clone, PartialEq, Eq)]
pub struct DataWriterNode {
    this: ActorAddress<DdsDataWriter>,
    parent_publisher: ActorAddress<DdsPublisher>,
    parent_participant: ActorAddress<DdsDomainParticipant>,
}

impl DataWriterNode {
    pub fn new(
        this: ActorAddress<DdsDataWriter>,
        parent_publisher: ActorAddress<DdsPublisher>,
        parent_participant: ActorAddress<DdsDomainParticipant>,
    ) -> Self {
        Self {
            this,
            parent_publisher,
            parent_participant,
        }
    }

    pub fn address(&self) -> &ActorAddress<DdsDataWriter> {
        &self.this
    }

    pub fn parent_publisher(&self) -> &ActorAddress<DdsPublisher> {
        &self.parent_publisher
    }

    pub fn parent_participant(&self) -> &ActorAddress<DdsDomainParticipant> {
        &self.parent_participant
    }
}

impl AnyDataWriter for DataWriterNode {}

#[derive(Clone, PartialEq, Eq)]
pub struct PublisherNode {
    this: ActorAddress<DdsPublisher>,
    parent: ActorAddress<DdsDomainParticipant>,
}

impl PublisherNode {
    pub fn new(
        this: ActorAddress<DdsPublisher>,
        parent: ActorAddress<DdsDomainParticipant>,
    ) -> Self {
        Self { this, parent }
    }

    pub fn address(&self) -> &ActorAddress<DdsPublisher> {
        &self.this
    }

    pub fn parent_participant(&self) -> &ActorAddress<DdsDomainParticipant> {
        &self.parent
    }
}
