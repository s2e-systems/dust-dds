use crate::{
    dds_async::{
        data_reader::DataReaderAsync, subscriber::SubscriberAsync,
        subscriber_listener::SubscriberListenerAsync,
    },
    infrastructure::status::{
        RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleRejectedStatus,
        SubscriptionMatchedStatus,
    },
    runtime::{actor::MailHandler, executor::block_on},
};

pub struct SubscriberListenerActor {
    listener: Box<dyn SubscriberListenerAsync + Send>,
}

impl SubscriberListenerActor {
    pub fn new(listener: Box<dyn SubscriberListenerAsync + Send>) -> Self {
        Self { listener }
    }
}

pub enum SubscriberListenerMail {
    TriggerDataOnReaders {
        the_subscriber: SubscriberAsync,
    },
    TriggerRequestedDeadlineMissed {
        the_reader: DataReaderAsync<()>,
        status: RequestedDeadlineMissedStatus,
    },
    TriggerSampleRejected {
        the_reader: DataReaderAsync<()>,
        status: SampleRejectedStatus,
    },
    TriggerSubscriptionMatched {
        the_reader: DataReaderAsync<()>,
        status: SubscriptionMatchedStatus,
    },
    TriggerRequestedIncompatibleQos {
        the_reader: DataReaderAsync<()>,
        status: RequestedIncompatibleQosStatus,
    },
}

impl MailHandler<SubscriberListenerMail> for SubscriberListenerActor {
    fn handle(&mut self, message: SubscriberListenerMail) {
        match message {
            SubscriberListenerMail::TriggerDataOnReaders { the_subscriber } => {
                block_on(self.listener.on_data_on_readers(the_subscriber))
            }
            SubscriberListenerMail::TriggerRequestedDeadlineMissed { the_reader, status } => {
                block_on(
                    self.listener
                        .on_requested_deadline_missed(the_reader.change_foo_type(), status),
                )
            }
            SubscriberListenerMail::TriggerSampleRejected { the_reader, status } => block_on(
                self.listener
                    .on_sample_rejected(the_reader.change_foo_type(), status),
            ),
            SubscriberListenerMail::TriggerSubscriptionMatched { the_reader, status } => block_on(
                self.listener
                    .on_subscription_matched(the_reader.change_foo_type(), status),
            ),
            SubscriberListenerMail::TriggerRequestedIncompatibleQos { the_reader, status } => {
                block_on(
                    self.listener
                        .on_requested_incompatible_qos(the_reader.change_foo_type(), status),
                )
            }
        }
    }
}
