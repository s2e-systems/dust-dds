use crate::{
    implementation::{
        actors::{
            data_reader_actor::{self, DataReaderActor},
            data_writer_actor::DataWriterActor,
            domain_participant_actor::{self, DomainParticipantActor},
            publisher_actor::PublisherActor,
            subscriber_actor::SubscriberActor,
            topic_actor::{self, TopicActor},
        },
        utils::actor::ActorAddress,
    },
    publication::data_writer::AnyDataWriter,
    subscription::data_reader::AnyDataReader,
};



#[derive(Clone, PartialEq, Eq)]
pub struct SubscriberNode {
    subscriber_address: ActorAddress<SubscriberActor>,
    participant_address: ActorAddress<DomainParticipantActor>,
}

impl SubscriberNode {
    pub fn new(
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
    ) -> Self {
        Self {
            subscriber_address,
            participant_address,
        }
    }

    pub fn subscriber_address(&self) -> &ActorAddress<SubscriberActor> {
        &self.subscriber_address
    }

    pub fn participant_address(&self) -> &ActorAddress<DomainParticipantActor> {
        &self.participant_address
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct DataReaderNode {
    reader_address: ActorAddress<DataReaderActor>,
    subscriber_address: ActorAddress<SubscriberActor>,
    participant_address: ActorAddress<DomainParticipantActor>,
}

impl DataReaderNode {
    pub fn new(
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
    ) -> Self {
        Self {
            reader_address,
            subscriber_address,
            participant_address,
        }
    }

    pub fn reader_address(&self) -> &ActorAddress<DataReaderActor> {
        &self.reader_address
    }

    pub fn subscriber_address(&self) -> &ActorAddress<SubscriberActor> {
        &self.subscriber_address
    }

    pub fn participant_address(&self) -> &ActorAddress<DomainParticipantActor> {
        &self.participant_address
    }

    pub fn topic_address(&self) -> ActorAddress<TopicActor> {
        let user_defined_topic_list = self
            .participant_address
            .send_mail_and_await_reply_blocking(
                domain_participant_actor::get_user_defined_topic_list::new(),
            )
            .expect("should never fail");
        for topic in user_defined_topic_list {
            if topic.send_mail_and_await_reply_blocking(topic_actor::get_type_name::new())
                == self
                    .reader_address
                    .send_mail_and_await_reply_blocking(data_reader_actor::get_type_name::new())
                && topic.send_mail_and_await_reply_blocking(topic_actor::get_name::new())
                    == self
                        .reader_address
                        .send_mail_and_await_reply_blocking(data_reader_actor::get_topic_name::new())
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
    topic_address: ActorAddress<TopicActor>,
    participant_address: ActorAddress<DomainParticipantActor>,
}

impl TopicNode {
    pub fn new(
        topic_address: ActorAddress<TopicActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
    ) -> Self {
        Self {
            topic_address,
            participant_address,
        }
    }

    pub fn topic_address(&self) -> &ActorAddress<TopicActor> {
        &self.topic_address
    }

    pub fn participant_address(&self) -> &ActorAddress<DomainParticipantActor> {
        &self.participant_address
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct DataWriterNode {
    writer_address: ActorAddress<DataWriterActor>,
    publisher_address: ActorAddress<PublisherActor>,
    participant_address: ActorAddress<DomainParticipantActor>,
}

impl DataWriterNode {
    pub fn new(
        writer_address: ActorAddress<DataWriterActor>,
        publisher_address: ActorAddress<PublisherActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
    ) -> Self {
        Self {
            writer_address,
            publisher_address,
            participant_address,
        }
    }

    pub fn writer_address(&self) -> &ActorAddress<DataWriterActor> {
        &self.writer_address
    }

    pub fn publisher_address(&self) -> &ActorAddress<PublisherActor> {
        &self.publisher_address
    }

    pub fn participant_address(&self) -> &ActorAddress<DomainParticipantActor> {
        &self.participant_address
    }
}

impl AnyDataWriter for DataWriterNode {}

#[derive(Clone, PartialEq, Eq)]
pub struct PublisherNode {
    publisher_address: ActorAddress<PublisherActor>,
    participant_address: ActorAddress<DomainParticipantActor>,
}

impl PublisherNode {
    pub fn new(
        publisher_address: ActorAddress<PublisherActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
    ) -> Self {
        Self {
            publisher_address,
            participant_address,
        }
    }

    pub fn publisher_address(&self) -> &ActorAddress<PublisherActor> {
        &self.publisher_address
    }

    pub fn participant_address(&self) -> &ActorAddress<DomainParticipantActor> {
        &self.participant_address
    }
}
