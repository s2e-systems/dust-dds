use crate::{
    implementation::{
        domain_participant_backend::domain_participant_actor::DomainParticipantActor,
        listeners::{data_reader_listener, domain_participant_listener, subscriber_listener},
        status_condition::status_condition_actor,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        status::StatusKind,
    },
    runtime::actor::{ActorAddress, Mail, MailHandler},
};

pub struct RequestedDeadlineMissed {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub change_instance_handle: InstanceHandle,
    pub participant_address: ActorAddress<DomainParticipantActor>,
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

        if data_reader
            .listener_mask()
            .contains(&StatusKind::RequestedDeadlineMissed)
        {
            let status = data_reader.get_requested_deadline_missed_status();
            let the_reader = self.get_data_reader_async(
                message.participant_address,
                message.subscriber_handle,
                message.data_reader_handle,
            )?;
            if let Some(l) = self
                .domain_participant
                .get_mut_subscriber(message.subscriber_handle)
                .ok_or(DdsError::AlreadyDeleted)?
                .get_mut_data_reader(message.data_reader_handle)
                .ok_or(DdsError::AlreadyDeleted)?
                .listener()
            {
                l.send_actor_mail(data_reader_listener::TriggerOnRequestedDeadlineMissed {
                    the_reader,
                    status,
                });
            }
        } else if self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .listener_mask()
            .contains(&StatusKind::RequestedDeadlineMissed)
        {
            let the_reader = self.get_data_reader_async(
                message.participant_address,
                message.subscriber_handle,
                message.data_reader_handle,
            )?;

            let status = self
                .domain_participant
                .get_mut_subscriber(message.subscriber_handle)
                .ok_or(DdsError::AlreadyDeleted)?
                .get_mut_data_reader(message.data_reader_handle)
                .ok_or(DdsError::AlreadyDeleted)?
                .get_requested_deadline_missed_status();
            if let Some(l) = self
                .domain_participant
                .get_mut_subscriber(message.subscriber_handle)
                .ok_or(DdsError::AlreadyDeleted)?
                .listener()
            {
                l.send_actor_mail(subscriber_listener::TriggerRequestedDeadlineMissed {
                    status,
                    the_reader,
                });
            }
        } else if self
            .domain_participant
            .listener_mask()
            .contains(&StatusKind::RequestedDeadlineMissed)
        {
            let the_reader = self.get_data_reader_async(
                message.participant_address,
                message.subscriber_handle,
                message.data_reader_handle,
            )?;

            let status = self
                .domain_participant
                .get_mut_subscriber(message.subscriber_handle)
                .ok_or(DdsError::AlreadyDeleted)?
                .get_mut_data_reader(message.data_reader_handle)
                .ok_or(DdsError::AlreadyDeleted)?
                .get_requested_deadline_missed_status();
            if let Some(l) = self.domain_participant.listener() {
                l.send_actor_mail(
                    domain_participant_listener::TriggerRequestedDeadlineMissed {
                        status,
                        the_reader,
                    },
                );
            }
        }

        self.domain_participant
            .get_mut_subscriber(message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_mut_data_reader(message.data_reader_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .status_condition()
            .send_actor_mail(status_condition_actor::AddCommunicationState {
                state: StatusKind::RequestedDeadlineMissed,
            });

        Ok(())
    }
}

pub struct OfferedDeadlineMissed {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub change_instance_handle: InstanceHandle,
}
impl Mail for OfferedDeadlineMissed {
    type Result = DdsResult<()>;
}
impl MailHandler<OfferedDeadlineMissed> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: OfferedDeadlineMissed,
    ) -> <OfferedDeadlineMissed as Mail>::Result {
        let publisher = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let data_writer = publisher
            .get_mut_data_writer(message.data_writer_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        data_writer.increment_offered_deadline_missed_status(message.change_instance_handle);

        Ok(())
    }
}
