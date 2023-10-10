use dust_dds_derive::actor_interface;

use crate::{
    implementation::dds::{dds_data_reader::DataReaderNode, dds_subscriber::SubscriberNode},
    infrastructure::status::{
        RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleLostStatus,
        SampleRejectedStatus, SubscriptionMatchedStatus,
    },
    subscription::{subscriber::Subscriber, subscriber_listener::SubscriberListener},
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
    async fn trigger_on_data_on_readers(&mut self, the_subscriber: SubscriberNode) {
        tokio::task::block_in_place(|| {
            self.listener
                .on_data_on_readers(&Subscriber::new(the_subscriber))
        });
    }

    async fn trigger_on_sample_rejected(
        &mut self,
        the_reader: DataReaderNode,
        status: SampleRejectedStatus,
    ) {
        tokio::task::block_in_place(|| self.listener.on_sample_rejected(&the_reader, status));
    }

    async fn trigger_on_requested_incompatible_qos(
        &mut self,
        the_reader: DataReaderNode,
        status: RequestedIncompatibleQosStatus,
    ) {
        tokio::task::block_in_place(|| {
            self.listener
                .on_requested_incompatible_qos(&the_reader, status)
        });
    }

    async fn trigger_on_requested_deadline_missed(
        &mut self,
        the_reader: DataReaderNode,
        status: RequestedDeadlineMissedStatus,
    ) {
        tokio::task::block_in_place(|| {
            self.listener
                .on_requested_deadline_missed(&the_reader, status)
        });
    }

    async fn trigger_on_subscription_matched(
        &mut self,
        the_reader: DataReaderNode,
        status: SubscriptionMatchedStatus,
    ) {
        tokio::task::block_in_place(|| self.listener.on_subscription_matched(&the_reader, status));
    }

    async fn trigger_on_sample_lost(
        &mut self,
        the_reader: DataReaderNode,
        status: SampleLostStatus,
    ) {
        tokio::task::block_in_place(|| self.listener.on_sample_lost(&the_reader, status));
    }
}
