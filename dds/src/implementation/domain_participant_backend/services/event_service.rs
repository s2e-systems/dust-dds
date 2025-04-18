use crate::{
    implementation::{
        domain_participant_backend::domain_participant_actor::DomainParticipantActor,
        listeners::{
            data_reader_listener, data_writer_listener, domain_participant_listener,
            publisher_listener, subscriber_listener,
        },
        status_condition::status_condition_actor,
    },
    infrastructure::{instance::InstanceHandle, status::StatusKind},
    runtime::actor::{ActorAddress, MailHandler},
};

pub struct RequestedDeadlineMissed {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub change_instance_handle: InstanceHandle,
    pub participant_address: ActorAddress<DomainParticipantActor>,
}
impl MailHandler<RequestedDeadlineMissed> for DomainParticipantActor {
    fn handle(&mut self, message: RequestedDeadlineMissed) {
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
        else {
            return;
        };
        let Some(data_reader) = subscriber.get_mut_data_reader(message.data_reader_handle) else {
            return;
        };
        data_reader.remove_instance_ownership(&message.change_instance_handle);
        data_reader.increment_requested_deadline_missed_status(message.change_instance_handle);

        if data_reader
            .listener_mask()
            .contains(&StatusKind::RequestedDeadlineMissed)
        {
            let status = data_reader.get_requested_deadline_missed_status();
            let Ok(the_reader) = self.get_data_reader_async(
                message.participant_address,
                message.subscriber_handle,
                message.data_reader_handle,
            ) else {
                return;
            };
            let Some(subscriber) = self
                .domain_participant
                .get_mut_subscriber(message.subscriber_handle)
            else {
                return;
            };
            let Some(data_reader) = subscriber.get_mut_data_reader(message.data_reader_handle)
            else {
                return;
            };
            if let Some(l) = data_reader.listener() {
                l.send_actor_mail(data_reader_listener::TriggerRequestedDeadlineMissed {
                    the_reader,
                    status,
                });
            }
        } else if subscriber
            .listener_mask()
            .contains(&StatusKind::RequestedDeadlineMissed)
        {
            let Ok(the_reader) = self.get_data_reader_async(
                message.participant_address,
                message.subscriber_handle,
                message.data_reader_handle,
            ) else {
                return;
            };

            let Some(subscriber) = self
                .domain_participant
                .get_mut_subscriber(message.subscriber_handle)
            else {
                return;
            };
            let Some(data_reader) = subscriber.get_mut_data_reader(message.data_reader_handle)
            else {
                return;
            };
            let status = data_reader.get_requested_deadline_missed_status();
            if let Some(l) = subscriber.listener() {
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
            let Ok(the_reader) = self.get_data_reader_async(
                message.participant_address,
                message.subscriber_handle,
                message.data_reader_handle,
            ) else {
                return;
            };

            let Some(subscriber) = self
                .domain_participant
                .get_mut_subscriber(message.subscriber_handle)
            else {
                return;
            };
            let Some(data_reader) = subscriber.get_mut_data_reader(message.data_reader_handle)
            else {
                return;
            };
            let status = data_reader.get_requested_deadline_missed_status();
            if let Some(l) = self.domain_participant.listener() {
                l.send_actor_mail(
                    domain_participant_listener::TriggerRequestedDeadlineMissed {
                        status,
                        the_reader,
                    },
                );
            }
        }
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
        else {
            return;
        };
        let Some(data_reader) = subscriber.get_mut_data_reader(message.data_reader_handle) else {
            return;
        };

        data_reader.status_condition().send_actor_mail(
            status_condition_actor::AddCommunicationState {
                state: StatusKind::RequestedDeadlineMissed,
            },
        );
    }
}

pub struct OfferedDeadlineMissed {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub change_instance_handle: InstanceHandle,
    pub participant_address: ActorAddress<DomainParticipantActor>,
}
impl MailHandler<OfferedDeadlineMissed> for DomainParticipantActor {
    fn handle(&mut self, message: OfferedDeadlineMissed) {
        let Some(publisher) = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
        else {
            return;
        };
        let Some(data_writer) = publisher.get_mut_data_writer(message.data_writer_handle) else {
            return;
        };

        data_writer.increment_offered_deadline_missed_status(message.change_instance_handle);

        if data_writer
            .listener_mask()
            .contains(&StatusKind::OfferedDeadlineMissed)
        {
            let status = data_writer.get_offered_deadline_missed_status();
            let Ok(the_writer) = self.get_data_writer_async(
                message.participant_address,
                message.publisher_handle,
                message.data_writer_handle,
            ) else {
                return;
            };

            let Some(publisher) = self
                .domain_participant
                .get_mut_publisher(message.publisher_handle)
            else {
                return;
            };
            let Some(data_writer) = publisher.get_mut_data_writer(message.data_writer_handle)
            else {
                return;
            };

            if let Some(l) = data_writer.listener() {
                l.send_actor_mail(data_writer_listener::TriggerOfferedDeadlineMissed {
                    the_writer,
                    status,
                });
            }
        } else if publisher
            .listener_mask()
            .contains(&StatusKind::OfferedDeadlineMissed)
        {
            let Ok(the_writer) = self.get_data_writer_async(
                message.participant_address,
                message.publisher_handle,
                message.data_writer_handle,
            ) else {
                return;
            };
            let Some(publisher) = self
                .domain_participant
                .get_mut_publisher(message.publisher_handle)
            else {
                return;
            };
            let Some(data_writer) = publisher.get_mut_data_writer(message.data_writer_handle)
            else {
                return;
            };
            let status = data_writer.get_offered_deadline_missed_status();
            if let Some(l) = publisher.listener() {
                l.send_actor_mail(publisher_listener::TriggerOfferedDeadlineMissed {
                    the_writer,
                    status,
                });
            }
        } else if self
            .domain_participant
            .listener_mask()
            .contains(&StatusKind::OfferedDeadlineMissed)
        {
            let Ok(the_writer) = self.get_data_writer_async(
                message.participant_address,
                message.publisher_handle,
                message.data_writer_handle,
            ) else {
                return;
            };

            let Some(publisher) = self
                .domain_participant
                .get_mut_publisher(message.publisher_handle)
            else {
                return;
            };
            let Some(data_writer) = publisher.get_mut_data_writer(message.data_writer_handle)
            else {
                return;
            };
            let status = data_writer.get_offered_deadline_missed_status();
            if let Some(l) = self.domain_participant.listener() {
                l.send_actor_mail(domain_participant_listener::TriggerOfferedDeadlineMissed {
                    the_writer,
                    status,
                });
            }
        }

        let Some(publisher) = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
        else {
            return;
        };
        let Some(data_writer) = publisher.get_mut_data_writer(message.data_writer_handle) else {
            return;
        };
        data_writer.status_condition().send_actor_mail(
            status_condition_actor::AddCommunicationState {
                state: StatusKind::OfferedDeadlineMissed,
            },
        );
    }
}
