use crate::{
    implementation::utils::actor::{actor_command_interface, Mail, MailHandler},
    infrastructure::status::{
        RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleLostStatus,
        SampleRejectedStatus, SubscriptionMatchedStatus,
    },
};

use super::{any_data_reader_listener::AnyDataReaderListener, nodes::DataReaderNode};

pub struct DdsDataReaderListener {
    listener: Box<dyn AnyDataReaderListener + Send + 'static>,
}

impl DdsDataReaderListener {
    pub fn new(listener: Box<dyn AnyDataReaderListener + Send + 'static>) -> Self {
        Self { listener }
    }
}

pub struct TriggerOnDataAvailable {
    reader: DataReaderNode,
}

impl TriggerOnDataAvailable {
    pub fn new(reader: DataReaderNode) -> Self {
        Self { reader }
    }
}

impl Mail for TriggerOnDataAvailable {
    type Result = ();
}

#[async_trait::async_trait]
impl MailHandler<TriggerOnDataAvailable> for DdsDataReaderListener {
    async fn handle(
        &mut self,
        mail: TriggerOnDataAvailable,
    ) -> <TriggerOnDataAvailable as Mail>::Result {
        tokio::task::block_in_place(|| self.listener.trigger_on_data_available(mail.reader));
    }
}

actor_command_interface! {
impl DdsDataReaderListener {
    pub fn trigger_on_sample_rejected(
        &mut self,
        reader: DataReaderNode,
        status: SampleRejectedStatus,
    ) {
        tokio::task::block_in_place(|| self.listener.trigger_on_sample_rejected(reader, status));
    }

    pub fn trigger_on_requested_incompatible_qos(
        &mut self,
        reader: DataReaderNode,
        status: RequestedIncompatibleQosStatus,
    ) {
        tokio::task::block_in_place(|| self.listener
            .trigger_on_requested_incompatible_qos(reader, status));
    }

    pub fn trigger_on_subscription_matched(
        &mut self,
        reader: DataReaderNode,
        status: SubscriptionMatchedStatus,
    ) {
        tokio::task::block_in_place(|| self.listener
            .trigger_on_subscription_matched(reader, status));
    }

    pub fn trigger_on_requested_deadline_missed(
        &mut self,
        reader: DataReaderNode,
        status: RequestedDeadlineMissedStatus,
    ) {
        tokio::task::block_in_place(|| self.listener
            .trigger_on_requested_deadline_missed(reader, status));
    }

    pub fn trigger_on_sample_lost(&mut self, reader: DataReaderNode, status: SampleLostStatus) {
        tokio::task::block_in_place(||self.listener.trigger_on_sample_lost(reader,status))
    }
}
}
