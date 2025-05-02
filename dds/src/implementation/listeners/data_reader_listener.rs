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
    DataAvailable {
        the_reader: DataReaderAsync<()>,
    },
    RequestedDeadlineMissed {
        the_reader: DataReaderAsync<()>,
        status: RequestedDeadlineMissedStatus,
    },
    SampleRejected {
        the_reader: DataReaderAsync<()>,
        status: SampleRejectedStatus,
    },
    SubscriptionMatched {
        the_reader: DataReaderAsync<()>,
        status: SubscriptionMatchedStatus,
    },
    RequestedIncompatibleQos {
        the_reader: DataReaderAsync<()>,
        status: RequestedIncompatibleQosStatus,
    },
}

impl MailHandler for DataReaderListenerActor {
    type Mail = DataReaderListenerMail;
    fn handle(&mut self, message: DataReaderListenerMail) {
        match message {
            DataReaderListenerMail::DataAvailable { the_reader } => {
                self.listener.trigger_on_data_available(the_reader)
            }
            DataReaderListenerMail::RequestedDeadlineMissed { the_reader, status } => {
                self.listener
                    .trigger_on_requested_deadline_missed(the_reader, status);
            }
            DataReaderListenerMail::SampleRejected { the_reader, status } => {
                self.listener.trigger_on_sample_rejected(the_reader, status)
            }
            DataReaderListenerMail::SubscriptionMatched { the_reader, status } => self
                .listener
                .trigger_on_subscription_matched(the_reader, status),
            DataReaderListenerMail::RequestedIncompatibleQos { the_reader, status } => {
                self.listener
                    .trigger_on_requested_incompatible_qos(the_reader, status);
            }
        }
    }
}
