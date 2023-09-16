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

pub struct TriggerOnSampleRejected {
    reader: DataReaderNode,
    status: SampleRejectedStatus,
}

impl TriggerOnSampleRejected {
    pub fn new(reader: DataReaderNode, status: SampleRejectedStatus) -> Self {
        Self { reader, status }
    }
}

impl Mail for TriggerOnSampleRejected {
    type Result = ();
}

#[async_trait::async_trait]
impl MailHandler<TriggerOnSampleRejected> for DdsDataReaderListener {
    async fn handle(
        &mut self,
        mail: TriggerOnSampleRejected,
    ) -> <TriggerOnSampleRejected as Mail>::Result {
        tokio::task::block_in_place(|| {
            self.listener
                .trigger_on_sample_rejected(mail.reader, mail.status)
        });
    }
}

pub struct TriggerOnSampleLost {
    reader: DataReaderNode,
    status: SampleLostStatus,
}

impl TriggerOnSampleLost {
    pub fn new(reader: DataReaderNode, status: SampleLostStatus) -> Self {
        Self { reader, status }
    }
}

impl Mail for TriggerOnSampleLost {
    type Result = ();
}

#[async_trait::async_trait]
impl MailHandler<TriggerOnSampleLost> for DdsDataReaderListener {
    async fn handle(&mut self, mail: TriggerOnSampleLost) -> <TriggerOnSampleLost as Mail>::Result {
        tokio::task::block_in_place(|| {
            self.listener
                .trigger_on_sample_lost(mail.reader, mail.status)
        })
    }
}

actor_command_interface! {
impl DdsDataReaderListener {
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
}
}
