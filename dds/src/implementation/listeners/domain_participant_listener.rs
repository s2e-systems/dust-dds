use std::thread::JoinHandle;

use crate::{
    dds_async::{
        data_reader::DataReaderAsync, domain_participant_listener::DomainParticipantListenerAsync,
        publisher::PublisherAsync, subscriber::SubscriberAsync, topic::TopicAsync,
    },
    implementation::{
        domain_participant_backend::entities::{
            data_reader::DataReaderEntity, data_writer::DataWriterEntity,
        },
        status_condition::status_condition_actor::StatusConditionActor,
    },
    infrastructure::status::{
        LivelinessChangedStatus, LivelinessLostStatus, OfferedDeadlineMissedStatus,
        OfferedIncompatibleQosStatus, PublicationMatchedStatus, RequestedDeadlineMissedStatus,
        RequestedIncompatibleQosStatus, SampleLostStatus, SampleRejectedStatus,
        SubscriptionMatchedStatus,
    },
    runtime::{
        actor::{ActorAddress, Mail, MailHandler},
        executor::block_on,
    },
};

pub struct DomainParticipantListenerActor {
    listener: Box<dyn DomainParticipantListenerAsync + Send>,
}

impl DomainParticipantListenerActor {
    pub fn new(listener: Box<dyn DomainParticipantListenerAsync + Send>) -> Self {
        Self { listener }
    }
}

pub struct TriggerRequestedDeadlineMissed {
    pub the_reader: DataReaderAsync<()>,
    pub status: RequestedDeadlineMissedStatus,
}
impl Mail for TriggerRequestedDeadlineMissed {
    type Result = ();
}
impl MailHandler<TriggerRequestedDeadlineMissed> for DomainParticipantListenerActor {
    fn handle(
        &mut self,
        message: TriggerRequestedDeadlineMissed,
    ) -> <TriggerRequestedDeadlineMissed as Mail>::Result {
        block_on(
            self.listener
                .on_requested_deadline_missed(message.the_reader, message.status),
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
impl MailHandler<TriggerSampleRejected> for DomainParticipantListenerActor {
    fn handle(
        &mut self,
        message: TriggerSampleRejected,
    ) -> <TriggerSampleRejected as Mail>::Result {
        block_on(
            self.listener
                .on_sample_rejected(message.the_reader, message.status),
        );
    }
}
