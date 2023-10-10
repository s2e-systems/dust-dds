use dust_dds_derive::actor_interface;

use crate::{
    implementation::dds::dds_data_reader::DdsDataReader,
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
    async fn trigger_on_data_available(&mut self, reader: DdsDataReader) {
        tokio::task::block_in_place(|| self.listener.trigger_on_data_available(reader));
    }

    async fn trigger_on_sample_rejected(
        &mut self,
        reader: DdsDataReader,
        status: SampleRejectedStatus,
    ) {
        tokio::task::block_in_place(|| self.listener.trigger_on_sample_rejected(reader, status));
    }

    async fn trigger_on_sample_lost(&mut self, reader: DdsDataReader, status: SampleLostStatus) {
        tokio::task::block_in_place(|| self.listener.trigger_on_sample_lost(reader, status))
    }

    async fn trigger_on_requested_incompatible_qos(
        &mut self,
        reader: DdsDataReader,
        status: RequestedIncompatibleQosStatus,
    ) {
        tokio::task::block_in_place(|| {
            self.listener
                .trigger_on_requested_incompatible_qos(reader, status)
        });
    }

    async fn trigger_on_subscription_matched(
        &mut self,
        reader: DdsDataReader,
        status: SubscriptionMatchedStatus,
    ) {
        tokio::task::block_in_place(|| {
            self.listener
                .trigger_on_subscription_matched(reader, status)
        });
    }

    async fn trigger_on_requested_deadline_missed(
        &mut self,
        reader: DdsDataReader,
        status: RequestedDeadlineMissedStatus,
    ) {
        tokio::task::block_in_place(|| {
            self.listener
                .trigger_on_requested_deadline_missed(reader, status)
        });
    }
}
