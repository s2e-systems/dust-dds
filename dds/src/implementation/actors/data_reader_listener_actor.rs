use dust_dds_derive::actor_interface;

use crate::{
    implementation::utils::actor::ActorAddress,
    infrastructure::status::{
        RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleLostStatus,
        SampleRejectedStatus, SubscriptionMatchedStatus,
    },
};

use super::{
    any_data_reader_listener::AnyDataReaderListener, data_reader_actor::DataReaderActor,
    domain_participant_actor::DomainParticipantActor, subscriber_actor::SubscriberActor,
    topic_actor::TopicActor,
};

pub struct DataReaderListenerActor {
    listener: Box<dyn AnyDataReaderListener + Send + 'static>,
}

impl DataReaderListenerActor {
    pub fn new(listener: Box<dyn AnyDataReaderListener + Send + 'static>) -> Self {
        Self { listener }
    }
}

#[actor_interface]
impl DataReaderListenerActor {
    async fn trigger_on_data_available(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        topic_address: ActorAddress<TopicActor>,
        runtime_handle: tokio::runtime::Handle,
    ) {
        tokio::task::block_in_place(|| {
            self.listener.trigger_on_data_available(
                reader_address,
                subscriber_address,
                participant_address,
                topic_address,
                runtime_handle,
            )
        });
    }

    async fn trigger_on_sample_rejected(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        topic_address: ActorAddress<TopicActor>,
        runtime_handle: tokio::runtime::Handle,
        status: SampleRejectedStatus,
    ) {
        tokio::task::block_in_place(|| {
            self.listener.trigger_on_sample_rejected(
                reader_address,
                subscriber_address,
                participant_address,
                topic_address,
                runtime_handle,
                status,
            )
        });
    }

    async fn trigger_on_sample_lost(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        topic_address: ActorAddress<TopicActor>,
        runtime_handle: tokio::runtime::Handle,
        status: SampleLostStatus,
    ) {
        tokio::task::block_in_place(|| {
            self.listener.trigger_on_sample_lost(
                reader_address,
                subscriber_address,
                participant_address,
                topic_address,
                runtime_handle,
                status,
            )
        })
    }

    async fn trigger_on_requested_incompatible_qos(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        topic_address: ActorAddress<TopicActor>,
        runtime_handle: tokio::runtime::Handle,
        status: RequestedIncompatibleQosStatus,
    ) {
        tokio::task::block_in_place(|| {
            self.listener.trigger_on_requested_incompatible_qos(
                reader_address,
                subscriber_address,
                participant_address,
                topic_address,
                runtime_handle,
                status,
            )
        });
    }

    async fn trigger_on_subscription_matched(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        topic_address: ActorAddress<TopicActor>,
        runtime_handle: tokio::runtime::Handle,
        status: SubscriptionMatchedStatus,
    ) {
        tokio::task::block_in_place(|| {
            self.listener.trigger_on_subscription_matched(
                reader_address,
                subscriber_address,
                participant_address,
                topic_address,
                runtime_handle,
                status,
            )
        });
    }

    async fn trigger_on_requested_deadline_missed(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        topic_address: ActorAddress<TopicActor>,
        runtime_handle: tokio::runtime::Handle,
        status: RequestedDeadlineMissedStatus,
    ) {
        tokio::task::block_in_place(|| {
            self.listener.trigger_on_requested_deadline_missed(
                reader_address,
                subscriber_address,
                participant_address,
                topic_address,
                runtime_handle,
                status,
            )
        });
    }
}
