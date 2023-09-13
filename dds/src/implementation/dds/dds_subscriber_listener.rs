use crate::{
    implementation::{
        dds::nodes::{DataReaderNode, SubscriberNode, SubscriberNodeKind},
        utils::actor::actor_command_interface,
    },
    infrastructure::status::{
        RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleRejectedStatus,
        SubscriptionMatchedStatus,
    },
    subscription::{subscriber::Subscriber, subscriber_listener::SubscriberListener},
};

pub struct DdsSubscriberListener {
    listener: Box<dyn SubscriberListener + Send>,
}

impl DdsSubscriberListener {
    pub fn new(listener: Box<dyn SubscriberListener + Send>) -> Self {
        Self { listener }
    }
}

actor_command_interface! {
impl DdsSubscriberListener {
    pub fn trigger_on_data_on_readers(&mut self, the_subscriber: SubscriberNode) {
        tokio::task::block_in_place(|| self.listener
            .on_data_on_readers(&Subscriber::new(SubscriberNodeKind::Listener(
                the_subscriber,
            ))));
    }

    pub fn trigger_on_sample_rejected(
        &mut self,
        the_reader: DataReaderNode,
        status: SampleRejectedStatus,
    ) {
        tokio::task::block_in_place(|| self.listener.on_sample_rejected(&the_reader, status));
    }

    pub fn trigger_on_requested_incompatible_qos(
        &mut self,
        the_reader: DataReaderNode,
        status: RequestedIncompatibleQosStatus,
    ) {
        tokio::task::block_in_place(|| self.listener
            .on_requested_incompatible_qos(&the_reader, status));
    }

    pub fn trigger_on_requested_deadline_missed(
        &mut self,
        reader: DataReaderNode,
        status: RequestedDeadlineMissedStatus,
    ) {
        tokio::task::block_in_place(|| self.listener.on_requested_deadline_missed(&reader, status));
    }

    pub fn trigger_on_subscription_matched(
        &mut self,
        reader: DataReaderNode,
        status: SubscriptionMatchedStatus,
    ) {
        tokio::task::block_in_place(|| self.listener.on_subscription_matched(&reader, status));
    }
}
}
