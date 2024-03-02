use crate::{
    dds_async::{data_reader::DataReaderAsync, subscriber::SubscriberAsync, topic::TopicAsync},
    implementation::utils::actor::ActorAddress,
    infrastructure::status::{
        LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
        SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
    },
    subscription::{data_reader::DataReader, data_reader_listener::DataReaderListener},
};

use super::{data_reader_actor::DataReaderActor, status_condition_actor::StatusConditionActor};

pub trait AnyDataReaderListener {
    fn trigger_on_data_available(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        subscriber: SubscriberAsync,
        topic: TopicAsync,
    );
    fn trigger_on_sample_rejected(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        subscriber: SubscriberAsync,
        topic: TopicAsync,
        status: SampleRejectedStatus,
    );
    #[allow(dead_code)]
    fn trigger_on_liveliness_changed(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        subscriber: SubscriberAsync,
        topic: TopicAsync,
        status: LivelinessChangedStatus,
    );
    fn trigger_on_requested_deadline_missed(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        subscriber: SubscriberAsync,
        topic: TopicAsync,
        status: RequestedDeadlineMissedStatus,
    );
    fn trigger_on_requested_incompatible_qos(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        subscriber: SubscriberAsync,
        topic: TopicAsync,
        status: RequestedIncompatibleQosStatus,
    );
    fn trigger_on_subscription_matched(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        subscriber: SubscriberAsync,
        topic: TopicAsync,
        status: SubscriptionMatchedStatus,
    );
    fn trigger_on_sample_lost(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        subscriber: SubscriberAsync,
        topic: TopicAsync,
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
        status_condition_address: ActorAddress<StatusConditionActor>,
        subscriber: SubscriberAsync,
        topic: TopicAsync,
    ) {
        self.on_data_available(&DataReader::new(DataReaderAsync::new(
            reader_address,
            status_condition_address,
            subscriber,
            topic,
        )))
    }

    fn trigger_on_sample_rejected(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        subscriber: SubscriberAsync,
        topic: TopicAsync,
        status: SampleRejectedStatus,
    ) {
        self.on_sample_rejected(
            &DataReader::new(DataReaderAsync::new(
                reader_address,
                status_condition_address,
                subscriber,
                topic,
            )),
            status,
        )
    }

    fn trigger_on_liveliness_changed(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        subscriber: SubscriberAsync,
        topic: TopicAsync,
        status: LivelinessChangedStatus,
    ) {
        self.on_liveliness_changed(
            &DataReader::new(DataReaderAsync::new(
                reader_address,
                status_condition_address,
                subscriber,
                topic,
            )),
            status,
        )
    }

    fn trigger_on_requested_deadline_missed(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        subscriber: SubscriberAsync,
        topic: TopicAsync,
        status: RequestedDeadlineMissedStatus,
    ) {
        self.on_requested_deadline_missed(
            &DataReader::new(DataReaderAsync::new(
                reader_address,
                status_condition_address,
                subscriber,
                topic,
            )),
            status,
        )
    }

    fn trigger_on_requested_incompatible_qos(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        subscriber: SubscriberAsync,
        topic: TopicAsync,
        status: RequestedIncompatibleQosStatus,
    ) {
        self.on_requested_incompatible_qos(
            &DataReader::new(DataReaderAsync::new(
                reader_address,
                status_condition_address,
                subscriber,
                topic,
            )),
            status,
        )
    }

    fn trigger_on_subscription_matched(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        subscriber: SubscriberAsync,
        topic: TopicAsync,
        status: SubscriptionMatchedStatus,
    ) {
        self.on_subscription_matched(
            &DataReader::new(DataReaderAsync::new(
                reader_address,
                status_condition_address,
                subscriber,
                topic,
            )),
            status,
        )
    }

    fn trigger_on_sample_lost(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        subscriber: SubscriberAsync,
        topic: TopicAsync,
        status: SampleLostStatus,
    ) {
        self.on_sample_lost(
            &DataReader::new(DataReaderAsync::new(
                reader_address,
                status_condition_address,
                subscriber,
                topic,
            )),
            status,
        )
    }
}
