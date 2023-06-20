use crate::{
    domain::domain_participant_listener::DomainParticipantListener,
    implementation::utils::actor::{ActorAddress, CommandHandler},
    infrastructure::{error::DdsResult, status::SampleRejectedStatus},
};

use super::nodes::DataReaderNode;

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
}
