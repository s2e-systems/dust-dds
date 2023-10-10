use dust_dds_derive::actor_interface;

use crate::{
    implementation::dds::nodes::DataReaderNode,
    infrastructure::status::{
        RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleLostStatus,
        SampleRejectedStatus, SubscriptionMatchedStatus,
    },
};

use super::any_data_reader_listener::AnyDataReaderListener;

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
    async fn trigger_on_data_available(&mut self, reader: DataReaderNode) {
        tokio::task::block_in_place(|| self.listener.trigger_on_data_available(reader));
    }

    async fn trigger_on_sample_rejected(
        &mut self,
        reader: DataReaderNode,
        status: SampleRejectedStatus,
    ) {
        tokio::task::block_in_place(|| self.listener.trigger_on_sample_rejected(reader, status));
    }

    async fn trigger_on_sample_lost(&mut self, reader: DataReaderNode, status: SampleLostStatus) {
        tokio::task::block_in_place(|| self.listener.trigger_on_sample_lost(reader, status))
    }

    async fn trigger_on_requested_incompatible_qos(
        &mut self,
        reader: DataReaderNode,
        status: RequestedIncompatibleQosStatus,
    ) {
        tokio::task::block_in_place(|| {
            self.listener
                .trigger_on_requested_incompatible_qos(reader, status)
        });
    }

    async fn trigger_on_subscription_matched(
        &mut self,
        reader: DataReaderNode,
        status: SubscriptionMatchedStatus,
    ) {
        tokio::task::block_in_place(|| {
            self.listener
                .trigger_on_subscription_matched(reader, status)
        });
    }

    async fn trigger_on_requested_deadline_missed(
        &mut self,
        reader: DataReaderNode,
        status: RequestedDeadlineMissedStatus,
    ) {
        tokio::task::block_in_place(|| {
            self.listener
                .trigger_on_requested_deadline_missed(reader, status)
        });
    }
}
