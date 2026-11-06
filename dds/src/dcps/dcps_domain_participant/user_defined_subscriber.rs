use crate::{
    dcps::{
        channels::mpsc::MpscSender,
        dcps_domain_participant::{
            data_reader_entity::SampleList, subscriber_entity::SubscriberEntity,
            user_defined_data_reader::UserDefinedDataReader,
        },
        listeners::domain_participant_listener::ListenerMail,
        status_condition::DcpsStatusCondition,
        status_mask::StatusMask,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataReaderQos, SubscriberQos},
        qos_policy::PresentationQosPolicyAccessScopeKind,
        sample_info::{InstanceStateMask, SampleStateMask, ViewStateMask},
    },
};
use alloc::{string::String, sync::Arc, vec::Vec};

pub struct UserDefinedSubscriber {
    pub subscriber_entity: SubscriberEntity,
    pub default_data_reader_qos: DataReaderQos,
    pub status_condition: DcpsStatusCondition,
    pub listener_sender: Option<MpscSender<ListenerMail>>,
    pub listener_mask: StatusMask,
    pub data_reader_list: Vec<UserDefinedDataReader>,
    pub access_depth: usize,
}

impl UserDefinedSubscriber {
    pub fn new(
        instance_handle: InstanceHandle,
        qos: SubscriberQos,
        listener_sender: Option<MpscSender<ListenerMail>>,
        listener_mask: StatusMask,
    ) -> Self {
        Self {
            subscriber_entity: SubscriberEntity::new(instance_handle, qos),
            default_data_reader_qos: DataReaderQos::const_default(),
            status_condition: DcpsStatusCondition::default(),
            listener_sender,
            listener_mask,
            data_reader_list: Vec::new(),
            access_depth: 0,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn read(
        &mut self,
        data_reader_handle: &InstanceHandle,
        max_samples: i32,
        sample_state_mask: SampleStateMask,
        view_state_mask: ViewStateMask,
        instance_state_mask: InstanceStateMask,
        specific_instance_handle: &Option<InstanceHandle>,
    ) -> DdsResult<(SampleList, Arc<str>)> {
        if self.subscriber_entity.qos.presentation.access_scope
            == PresentationQosPolicyAccessScopeKind::Group
            && self.access_depth == 0
        {
            return Err(DdsError::PreconditionNotMet(String::from(
                "read called without calling begin_access on Subscriber with GROUP access_scope",
            )));
        }

        let Some(data_reader) = self
            .data_reader_list
            .iter_mut()
            .find(|x| &x.instance_handle == data_reader_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let sample_list = data_reader.read(
            max_samples,
            sample_state_mask,
            view_state_mask,
            instance_state_mask,
            specific_instance_handle,
            &self.subscriber_entity.qos.presentation,
        )?;
        Ok((sample_list, data_reader.topic_name.clone()))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn take(
        &mut self,
        data_reader_handle: &InstanceHandle,
        max_samples: i32,
        sample_state_mask: SampleStateMask,
        view_state_mask: ViewStateMask,
        instance_state_mask: InstanceStateMask,
        specific_instance_handle: &Option<InstanceHandle>,
    ) -> DdsResult<(SampleList, Arc<str>)> {
        if self.subscriber_entity.qos.presentation.access_scope
            == PresentationQosPolicyAccessScopeKind::Group
            && self.access_depth == 0
        {
            return Err(DdsError::PreconditionNotMet(String::from(
                "take called without calling begin_access on Subscriber with GROUP access_scope",
            )));
        }

        let Some(data_reader) = self
            .data_reader_list
            .iter_mut()
            .find(|x| &x.instance_handle == data_reader_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let sample_list = data_reader.take(
            max_samples,
            sample_state_mask,
            view_state_mask,
            instance_state_mask,
            specific_instance_handle,
            &self.subscriber_entity.qos.presentation,
        )?;
        Ok((sample_list, data_reader.topic_name.clone()))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn read_next_instance(
        &mut self,
        data_reader_handle: &InstanceHandle,
        max_samples: i32,
        previous_handle: &Option<InstanceHandle>,
        sample_state_mask: SampleStateMask,
        view_state_mask: ViewStateMask,
        instance_state_mask: InstanceStateMask,
    ) -> DdsResult<(SampleList, Arc<str>)> {
        if self.subscriber_entity.qos.presentation.access_scope
            == PresentationQosPolicyAccessScopeKind::Group
            && self.access_depth == 0
        {
            return Err(DdsError::PreconditionNotMet(String::from(
                "read_next_instance called without calling begin_access on Subscriber with GROUP access_scope",
            )));
        }

        let Some(data_reader) = self
            .data_reader_list
            .iter_mut()
            .find(|x| &x.instance_handle == data_reader_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let sample_list = data_reader.read_next_instance(
            max_samples,
            previous_handle,
            sample_state_mask,
            view_state_mask,
            instance_state_mask,
            &self.subscriber_entity.qos.presentation,
        )?;
        Ok((sample_list, data_reader.topic_name.clone()))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn take_next_instance(
        &mut self,
        data_reader_handle: &InstanceHandle,
        max_samples: i32,
        previous_handle: &Option<InstanceHandle>,
        sample_state_mask: SampleStateMask,
        view_state_mask: ViewStateMask,
        instance_state_mask: InstanceStateMask,
    ) -> DdsResult<(SampleList, Arc<str>)> {
        if self.subscriber_entity.qos.presentation.access_scope
            == PresentationQosPolicyAccessScopeKind::Group
            && self.access_depth == 0
        {
            return Err(DdsError::PreconditionNotMet(String::from(
                "take_next_instance called without calling begin_access on Subscriber with GROUP access_scope",
            )));
        }

        let Some(data_reader) = self
            .data_reader_list
            .iter_mut()
            .find(|x| &x.instance_handle == data_reader_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let sample_list = data_reader.take_next_instance(
            max_samples,
            previous_handle,
            sample_state_mask,
            view_state_mask,
            instance_state_mask,
            &self.subscriber_entity.qos.presentation,
        )?;
        Ok((sample_list, data_reader.topic_name.clone()))
    }
}
