use crate::{
    implementation::domain_participant_backend::domain_participant_actor::DomainParticipantActor,
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
    },
    runtime::actor::{Mail, MailHandler},
};

pub struct RequestedDeadlineMissed {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub change_instance_handle: InstanceHandle,
}
impl Mail for RequestedDeadlineMissed {
    type Result = DdsResult<()>;
}
impl MailHandler<RequestedDeadlineMissed> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: RequestedDeadlineMissed,
    ) -> <RequestedDeadlineMissed as Mail>::Result {
        let subscriber = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let data_reader = subscriber
            .get_mut_data_reader(message.data_reader_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        data_reader.remove_instance_ownership(&message.change_instance_handle);
        data_reader.increment_requested_deadline_missed_status(message.change_instance_handle);

        Ok(())
    }
}
