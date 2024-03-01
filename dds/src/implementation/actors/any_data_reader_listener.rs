use crate::{
    dds_async::data_reader::DataReaderAsync,
    implementation::utils::actor::ActorAddress,
    infrastructure::status::{
        LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
        SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
    },
    subscription::{data_reader::DataReader, data_reader_listener::DataReaderListener},
};

use super::{
    data_reader_actor::DataReaderActor, domain_participant_actor::DomainParticipantActor,
    subscriber_actor::SubscriberActor, topic_actor::TopicActor,
};

pub trait AnyDataReaderListener {
    fn trigger_on_data_available(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        topic_address: ActorAddress<TopicActor>,
        runtime_handle: tokio::runtime::Handle,
    );
    fn trigger_on_sample_rejected(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        topic_address: ActorAddress<TopicActor>,
        runtime_handle: tokio::runtime::Handle,
        status: SampleRejectedStatus,
    );
    #[allow(dead_code)]
    fn trigger_on_liveliness_changed(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        topic_address: ActorAddress<TopicActor>,
        runtime_handle: tokio::runtime::Handle,
        status: LivelinessChangedStatus,
    );
    fn trigger_on_requested_deadline_missed(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        topic_address: ActorAddress<TopicActor>,
        runtime_handle: tokio::runtime::Handle,
        status: RequestedDeadlineMissedStatus,
    );
    fn trigger_on_requested_incompatible_qos(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        topic_address: ActorAddress<TopicActor>,
        runtime_handle: tokio::runtime::Handle,
        status: RequestedIncompatibleQosStatus,
    );
    fn trigger_on_subscription_matched(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        topic_address: ActorAddress<TopicActor>,
        runtime_handle: tokio::runtime::Handle,
        status: SubscriptionMatchedStatus,
    );
    fn trigger_on_sample_lost(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        topic_address: ActorAddress<TopicActor>,
        runtime_handle: tokio::runtime::Handle,
        status: SampleLostStatus,
    );
}

impl<T> AnyDataReaderListener for T
where
    T: DataReaderListener,
{
    fn trigger_on_data_available(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        topic_address: ActorAddress<TopicActor>,
        runtime_handle: tokio::runtime::Handle,
    ) {
        self.on_data_available(&DataReader::new(DataReaderAsync::new(
            reader_address,
            subscriber_address,
            participant_address,
            topic_address,
            runtime_handle,
        )))
    }

    fn trigger_on_sample_rejected(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        topic_address: ActorAddress<TopicActor>,
        runtime_handle: tokio::runtime::Handle,
        status: SampleRejectedStatus,
    ) {
        self.on_sample_rejected(
            &DataReader::new(DataReaderAsync::new(
                reader_address,
                subscriber_address,
                participant_address,
                topic_address,
                runtime_handle,
            )),
            status,
        )
    }

    fn trigger_on_liveliness_changed(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        topic_address: ActorAddress<TopicActor>,
        runtime_handle: tokio::runtime::Handle,
        status: LivelinessChangedStatus,
    ) {
        self.on_liveliness_changed(
            &DataReader::new(DataReaderAsync::new(
                reader_address,
                subscriber_address,
                participant_address,
                topic_address,
                runtime_handle,
            )),
            status,
        )
    }

    fn trigger_on_requested_deadline_missed(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        topic_address: ActorAddress<TopicActor>,
        runtime_handle: tokio::runtime::Handle,
        status: RequestedDeadlineMissedStatus,
    ) {
        self.on_requested_deadline_missed(
            &DataReader::new(DataReaderAsync::new(
                reader_address,
                subscriber_address,
                participant_address,
                topic_address,
                runtime_handle,
            )),
            status,
        )
    }

    fn trigger_on_requested_incompatible_qos(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        topic_address: ActorAddress<TopicActor>,
        runtime_handle: tokio::runtime::Handle,
        status: RequestedIncompatibleQosStatus,
    ) {
        self.on_requested_incompatible_qos(
            &DataReader::new(DataReaderAsync::new(
                reader_address,
                subscriber_address,
                participant_address,
                topic_address,
                runtime_handle,
            )),
            status,
        )
    }

    fn trigger_on_subscription_matched(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        topic_address: ActorAddress<TopicActor>,
        runtime_handle: tokio::runtime::Handle,
        status: SubscriptionMatchedStatus,
    ) {
        self.on_subscription_matched(
            &DataReader::new(DataReaderAsync::new(
                reader_address,
                subscriber_address,
                participant_address,
                topic_address,
                runtime_handle,
            )),
            status,
        )
    }

    fn trigger_on_sample_lost(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        topic_address: ActorAddress<TopicActor>,
        runtime_handle: tokio::runtime::Handle,
        status: SampleLostStatus,
    ) {
        self.on_sample_lost(
            &DataReader::new(DataReaderAsync::new(
                reader_address,
                subscriber_address,
                participant_address,
                topic_address,
                runtime_handle,
            )),
            status,
        )
    }
}
