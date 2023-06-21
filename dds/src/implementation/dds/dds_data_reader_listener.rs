use crate::{
    implementation::utils::actor::{ActorAddress, CommandHandler},
    infrastructure::{
        error::DdsResult,
        status::{
            RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleRejectedStatus,
            SubscriptionMatchedStatus,
        },
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

impl DdsDataReaderListener {
    fn trigger_on_data_available(&mut self, reader: DataReaderNode) {
        self.listener.trigger_on_data_available(reader)
    }

    fn trigger_on_sample_rejected(&mut self, reader: DataReaderNode, status: SampleRejectedStatus) {
        self.listener.trigger_on_sample_rejected(reader, status)
    }

    fn trigger_on_requested_incompatible_qos(
        &mut self,
        reader: DataReaderNode,
        status: RequestedIncompatibleQosStatus,
    ) {
        self.listener
            .trigger_on_requested_incompatible_qos(reader, status)
    }

    fn trigger_on_subscription_matched(
        &mut self,
        reader: DataReaderNode,
        status: SubscriptionMatchedStatus,
    ) {
        self.listener
            .trigger_on_subscription_matched(reader, status)
    }
}

impl ActorAddress<DdsDataReaderListener> {
    pub fn trigger_on_data_available(&self, reader: DataReaderNode) -> DdsResult<()> {
        struct TriggerOnDataAvailable {
            reader: DataReaderNode,
        }

        impl CommandHandler<TriggerOnDataAvailable> for DdsDataReaderListener {
            fn handle(&mut self, mail: TriggerOnDataAvailable) {
                self.trigger_on_data_available(mail.reader)
            }
        }

        self.send_command(TriggerOnDataAvailable { reader })
    }

    pub fn trigger_on_sample_rejected(
        &self,
        reader: DataReaderNode,
        status: SampleRejectedStatus,
    ) -> DdsResult<()> {
        struct TriggerOnSampleRejected {
            reader: DataReaderNode,
            status: SampleRejectedStatus,
        }

        impl CommandHandler<TriggerOnSampleRejected> for DdsDataReaderListener {
            fn handle(&mut self, mail: TriggerOnSampleRejected) {
                self.trigger_on_sample_rejected(mail.reader, mail.status)
            }
        }

        self.send_command(TriggerOnSampleRejected { reader, status })
    }

    pub fn trigger_on_requested_incompatible_qos(
        &self,
        reader: DataReaderNode,
        status: RequestedIncompatibleQosStatus,
    ) -> DdsResult<()> {
        struct TriggerOnRequestedIncompatibleQos {
            reader: DataReaderNode,
            status: RequestedIncompatibleQosStatus,
        }

        impl CommandHandler<TriggerOnRequestedIncompatibleQos> for DdsDataReaderListener {
            fn handle(&mut self, mail: TriggerOnRequestedIncompatibleQos) {
                self.trigger_on_requested_incompatible_qos(mail.reader, mail.status)
            }
        }

        self.send_command(TriggerOnRequestedIncompatibleQos { reader, status })
    }

    pub fn trigger_on_subscription_matched(
        &self,
        reader: DataReaderNode,
        status: SubscriptionMatchedStatus,
    ) -> DdsResult<()> {
        struct TriggerOnSubscriptionMatched {
            reader: DataReaderNode,
            status: SubscriptionMatchedStatus,
        }

        impl CommandHandler<TriggerOnSubscriptionMatched> for DdsDataReaderListener {
            fn handle(&mut self, mail: TriggerOnSubscriptionMatched) {
                self.trigger_on_subscription_matched(mail.reader, mail.status)
            }
        }

        self.send_command(TriggerOnSubscriptionMatched { reader, status })
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

        impl CommandHandler<TriggerOnRequestedDeadlineMissed> for DdsDataReaderListener {
            fn handle(&mut self, mail: TriggerOnRequestedDeadlineMissed) {
                self.listener
                    .trigger_on_requested_deadline_missed(mail.reader, mail.status)
            }
        }

        self.send_command(TriggerOnRequestedDeadlineMissed { reader, status })
    }
}
