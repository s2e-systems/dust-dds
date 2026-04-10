use crate::{
    dcps::{
        channels::oneshot::OneshotSender, dcps_participant_factory::DcpsParticipantFactory,
        status_condition::StatusConditionEntity,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        status::StatusKind,
    },
    runtime::DdsRuntime,
};

use alloc::vec::Vec;

impl<R: DdsRuntime> DcpsParticipantFactory<R> {
    pub fn get_status_condition_enabled_statuses(
        &mut self,
        entity: StatusConditionEntity,
    ) -> DdsResult<Vec<StatusKind>> {
        match entity {
            StatusConditionEntity::Subscriber {
                participant_handle,
                subscriber_handle,
            } => {
                let dp = self.find_participant(participant_handle)?;
                let s = dp
                    .domain_participant
                    .user_defined_subscriber_list
                    .iter_mut()
                    .chain(core::iter::once(
                        &mut dp.domain_participant.builtin_subscriber,
                    ))
                    .find(|s| s.instance_handle == subscriber_handle)
                    .ok_or(DdsError::AlreadyDeleted)?;
                return Ok(s.status_condition.get_enabled_statuses());
            }
            StatusConditionEntity::Topic {
                participant_handle,
                topic_handle,
            } => {
                let dp = self.find_participant(participant_handle)?;
                for t in dp.domain_participant.topic_description_list.iter_mut() {
                    match t {
                        super::TopicDescriptionKind::Topic(topic_entity) => {
                            if topic_entity.instance_handle == topic_handle {
                                return Ok(topic_entity.status_condition.get_enabled_statuses());
                            }
                        }
                        super::TopicDescriptionKind::ContentFilteredTopic(_) => (),
                    }
                }
                return Err(DdsError::AlreadyDeleted);
            }
            StatusConditionEntity::DataWriter {
                participant_handle,
                publisher_handle,
                writer_handle,
            } => {
                let dp = self.find_participant(participant_handle)?;
                for p in dp
                    .domain_participant
                    .user_defined_publisher_list
                    .iter_mut()
                    .chain(core::iter::once(
                        &mut dp.domain_participant.builtin_publisher,
                    ))
                {
                    if p.instance_handle == publisher_handle {
                        for dw in p.data_writer_list.iter_mut() {
                            if dw.instance_handle == writer_handle {
                                return Ok(dw.status_condition.get_enabled_statuses());
                            }
                        }
                    }
                }
            }
            StatusConditionEntity::DataReader {
                participant_handle,
                subscriber_handle,
                reader_handle,
            } => {
                let dp = self.find_participant(participant_handle)?;
                for s in dp
                    .domain_participant
                    .user_defined_subscriber_list
                    .iter_mut()
                    .chain(core::iter::once(
                        &mut dp.domain_participant.builtin_subscriber,
                    ))
                {
                    if s.instance_handle == subscriber_handle {
                        for dr in s.data_reader_list.iter_mut() {
                            if dr.instance_handle == reader_handle {
                                return Ok(dr.status_condition.get_enabled_statuses());
                            }
                        }
                    }
                }
            }
        }

        Err(DdsError::AlreadyDeleted)
    }

