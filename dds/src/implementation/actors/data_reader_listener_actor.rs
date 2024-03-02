use dust_dds_derive::actor_interface;

use crate::{
    dds_async::{subscriber::SubscriberAsync, topic::TopicAsync},
    implementation::utils::actor::ActorAddress,
    infrastructure::status::{
        RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleLostStatus,
        SampleRejectedStatus, SubscriptionMatchedStatus,
    },
};

use super::{
    any_data_reader_listener::AnyDataReaderListener, data_reader_actor::DataReaderActor,
    status_condition_actor::StatusConditionActor,
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
        status_condition_address: ActorAddress<StatusConditionActor>,
        subscriber: SubscriberAsync,
        topic: TopicAsync,
    ) {
        tokio::task::block_in_place(|| {
            self.listener.trigger_on_data_available(
                reader_address,
                status_condition_address,
                subscriber,
                topic,
            )
        });
    }

    async fn trigger_on_sample_rejected(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        subscriber: SubscriberAsync,
        topic: TopicAsync,
        status: SampleRejectedStatus,
    ) {
        tokio::task::block_in_place(|| {
            self.listener.trigger_on_sample_rejected(
                reader_address,
                status_condition_address,
                subscriber,
                topic,
                status,
            )
        });
    }

    async fn trigger_on_sample_lost(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        subscriber: SubscriberAsync,
        topic: TopicAsync,
        status: SampleLostStatus,
    ) {
        tokio::task::block_in_place(|| {
            self.listener.trigger_on_sample_lost(
                reader_address,
                status_condition_address,
                subscriber,
                topic,
                status,
            )
        })
    }

    async fn trigger_on_requested_incompatible_qos(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        subscriber: SubscriberAsync,
        topic: TopicAsync,
        status: RequestedIncompatibleQosStatus,
    ) {
        tokio::task::block_in_place(|| {
            self.listener.trigger_on_requested_incompatible_qos(
                reader_address,
                status_condition_address,
                subscriber,
                topic,
                status,
            )
        });
    }

    async fn trigger_on_subscription_matched(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        subscriber: SubscriberAsync,
        topic: TopicAsync,
        status: SubscriptionMatchedStatus,
    ) {
        tokio::task::block_in_place(|| {
            self.listener.trigger_on_subscription_matched(
                reader_address,
                status_condition_address,
                subscriber,
                topic,
                status,
            )
        });
    }

    async fn trigger_on_requested_deadline_missed(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        subscriber: SubscriberAsync,
        topic: TopicAsync,
        status: RequestedDeadlineMissedStatus,
    ) {
        tokio::task::block_in_place(|| {
            self.listener.trigger_on_requested_deadline_missed(
                reader_address,
                status_condition_address,
                subscriber,
                topic,
                status,
            )
        });
    }
}
