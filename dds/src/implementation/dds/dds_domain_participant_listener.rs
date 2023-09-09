use crate::{
    domain::domain_participant_listener::DomainParticipantListener,
    implementation::utils::actor::actor_command_interface,
    infrastructure::status::{
        OfferedIncompatibleQosStatus, PublicationMatchedStatus, RequestedDeadlineMissedStatus,
        RequestedIncompatibleQosStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
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

actor_command_interface! {
impl DdsDomainParticipantListener {
    pub fn trigger_on_sample_rejected(
        &mut self,
        the_reader: DataReaderNode,
        status: SampleRejectedStatus,
    ) {
        self.listener.on_sample_rejected(&the_reader, status)
    }

    pub fn trigger_on_requested_incompatible_qos(
        &mut self,
        the_reader: DataReaderNode,
        status: RequestedIncompatibleQosStatus,
    ) {
        self.listener
            .on_requested_incompatible_qos(&the_reader, status)
    }

    pub fn trigger_on_offered_incompatible_qos(
        &mut self,
        the_writer: DataWriterNode,
        status: OfferedIncompatibleQosStatus,
    ) {
        self.listener
            .on_offered_incompatible_qos(&the_writer, status)
    }

    pub fn trigger_on_requested_deadline_missed(
        &mut self,
        reader: DataReaderNode,
        status: RequestedDeadlineMissedStatus,
    ) {
        self.listener.on_requested_deadline_missed(&reader, status)
    }

    pub fn trigger_on_subscription_matched(
        &mut self,
        reader: DataReaderNode,
        status: SubscriptionMatchedStatus,
    ) {
        self.listener.on_subscription_matched(&reader, status)
    }

    pub fn trigger_on_publication_matched(
        &mut self,
        the_writer: DataWriterNode,
        status: PublicationMatchedStatus,
    ) {
        self.listener.on_publication_matched(&the_writer, status)
    }
}
}
