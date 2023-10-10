use crate::{
    implementation::{
        actors::{
            data_reader_actor::{self, DataReaderActor},
            domain_participant_actor::{self, DomainParticipantActor},
            subscriber_actor::SubscriberActor,
            topic_actor::{self, TopicActor},
        },
        utils::actor::ActorAddress,
    },
    subscription::data_reader::AnyDataReader,
};

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
