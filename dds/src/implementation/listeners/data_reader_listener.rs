use crate::{
    dds_async::data_reader::DataReaderAsync,
    implementation::any_data_reader_listener::AnyDataReaderListener,
    infrastructure::status::{
        RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleRejectedStatus,
        SubscriptionMatchedStatus,
    },
    runtime::actor::MailHandler,
};

pub struct DataReaderListenerActor {
    listener: Box<dyn AnyDataReaderListener>,
}

impl DataReaderListenerActor {
    pub fn new(listener: Box<dyn AnyDataReaderListener>) -> Self {
        Self { listener }
    }
}

pub enum DataReaderListenerMail {
    TriggerDataAvailable {
        the_reader: DataReaderAsync<()>,
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

impl MailHandler<DataReaderListenerMail> for DataReaderListenerActor {
    fn handle(&mut self, message: DataReaderListenerMail) {
        match message {
            DataReaderListenerMail::TriggerDataAvailable { the_reader } => {
                self.listener.trigger_on_data_available(the_reader)
            }
            DataReaderListenerMail::TriggerRequestedDeadlineMissed { the_reader, status } => {
                self.listener
                    .trigger_on_requested_deadline_missed(the_reader, status);
            }
            DataReaderListenerMail::TriggerSampleRejected { the_reader, status } => {
                self.listener.trigger_on_sample_rejected(the_reader, status)
            }
            DataReaderListenerMail::TriggerSubscriptionMatched { the_reader, status } => self
                .listener
                .trigger_on_subscription_matched(the_reader, status),
            DataReaderListenerMail::TriggerRequestedIncompatibleQos { the_reader, status } => {
                self.listener
                    .trigger_on_requested_incompatible_qos(the_reader, status);
            }
        }
    }
}