    pub fn set_status_condition_enabled_statuses(
        &mut self,
        entity: StatusConditionEntity,
        status_mask: Vec<StatusKind>,
    ) -> DdsResult<()> {
        match entity {
            StatusConditionEntity::Subscriber {
                participant_handle,
                subscriber_handle,
            } => {
                let dp = self.find_participant(participant_handle)?;
                let s = dp
                    .domain_participant
                    .user_defined_subscriber_list
                    .iter_mut()
                    .chain(core::iter::once(
                        &mut dp.domain_participant.builtin_subscriber,
                    ))
                    .find(|s| s.instance_handle == subscriber_handle)
                    .ok_or(DdsError::AlreadyDeleted)?;
                s.status_condition.set_enabled_statuses(status_mask);
                return Ok(());
            }
            StatusConditionEntity::Topic {
                participant_handle,
                topic_handle,
            } => {
                let dp = self.find_participant(participant_handle)?;
                for t in dp.domain_participant.topic_description_list.iter_mut() {
                    match t {
                        super::TopicDescriptionKind::Topic(topic_entity) => {
                            if topic_entity.instance_handle == topic_handle {
                                topic_entity
                                    .status_condition
                                    .set_enabled_statuses(status_mask);
                                return Ok(());
                            }
                        }
                        super::TopicDescriptionKind::ContentFilteredTopic(_) => (),
                    }
                }
                return Err(DdsError::AlreadyDeleted);
            }
            StatusConditionEntity::DataWriter {
                participant_handle,
                publisher_handle,
                writer_handle,
            } => {
                let dp = self.find_participant(participant_handle)?;
                for p in dp
                    .domain_participant
                    .user_defined_publisher_list
                    .iter_mut()
                    .chain(core::iter::once(
                        &mut dp.domain_participant.builtin_publisher,
                    ))
                {
                    if p.instance_handle == publisher_handle {
                        for dw in p.data_writer_list.iter_mut() {
                            if dw.instance_handle == writer_handle {
                                dw.status_condition.set_enabled_statuses(status_mask);
                                return Ok(());
                            }
                        }
                    }
                }
            }
            StatusConditionEntity::DataReader {
                participant_handle,
                subscriber_handle,
                reader_handle,
            } => {
                let dp = self.find_participant(participant_handle)?;
                for s in dp
                    .domain_participant
                    .user_defined_subscriber_list
                    .iter_mut()
                    .chain(core::iter::once(
                        &mut dp.domain_participant.builtin_subscriber,
                    ))
                {
                    if s.instance_handle == subscriber_handle {
                        for dr in s.data_reader_list.iter_mut() {
                            if dr.instance_handle == reader_handle {
                                dr.status_condition.set_enabled_statuses(status_mask);
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }

        Err(DdsError::AlreadyDeleted)
    }

    pub fn get_status_condition_trigger_value(
        &mut self,
        entity: StatusConditionEntity,
    ) -> DdsResult<bool> {
        match entity {
            StatusConditionEntity::Subscriber {
                participant_handle,
                subscriber_handle,
            } => {
                let dp = self.find_participant(participant_handle)?;
                let s = dp
                    .domain_participant
                    .user_defined_subscriber_list
                    .iter_mut()
                    .chain(core::iter::once(
                        &mut dp.domain_participant.builtin_subscriber,
                    ))
                    .find(|s| s.instance_handle == subscriber_handle)
                    .ok_or(DdsError::AlreadyDeleted)?;
                return Ok(s.status_condition.get_trigger_value());
            }
            StatusConditionEntity::Topic {
                participant_handle,
                topic_handle,
            } => {
                let dp = self.find_participant(participant_handle)?;
                for t in dp.domain_participant.topic_description_list.iter_mut() {
                    match t {
                        super::TopicDescriptionKind::Topic(topic_entity) => {
                            if topic_entity.instance_handle == topic_handle {
                                return Ok(topic_entity.status_condition.get_trigger_value());
                            }
                        }
                        super::TopicDescriptionKind::ContentFilteredTopic(_) => (),
                    }
                }
                return Err(DdsError::AlreadyDeleted);
            }
            StatusConditionEntity::DataWriter {
                participant_handle,
                publisher_handle,
                writer_handle,
            } => {
                let dp = self.find_participant(participant_handle)?;
                for p in dp
                    .domain_participant
                    .user_defined_publisher_list
                    .iter_mut()
                    .chain(core::iter::once(
                        &mut dp.domain_participant.builtin_publisher,
                    ))
                {
                    if p.instance_handle == publisher_handle {
                        for dw in p.data_writer_list.iter_mut() {
                            if dw.instance_handle == writer_handle {
                                return Ok(dw.status_condition.get_trigger_value());
                            }
                        }
                    }
                }
            }
            StatusConditionEntity::DataReader {
                participant_handle,
                subscriber_handle,
                reader_handle,
            } => {
                let dp = self.find_participant(participant_handle)?;
                for s in dp
                    .domain_participant
                    .user_defined_subscriber_list
                    .iter_mut()
                    .chain(core::iter::once(
                        &mut dp.domain_participant.builtin_subscriber,
                    ))
                {
                    if s.instance_handle == subscriber_handle {
                        for dr in s.data_reader_list.iter_mut() {
                            if dr.instance_handle == reader_handle {
                                return Ok(dr.status_condition.get_trigger_value());
                            }
                        }
                    }
                }
            }
        }

        Err(DdsError::AlreadyDeleted)
    }

    pub fn register_notification(
        &mut self,
        entity: StatusConditionEntity,
        notification_sender: OneshotSender<()>,
    ) -> DdsResult<()> {
        match entity {
            StatusConditionEntity::Subscriber {
                participant_handle,
                subscriber_handle,
            } => {
                let dp = self.find_participant(participant_handle)?;
                let s = dp
                    .domain_participant
                    .user_defined_subscriber_list
                    .iter_mut()
                    .chain(core::iter::once(
                        &mut dp.domain_participant.builtin_subscriber,
                    ))
                    .find(|s| s.instance_handle == subscriber_handle)
                    .ok_or(DdsError::AlreadyDeleted)?;
                s.status_condition
                    .register_notification(notification_sender);
                return Ok(());
            }
            StatusConditionEntity::Topic {
                participant_handle,
                topic_handle,
            } => {
                let dp = self.find_participant(participant_handle)?;
                for t in dp.domain_participant.topic_description_list.iter_mut() {
                    match t {
                        super::TopicDescriptionKind::Topic(topic_entity) => {
                            if topic_entity.instance_handle == topic_handle {
                                topic_entity
                                    .status_condition
                                    .register_notification(notification_sender);
                                return Ok(());
                            }
                        }
                        super::TopicDescriptionKind::ContentFilteredTopic(_) => (),
                    }
                }
                return Err(DdsError::AlreadyDeleted);
            }
            StatusConditionEntity::DataWriter {
                participant_handle,
                publisher_handle,
                writer_handle,
            } => {
                let dp = self.find_participant(participant_handle)?;
                for p in dp
                    .domain_participant
                    .user_defined_publisher_list
                    .iter_mut()
                    .chain(core::iter::once(
                        &mut dp.domain_participant.builtin_publisher,
                    ))
                {
                    if p.instance_handle == publisher_handle {
                        for dw in p.data_writer_list.iter_mut() {
                            if dw.instance_handle == writer_handle {
                                dw.status_condition
                                    .register_notification(notification_sender);
                                return Ok(());
                            }
                        }
                    }
                }
            }
            StatusConditionEntity::DataReader {
                participant_handle,
                subscriber_handle,
                reader_handle,
            } => {
                let dp = self.find_participant(participant_handle)?;
                for s in dp
                    .domain_participant
                    .user_defined_subscriber_list
                    .iter_mut()
                    .chain(core::iter::once(
                        &mut dp.domain_participant.builtin_subscriber,
                    ))
                {
                    if s.instance_handle == subscriber_handle {
                        for dr in s.data_reader_list.iter_mut() {
                            if dr.instance_handle == reader_handle {
                                dr.status_condition
                                    .register_notification(notification_sender);
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }

        Err(DdsError::AlreadyDeleted)
    }
}
