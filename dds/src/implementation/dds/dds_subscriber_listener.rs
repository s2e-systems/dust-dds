use crate::{
    implementation::{
        dds::nodes::{DataReaderNode, SubscriberNode},
        utils::actor::{Mail, MailHandler},
    },
    infrastructure::status::{
        RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleLostStatus,
        SampleRejectedStatus, SubscriptionMatchedStatus,
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

pub struct TriggerOnDataOnReaders {
    the_subscriber: SubscriberNode,
}

impl TriggerOnDataOnReaders {
    pub fn new(the_subscriber: SubscriberNode) -> Self {
        Self { the_subscriber }
    }
}

impl Mail for TriggerOnDataOnReaders {
    type Result = ();
}

#[async_trait::async_trait]
impl MailHandler<TriggerOnDataOnReaders> for DdsSubscriberListener {
    async fn handle(
        &mut self,
        mail: TriggerOnDataOnReaders,
    ) -> <TriggerOnDataOnReaders as Mail>::Result {
        tokio::task::block_in_place(|| {
            self.listener
                .on_data_on_readers(&Subscriber::new(mail.the_subscriber))
        });
    }
}

pub struct TriggerOnSampleRejected {
    the_reader: DataReaderNode,
    status: SampleRejectedStatus,
}

impl TriggerOnSampleRejected {
    pub fn new(the_reader: DataReaderNode, status: SampleRejectedStatus) -> Self {
        Self { the_reader, status }
    }
}

impl Mail for TriggerOnSampleRejected {
    type Result = ();
}

#[async_trait::async_trait]
impl MailHandler<TriggerOnSampleRejected> for DdsSubscriberListener {
    async fn handle(
        &mut self,
        mail: TriggerOnSampleRejected,
    ) -> <TriggerOnSampleRejected as Mail>::Result {
        tokio::task::block_in_place(|| {
            self.listener
                .on_sample_rejected(&mail.the_reader, mail.status)
        });
    }
}

pub struct TriggerOnRequestedIncompatibleQos {
    the_reader: DataReaderNode,
    status: RequestedIncompatibleQosStatus,
}

impl TriggerOnRequestedIncompatibleQos {
    pub fn new(the_reader: DataReaderNode, status: RequestedIncompatibleQosStatus) -> Self {
        Self { the_reader, status }
    }
}

impl Mail for TriggerOnRequestedIncompatibleQos {
    type Result = ();
}

#[async_trait::async_trait]
impl MailHandler<TriggerOnRequestedIncompatibleQos> for DdsSubscriberListener {
    async fn handle(
        &mut self,
        mail: TriggerOnRequestedIncompatibleQos,
    ) -> <TriggerOnRequestedIncompatibleQos as Mail>::Result {
        tokio::task::block_in_place(|| {
            self.listener
                .on_requested_incompatible_qos(&mail.the_reader, mail.status)
        });
    }
}

pub struct TriggerOnRequestedDeadlineMissed {
    the_reader: DataReaderNode,
    status: RequestedDeadlineMissedStatus,
}

impl TriggerOnRequestedDeadlineMissed {
    pub fn new(the_reader: DataReaderNode, status: RequestedDeadlineMissedStatus) -> Self {
        Self { the_reader, status }
    }
}

impl Mail for TriggerOnRequestedDeadlineMissed {
    type Result = ();
}

#[async_trait::async_trait]
impl MailHandler<TriggerOnRequestedDeadlineMissed> for DdsSubscriberListener {
    async fn handle(
        &mut self,
        mail: TriggerOnRequestedDeadlineMissed,
    ) -> <TriggerOnRequestedDeadlineMissed as Mail>::Result {
        tokio::task::block_in_place(|| {
            self.listener
                .on_requested_deadline_missed(&mail.the_reader, mail.status)
        });
    }
}

pub struct TriggerOnSubscriptionMatched {
    the_reader: DataReaderNode,
    status: SubscriptionMatchedStatus,
}

impl TriggerOnSubscriptionMatched {
    pub fn new(the_reader: DataReaderNode, status: SubscriptionMatchedStatus) -> Self {
        Self { the_reader, status }
    }
}

impl Mail for TriggerOnSubscriptionMatched {
    type Result = ();
}

#[async_trait::async_trait]
impl MailHandler<TriggerOnSubscriptionMatched> for DdsSubscriberListener {
    async fn handle(
        &mut self,
        mail: TriggerOnSubscriptionMatched,
    ) -> <TriggerOnSubscriptionMatched as Mail>::Result {
        tokio::task::block_in_place(|| {
            self.listener
                .on_subscription_matched(&mail.the_reader, mail.status)
        });
    }
}

pub struct TriggerOnSampleLost {
    the_reader: DataReaderNode,
    status: SampleLostStatus,
}

impl TriggerOnSampleLost {
    pub fn new(the_reader: DataReaderNode, status: SampleLostStatus) -> Self {
        Self { the_reader, status }
    }
}

impl Mail for TriggerOnSampleLost {
    type Result = ();
}

#[async_trait::async_trait]
impl MailHandler<TriggerOnSampleLost> for DdsSubscriberListener {
    async fn handle(&mut self, mail: TriggerOnSampleLost) -> <TriggerOnSampleLost as Mail>::Result {
        tokio::task::block_in_place(|| self.listener.on_sample_lost(&mail.the_reader, mail.status));
    }
}
