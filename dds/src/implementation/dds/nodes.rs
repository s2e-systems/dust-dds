use crate::{
    implementation::{
        rtps::{
            stateful_reader::RtpsStatefulReader, stateful_writer::RtpsStatefulWriter, types::Guid,
        },
        utils::actor::ActorAddress,
    },
    publication::data_writer::AnyDataWriter,
    subscription::data_reader::AnyDataReader,
    topic_definition::topic::AnyTopic,
};

use super::{
    dds_data_reader::DdsDataReader, dds_data_writer::DdsDataWriter,
    dds_domain_participant::DdsDomainParticipant, dds_publisher::DdsPublisher,
    dds_subscriber::DdsSubscriber, dds_topic::DdsTopic,
};

pub enum SubscriberNodeKind {
    Builtin(SubscriberNode),
    UserDefined(SubscriberNode),
    _Listener(SubscriberNode),
}

pub enum DataWriterNodeKind {
    UserDefined(DataWriterNode),
    Listener(DataWriterNode),
}

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

#[derive(Clone)]
pub struct DataReaderNode {
    this: ActorAddress<DdsDataReader<RtpsStatefulReader>>,
    parent_subscriber: ActorAddress<DdsSubscriber>,
    parent_participant: ActorAddress<DdsDomainParticipant>,
}

impl DataReaderNode {
    pub fn new(
        this: ActorAddress<DdsDataReader<RtpsStatefulReader>>,
        parent_subscriber: ActorAddress<DdsSubscriber>,
        parent_participant: ActorAddress<DdsDomainParticipant>,
    ) -> Self {
        Self {
            this,
            parent_subscriber,
            parent_participant,
        }
    }

    pub fn address(&self) -> &ActorAddress<DdsDataReader<RtpsStatefulReader>> {
        &self.this
    }

    pub fn parent_subscriber(&self) -> &ActorAddress<DdsSubscriber> {
        &self.parent_subscriber
    }

    pub fn parent_participant(&self) -> &ActorAddress<DdsDomainParticipant> {
        &self.parent_participant
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

#[derive(Clone)]
pub struct DataWriterNode {
    this: ActorAddress<DdsDataWriter<RtpsStatefulWriter>>,
    parent_publisher: ActorAddress<DdsPublisher>,
    parent_participant: ActorAddress<DdsDomainParticipant>,
}

impl DataWriterNode {
    pub fn new(
        this: ActorAddress<DdsDataWriter<RtpsStatefulWriter>>,
        parent_publisher: ActorAddress<DdsPublisher>,
        parent_participant: ActorAddress<DdsDomainParticipant>,
    ) -> Self {
        Self {
            this,
            parent_publisher,
            parent_participant,
        }
    }

    pub fn address(&self) -> &ActorAddress<DdsDataWriter<RtpsStatefulWriter>> {
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

#[derive(PartialEq, Eq, Debug)]
pub struct DomainParticipantNode(Guid);

impl DomainParticipantNode {
    pub fn _new(node: Guid) -> Self {
        Self(node)
    }

    pub fn _guid(&self) -> Guid {
        self.0
    }
}
