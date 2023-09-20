use crate::{
    domain::domain_participant_listener::DomainParticipantListener,
    implementation::utils::actor::{Mail, MailHandler},
    infrastructure::status::{
        OfferedIncompatibleQosStatus, PublicationMatchedStatus, RequestedDeadlineMissedStatus,
        RequestedIncompatibleQosStatus, SampleLostStatus, SampleRejectedStatus,
        SubscriptionMatchedStatus,
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
impl MailHandler<TriggerOnSampleRejected> for DdsDomainParticipantListener {
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
impl MailHandler<TriggerOnRequestedIncompatibleQos> for DdsDomainParticipantListener {
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

pub struct TriggerOnOfferedIncompatibleQos {
    the_writer: DataWriterNode,
    status: OfferedIncompatibleQosStatus,
}

impl TriggerOnOfferedIncompatibleQos {
    pub fn new(the_writer: DataWriterNode, status: OfferedIncompatibleQosStatus) -> Self {
        Self { the_writer, status }
    }
}

impl Mail for TriggerOnOfferedIncompatibleQos {
    type Result = ();
}

#[async_trait::async_trait]
impl MailHandler<TriggerOnOfferedIncompatibleQos> for DdsDomainParticipantListener {
    async fn handle(
        &mut self,
        mail: TriggerOnOfferedIncompatibleQos,
    ) -> <TriggerOnOfferedIncompatibleQos as Mail>::Result {
        tokio::task::block_in_place(|| {
            self.listener
                .on_offered_incompatible_qos(&mail.the_writer, mail.status)
        });
    }
}

pub struct TriggerOnPublicationMatched {
    the_writer: DataWriterNode,
    status: PublicationMatchedStatus,
}

impl TriggerOnPublicationMatched {
    pub fn new(the_writer: DataWriterNode, status: PublicationMatchedStatus) -> Self {
        Self { the_writer, status }
    }
}

impl Mail for TriggerOnPublicationMatched {
    type Result = ();
}

#[async_trait::async_trait]
impl MailHandler<TriggerOnPublicationMatched> for DdsDomainParticipantListener {
    async fn handle(
        &mut self,
        mail: TriggerOnPublicationMatched,
    ) -> <TriggerOnPublicationMatched as Mail>::Result {
        tokio::task::block_in_place(|| {
            self.listener
                .on_publication_matched(&mail.the_writer, mail.status)
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
impl MailHandler<TriggerOnRequestedDeadlineMissed> for DdsDomainParticipantListener {
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
impl MailHandler<TriggerOnSubscriptionMatched> for DdsDomainParticipantListener {
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
impl MailHandler<TriggerOnSampleLost> for DdsDomainParticipantListener {
    async fn handle(&mut self, mail: TriggerOnSampleLost) -> <TriggerOnSampleLost as Mail>::Result {
        tokio::task::block_in_place(|| self.listener.on_sample_lost(&mail.the_reader, mail.status));
    }
}
