use crate::{
    dds_async::{
        data_reader::DataReaderAsync, subscriber::SubscriberAsync,
        subscriber_listener::SubscriberListenerAsync,
    },
    infrastructure::status::{
        RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleRejectedStatus,
        SubscriptionMatchedStatus,
    },
    runtime::{
        actor::{Mail, MailHandler},
        executor::block_on,
    },
};

pub struct SubscriberListenerActor {
    listener: Box<dyn SubscriberListenerAsync + Send>,
}

impl SubscriberListenerActor {
    pub fn new(listener: Box<dyn SubscriberListenerAsync + Send>) -> Self {
        Self { listener }
    }
}

pub struct TriggerDataOnReaders {
    pub the_subscriber: SubscriberAsync,
}
impl Mail for TriggerDataOnReaders {
    type Result = ();
}
impl MailHandler<TriggerDataOnReaders> for SubscriberListenerActor {
    fn handle(&mut self, message: TriggerDataOnReaders) -> <TriggerDataOnReaders as Mail>::Result {
        block_on(self.listener.on_data_on_readers(message.the_subscriber));
    }
}

pub struct TriggerRequestedDeadlineMissed {
    pub the_reader: DataReaderAsync<()>,
    pub status: RequestedDeadlineMissedStatus,
}
impl Mail for TriggerRequestedDeadlineMissed {
    type Result = ();
}
impl MailHandler<TriggerRequestedDeadlineMissed> for SubscriberListenerActor {
    fn handle(
        &mut self,
        message: TriggerRequestedDeadlineMissed,
    ) -> <TriggerRequestedDeadlineMissed as Mail>::Result {
        block_on(
            self.listener
                .on_requested_deadline_missed(message.the_reader.change_foo_type(), message.status),
        );
    }
}

pub struct TriggerSampleRejected {
    pub the_reader: DataReaderAsync<()>,
    pub status: SampleRejectedStatus,
}
impl Mail for TriggerSampleRejected {
    type Result = ();
}
impl MailHandler<TriggerSampleRejected> for SubscriberListenerActor {
    fn handle(
        &mut self,
        message: TriggerSampleRejected,
    ) -> <TriggerSampleRejected as Mail>::Result {
        block_on(
            self.listener
                .on_sample_rejected(message.the_reader.change_foo_type(), message.status),
        );
    }
}

pub struct TriggerSubscriptionMatched {
    pub the_reader: DataReaderAsync<()>,
    pub status: SubscriptionMatchedStatus,
}
impl Mail for TriggerSubscriptionMatched {
    type Result = ();
}
impl MailHandler<TriggerSubscriptionMatched> for SubscriberListenerActor {
    fn handle(
        &mut self,
        message: TriggerSubscriptionMatched,
    ) -> <TriggerSubscriptionMatched as Mail>::Result {
        block_on(
            self.listener
                .on_subscription_matched(message.the_reader.change_foo_type(), message.status),
        );
    }
}

pub struct TriggerRequestedIncompatibleQos {
    pub the_reader: DataReaderAsync<()>,
    pub status: RequestedIncompatibleQosStatus,
}
impl Mail for TriggerRequestedIncompatibleQos {
    type Result = ();
}
impl MailHandler<TriggerRequestedIncompatibleQos> for SubscriberListenerActor {
    fn handle(
        &mut self,
        message: TriggerRequestedIncompatibleQos,
    ) -> <TriggerRequestedIncompatibleQos as Mail>::Result {
        block_on(
            self.listener.on_requested_incompatible_qos(
                message.the_reader.change_foo_type(),
                message.status,
            ),
        );
    }
}
