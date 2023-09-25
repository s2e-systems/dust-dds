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
pub struct DomainParticipantNode {
    participant_address: ActorAddress<DdsDomainParticipant>,
}

impl DomainParticipantNode {
    pub fn new(participant_address: ActorAddress<DdsDomainParticipant>) -> Self {
        Self {
            participant_address,
        }
    }

    pub fn participant_address(&self) -> &ActorAddress<DdsDomainParticipant> {
        &self.participant_address
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct SubscriberNode {
    subscriber_address: ActorAddress<DdsSubscriber>,
    participant_address: ActorAddress<DdsDomainParticipant>,
}

impl SubscriberNode {
    pub fn new(
        subscriber_address: ActorAddress<DdsSubscriber>,
        participant_address: ActorAddress<DdsDomainParticipant>,
    ) -> Self {
        Self {
            subscriber_address,
            participant_address,
        }
    }

    pub fn subscriber_address(&self) -> &ActorAddress<DdsSubscriber> {
        &self.subscriber_address
    }

    pub fn participant_address(&self) -> &ActorAddress<DdsDomainParticipant> {
        &self.participant_address
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct DataReaderNode {
    reader_address: ActorAddress<DdsDataReader>,
    subscriber_address: ActorAddress<DdsSubscriber>,
    participant_address: ActorAddress<DdsDomainParticipant>,
}

impl DataReaderNode {
    pub fn new(
        reader_address: ActorAddress<DdsDataReader>,
        subscriber_address: ActorAddress<DdsSubscriber>,
        participant_address: ActorAddress<DdsDomainParticipant>,
    ) -> Self {
        Self {
            reader_address,
            subscriber_address,
            participant_address,
        }
    }

    pub fn reader_address(&self) -> &ActorAddress<DdsDataReader> {
        &self.reader_address
    }

    pub fn subscriber_address(&self) -> &ActorAddress<DdsSubscriber> {
        &self.subscriber_address
    }

    pub fn participant_address(&self) -> &ActorAddress<DdsDomainParticipant> {
        &self.participant_address
    }

    pub fn topic_address(&self) -> ActorAddress<DdsTopic> {
        let user_defined_topic_list = self
            .participant_address
            .send_and_reply_blocking(dds_domain_participant::GetUserDefinedTopicList)
            .expect("should never fail");
        for topic in user_defined_topic_list {
            if topic.get_type_name()
                == self
                    .reader_address
                    .send_and_reply_blocking(dds_data_reader::GetTypeName)
                && topic.get_name()
                    == self
                        .reader_address
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
    topic_address: ActorAddress<DdsTopic>,
    participant_address: ActorAddress<DdsDomainParticipant>,
}

impl TopicNode {
    pub fn new(
        topic_address: ActorAddress<DdsTopic>,
        participant_address: ActorAddress<DdsDomainParticipant>,
    ) -> Self {
        Self {
            topic_address,
            participant_address,
        }
    }

    pub fn topic_address(&self) -> &ActorAddress<DdsTopic> {
        &self.topic_address
    }

    pub fn participant_address(&self) -> &ActorAddress<DdsDomainParticipant> {
        &self.participant_address
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct DataWriterNode {
    writer_address: ActorAddress<DdsDataWriter>,
    publisher_address: ActorAddress<DdsPublisher>,
    participant_address: ActorAddress<DdsDomainParticipant>,
}

impl DataWriterNode {
    pub fn new(
        writer_address: ActorAddress<DdsDataWriter>,
        publisher_address: ActorAddress<DdsPublisher>,
        participant_address: ActorAddress<DdsDomainParticipant>,
    ) -> Self {
        Self {
            writer_address,
            publisher_address,
            participant_address,
        }
    }

    pub fn writer_address(&self) -> &ActorAddress<DdsDataWriter> {
        &self.writer_address
    }

    pub fn publisher_address(&self) -> &ActorAddress<DdsPublisher> {
        &self.publisher_address
    }

    pub fn participant_address(&self) -> &ActorAddress<DdsDomainParticipant> {
        &self.participant_address
    }
}

impl AnyDataWriter for DataWriterNode {}

#[derive(Clone, PartialEq, Eq)]
pub struct PublisherNode {
    publisher_address: ActorAddress<DdsPublisher>,
    participant_address: ActorAddress<DdsDomainParticipant>,
}

impl PublisherNode {
    pub fn new(
        publisher_address: ActorAddress<DdsPublisher>,
        participant_address: ActorAddress<DdsDomainParticipant>,
    ) -> Self {
        Self {
            publisher_address,
            participant_address,
        }
    }

    pub fn publisher_address(&self) -> &ActorAddress<DdsPublisher> {
        &self.publisher_address
    }

    pub fn participant_address(&self) -> &ActorAddress<DdsDomainParticipant> {
        &self.participant_address
    }
}
