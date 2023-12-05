use dust_dds_derive::actor_interface;

use crate::{
    implementation::utils::actor::ActorAddress,
    infrastructure::status::{
        RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleLostStatus,
        SampleRejectedStatus, SubscriptionMatchedStatus,
    },
    subscription::{
        data_reader::DataReader, subscriber::Subscriber, subscriber_listener::SubscriberListener,
    },
};

use super::{
    data_reader_actor::DataReaderActor, domain_participant_actor::DomainParticipantActor,
    subscriber_actor::SubscriberActor,
};

pub struct SubscriberListenerActor {
    listener: Box<dyn SubscriberListener + Send>,
}

impl SubscriberListenerActor {
    pub fn new(listener: Box<dyn SubscriberListener + Send>) -> Self {
        Self { listener }
    }
}

#[actor_interface]
impl SubscriberListenerActor {
    async fn trigger_on_data_on_readers(
        &mut self,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
    ) {
        self.listener
            .on_data_on_readers(&Subscriber::new(subscriber_address, participant_address));
    }

    async fn trigger_on_sample_rejected(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        status: SampleRejectedStatus,
    ) {
        self.listener.on_sample_rejected(
            &DataReader::<()>::new(reader_address, subscriber_address, participant_address),
            status,
        )
    }

    async fn trigger_on_requested_incompatible_qos(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        status: RequestedIncompatibleQosStatus,
    ) {
        self.listener.on_requested_incompatible_qos(
            &DataReader::<()>::new(reader_address, subscriber_address, participant_address),
            status,
        )
    }

    async fn trigger_on_requested_deadline_missed(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        status: RequestedDeadlineMissedStatus,
    ) {
        self.listener.on_requested_deadline_missed(
            &DataReader::<()>::new(reader_address, subscriber_address, participant_address),
            status,
        )
    }

    async fn trigger_on_subscription_matched(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        status: SubscriptionMatchedStatus,
    ) {
        self.listener.on_subscription_matched(
            &DataReader::<()>::new(reader_address, subscriber_address, participant_address),
            status,
        )
    }

    async fn trigger_on_sample_lost(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        status: SampleLostStatus,
    ) {
        self.listener.on_sample_lost(
            &DataReader::<()>::new(reader_address, subscriber_address, participant_address),
            status,
        )
    }
}
