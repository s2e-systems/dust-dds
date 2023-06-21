use crate::{
    domain::domain_participant_listener::DomainParticipantListener,
    implementation::utils::actor::{ActorAddress, CommandHandler},
    infrastructure::{
        error::DdsResult,
        status::{
            OfferedIncompatibleQosStatus, PublicationMatchedStatus, RequestedDeadlineMissedStatus,
            RequestedIncompatibleQosStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
        },
    },
};

use super::nodes::{DataReaderNode, DataWriterNode};

pub struct DdsDomainParticipantListener {
    listener: Box<dyn DomainParticipantListener + Send>,
}

impl DdsDomainParticipantListener {
    pub fn new(listener: Box<dyn DomainParticipantListener + Send>) -> Self {
        Self { listener }
    }
}

impl ActorAddress<DdsDomainParticipantListener> {
    pub fn trigger_on_sample_rejected(
        &self,
        the_reader: DataReaderNode,
        status: SampleRejectedStatus,
    ) -> DdsResult<()> {
        struct TriggerOnSampleRejected {
            the_reader: DataReaderNode,
            status: SampleRejectedStatus,
        }

        impl CommandHandler<TriggerOnSampleRejected> for DdsDomainParticipantListener {
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

        impl CommandHandler<TriggerOnRequestedIncompatibleQos> for DdsDomainParticipantListener {
            fn handle(&mut self, mail: TriggerOnRequestedIncompatibleQos) {
                self.listener
                    .on_requested_incompatible_qos(&mail.the_reader, mail.status)
            }
        }

        self.send_command(TriggerOnRequestedIncompatibleQos { the_reader, status })
    }

    pub fn trigger_on_offered_incompatible_qos(
        &self,
        the_writer: DataWriterNode,
        status: OfferedIncompatibleQosStatus,
    ) -> DdsResult<()> {
        struct OnOfferedIncompatibleQos {
            the_writer: DataWriterNode,
            status: OfferedIncompatibleQosStatus,
        }

        impl CommandHandler<OnOfferedIncompatibleQos> for DdsDomainParticipantListener {
            fn handle(&mut self, mail: OnOfferedIncompatibleQos) {
                self.listener
                    .on_offered_incompatible_qos(&mail.the_writer, mail.status)
            }
        }

        self.send_command(OnOfferedIncompatibleQos { the_writer, status })
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

        impl CommandHandler<TriggerOnRequestedDeadlineMissed> for DdsDomainParticipantListener {
            fn handle(&mut self, mail: TriggerOnRequestedDeadlineMissed) {
                self.listener
                    .on_requested_deadline_missed(&mail.reader, mail.status)
            }
        }

        self.send_command(TriggerOnRequestedDeadlineMissed { reader, status })
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

        impl CommandHandler<TriggerOnSubscriptionMatched> for DdsDomainParticipantListener {
            fn handle(&mut self, mail: TriggerOnSubscriptionMatched) {
                self.listener
                    .on_subscription_matched(&mail.reader, mail.status)
            }
        }

        self.send_command(TriggerOnSubscriptionMatched { reader, status })
    }

    pub fn trigger_on_publication_matched(
        &self,
        the_writer: DataWriterNode,
        status: PublicationMatchedStatus,
    ) -> DdsResult<()> {
        struct OnPublicationMatched {
            the_writer: DataWriterNode,
            status: PublicationMatchedStatus,
        }

        impl CommandHandler<OnPublicationMatched> for DdsDomainParticipantListener {
            fn handle(&mut self, mail: OnPublicationMatched) {
                self.listener
                    .on_publication_matched(&mail.the_writer, mail.status)
            }
        }

        self.send_command(OnPublicationMatched { the_writer, status })
    }
}
