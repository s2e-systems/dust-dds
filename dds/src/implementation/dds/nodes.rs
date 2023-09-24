use crate::{
    implementation::{dds::dds_domain_participant, utils::actor::ActorAddress},
    publication::data_writer::AnyDataWriter,
    subscription::data_reader::AnyDataReader,
};

use super::{
    dds_data_reader::{self, DdsDataReader},
    dds_data_writer::DdsDataWriter,
    dds_domain_participant::DdsDomainParticipant,
    dds_publisher::DdsPublisher,
    dds_subscriber::DdsSubscriber,
    dds_topic::DdsTopic,
};

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
        let user_defined_topic_list = self
            .parent_participant
            .send_and_reply_blocking(dds_domain_participant::GetUserDefinedTopicList)
            .expect("should never fail");
        for topic in user_defined_topic_list {
            if topic.get_type_name()
                == self
                    .this
                    .send_and_reply_blocking(dds_data_reader::GetTypeName)
                && topic.get_name()
                    == self
                        .this
                        .send_and_reply_blocking(dds_data_reader::GetTopicName)
            {
                return topic;
            }
        }
        panic!("Should always exist");
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
