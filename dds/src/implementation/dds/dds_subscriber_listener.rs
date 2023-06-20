use crate::{
    implementation::{
        dds::nodes::{DataReaderNode, SubscriberNode, SubscriberNodeKind},
        utils::actor::{ActorAddress, CommandHandler},
    },
    infrastructure::{
        error::DdsResult,
        status::{
            RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleRejectedStatus,
        },
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

impl ActorAddress<DdsSubscriberListener> {
    pub fn trigger_on_data_on_readers(&self, the_subscriber: SubscriberNode) -> DdsResult<()> {
        struct TriggerOnDataOnReaders {
            the_subscriber: SubscriberNode,
        }

        impl CommandHandler<TriggerOnDataOnReaders> for DdsSubscriberListener {
            fn handle(&mut self, mail: TriggerOnDataOnReaders) {
                self.listener
                    .on_data_on_readers(&Subscriber::new(SubscriberNodeKind::Listener(
                        mail.the_subscriber,
                    )))
            }
        }

        self.send_command(TriggerOnDataOnReaders { the_subscriber })
    }

    pub fn trigger_on_sample_rejected(
        &self,
        the_reader: DataReaderNode,
        status: SampleRejectedStatus,
    ) -> DdsResult<()> {
        struct TriggerOnSampleRejected {
            the_reader: DataReaderNode,
            status: SampleRejectedStatus,
        }

        impl CommandHandler<TriggerOnSampleRejected> for DdsSubscriberListener {
            fn handle(&mut self, mail: TriggerOnSampleRejected) {
                self.listener
                    .on_sample_rejected(&mail.the_reader, mail.status)
            }
        }

        self.send_command(TriggerOnSampleRejected { the_reader, status })
    }

    pub fn trigger_on_requested_incompatible_qos(
        &self,
        the_reader: DataReaderNode,
        status: RequestedIncompatibleQosStatus,
    ) -> DdsResult<()> {
        struct TriggerOnRequestedIncompatibleQos {
            the_reader: DataReaderNode,
            status: RequestedIncompatibleQosStatus,
        }

        impl CommandHandler<TriggerOnRequestedIncompatibleQos> for DdsSubscriberListener {
            fn handle(&mut self, mail: TriggerOnRequestedIncompatibleQos) {
                self.listener
                    .on_requested_incompatible_qos(&mail.the_reader, mail.status)
            }
        }

        self.send_command(TriggerOnRequestedIncompatibleQos { the_reader, status })
    }

    pub fn trigger_on_requested_deadline_missed(
        &self,
        reader: DataReaderNode,
        status: RequestedDeadlineMissedStatus,
    ) -> DdsResult<()> {
        struct TriggerOnRequestedDeadlineMissed {
            reader: DataReaderNode,
            status: RequestedDeadlineMissedStatus,
        }

        impl CommandHandler<TriggerOnRequestedDeadlineMissed> for DdsSubscriberListener {
            fn handle(&mut self, mail: TriggerOnRequestedDeadlineMissed) {
                self.listener
                    .on_requested_deadline_missed(&mail.reader, mail.status)
            }
        }

        self.send_command(TriggerOnRequestedDeadlineMissed { reader, status })
    }
}
