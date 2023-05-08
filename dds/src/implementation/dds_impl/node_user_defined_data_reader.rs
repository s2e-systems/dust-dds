use crate::{
    builtin_topics::PublicationBuiltinTopicData,
    implementation::{
        rtps::types::Guid,
        utils::shared_object::{DdsRwLock, DdsShared},
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind},
        status::{
            LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
            SampleLostStatus, SampleRejectedStatus, StatusKind, SubscriptionMatchedStatus,
        },
    },
    subscription::{
        data_reader::Sample,
        sample_info::{InstanceStateKind, SampleStateKind, ViewStateKind},
    },
    topic_definition::type_support::DdsDeserialize,
};

use super::{
    any_data_reader_listener::AnyDataReaderListener,
    dds_domain_participant::{AnnounceKind, DdsDomainParticipant},
    node_user_defined_subscriber::UserDefinedSubscriberNode,
    node_user_defined_topic::UserDefinedTopicNode,
    status_condition_impl::StatusConditionImpl,
};

#[derive(PartialEq, Eq, Debug)]
pub struct UserDefinedDataReaderNode {
    this: Guid,
    parent_subcriber: Guid,
    parent_participant: Guid,
}

impl UserDefinedDataReaderNode {
    pub fn new(this: Guid, parent_subcriber: Guid, parent_participant: Guid) -> Self {
        Self {
            this,
            parent_subcriber,
            parent_participant,
        }
    }

    pub fn guid(&self) -> DdsResult<Guid> {
        Ok(self.this)
    }

    pub fn read<Foo>(
        &self,
        domain_participant: &DdsDomainParticipant,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        domain_participant
            .get_subscriber(self.parent_subcriber)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_stateful_data_reader(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .read(
                max_samples,
                sample_states,
                view_states,
                instance_states,
                specific_instance_handle,
            )
    }

    pub fn take<Foo>(
        &self,
        domain_participant: &DdsDomainParticipant,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        domain_participant
            .get_subscriber(self.parent_subcriber)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_stateful_data_reader(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .take(
                max_samples,
                sample_states,
                view_states,
                instance_states,
                specific_instance_handle,
            )
    }

    pub fn read_next_instance<Foo>(
        &self,
        domain_participant: &DdsDomainParticipant,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        domain_participant
            .get_subscriber(self.parent_subcriber)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_stateful_data_reader(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .read_next_instance(
                max_samples,
                previous_handle,
                sample_states,
                view_states,
                instance_states,
            )
    }

    pub fn take_next_instance<Foo>(
        &self,
        domain_participant: &DdsDomainParticipant,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        domain_participant
            .get_subscriber(self.parent_subcriber)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_stateful_data_reader(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .take_next_instance(
                max_samples,
                previous_handle,
                sample_states,
                view_states,
                instance_states,
            )
    }

    pub fn get_key_value<Foo>(
        &self,
        domain_participant: &DdsDomainParticipant,
        key_holder: &mut Foo,
        handle: InstanceHandle,
    ) -> DdsResult<()> {
        domain_participant
            .get_subscriber(self.parent_subcriber)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_stateful_data_reader(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_key_value(key_holder, handle)
    }

    pub fn lookup_instance<Foo>(
        &self,
        domain_participant: &DdsDomainParticipant,
        instance: &Foo,
    ) -> DdsResult<Option<InstanceHandle>> {
        domain_participant
            .get_subscriber(self.parent_subcriber)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_stateful_data_reader(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .lookup_instance(instance)
    }

    pub fn get_liveliness_changed_status(
        &self,
        domain_participant: &DdsDomainParticipant,
    ) -> DdsResult<LivelinessChangedStatus> {
        Ok(domain_participant
            .get_subscriber(self.parent_subcriber)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_stateful_data_reader(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_liveliness_changed_status())
    }

    pub fn get_requested_deadline_missed_status(
        &self,
        domain_participant: &DdsDomainParticipant,
    ) -> DdsResult<RequestedDeadlineMissedStatus> {
        Ok(domain_participant
            .get_subscriber(self.parent_subcriber)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_stateful_data_reader(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_requested_deadline_missed_status())
    }

    pub fn get_requested_incompatible_qos_status(
        &self,
        domain_participant: &DdsDomainParticipant,
    ) -> DdsResult<RequestedIncompatibleQosStatus> {
        Ok(domain_participant
            .get_subscriber(self.parent_subcriber)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_stateful_data_reader(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_requested_incompatible_qos_status())
    }

    pub fn get_sample_lost_status(
        &self,
        domain_participant: &DdsDomainParticipant,
    ) -> DdsResult<SampleLostStatus> {
        Ok(domain_participant
            .get_subscriber(self.parent_subcriber)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_stateful_data_reader(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_sample_lost_status())
    }

    pub fn get_sample_rejected_status(
        &self,
        domain_participant: &DdsDomainParticipant,
    ) -> DdsResult<SampleRejectedStatus> {
        Ok(domain_participant
            .get_subscriber(self.parent_subcriber)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_stateful_data_reader(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_sample_rejected_status())
    }

    pub fn get_subscription_matched_status(
        &self,
        domain_participant: &DdsDomainParticipant,
    ) -> DdsResult<SubscriptionMatchedStatus> {
        Ok(domain_participant
            .get_subscriber(self.parent_subcriber)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_stateful_data_reader(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_subscription_matched_status())
    }

    pub fn get_topicdescription(
        &self,
        domain_participant: &DdsDomainParticipant,
    ) -> DdsResult<UserDefinedTopicNode> {
        let data_reader = domain_participant
            .get_subscriber(self.parent_subcriber)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_stateful_data_reader(self.this)
            .ok_or(DdsError::AlreadyDeleted)?;
        let topic = domain_participant
            .topic_list()
            .iter()
            .find(|t| {
                t.get_name() == data_reader.get_topic_name()
                    && t.get_type_name() == data_reader.get_type_name()
            })
            .cloned()
            .expect("Topic must exist");

        Ok(UserDefinedTopicNode::new(
            topic.guid(),
            self.parent_participant,
        ))
    }

    pub fn get_subscriber(&self) -> UserDefinedSubscriberNode {
        UserDefinedSubscriberNode::new(self.parent_subcriber, self.parent_participant)
    }

    pub fn is_historical_data_received(
        &self,
        domain_participant: &DdsDomainParticipant,
    ) -> DdsResult<bool> {
        domain_participant
            .get_subscriber(self.parent_subcriber)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_stateful_data_reader(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .is_historical_data_received()
    }

    pub fn get_matched_publication_data(
        &self,
        domain_participant: &DdsDomainParticipant,
        publication_handle: InstanceHandle,
    ) -> DdsResult<PublicationBuiltinTopicData> {
        domain_participant
            .get_subscriber(self.parent_subcriber)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_stateful_data_reader(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_matched_publication_data(publication_handle)
    }

    pub fn get_matched_publications(
        &self,
        domain_participant: &DdsDomainParticipant,
    ) -> DdsResult<Vec<InstanceHandle>> {
        Ok(domain_participant
            .get_subscriber(self.parent_subcriber)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_stateful_data_reader(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_matched_publications())
    }

    pub fn set_qos(
        &self,
        domain_participant: &DdsDomainParticipant,
        qos: QosKind<DataReaderQos>,
    ) -> DdsResult<()> {
        domain_participant
            .get_subscriber(self.parent_subcriber)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_stateful_data_reader(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .set_qos(qos)?;

        let data_reader = domain_participant
            .get_subscriber(self.parent_subcriber)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_stateful_data_reader(self.this)
            .ok_or(DdsError::AlreadyDeleted)?;
        if domain_participant
            .get_subscriber(self.parent_subcriber)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_stateful_data_reader(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .is_enabled()
        {
            let topic = domain_participant
                .topic_list()
                .iter()
                .find(|t| {
                    t.get_name() == data_reader.get_topic_name()
                        && t.get_type_name() == data_reader.get_type_name()
                })
                .cloned()
                .expect("Topic must exist");
            let subscriber_qos = domain_participant
                .get_subscriber(self.parent_subcriber)
                .ok_or(DdsError::AlreadyDeleted)?
                .get_qos();
            let discovered_reader_data = domain_participant
                .get_subscriber(self.parent_subcriber)
                .ok_or(DdsError::AlreadyDeleted)?
                .get_stateful_data_reader(self.this)
                .ok_or(DdsError::AlreadyDeleted)?
                .as_discovered_reader_data(&topic.get_qos(), &subscriber_qos);
            domain_participant
                .announce_sender()
                .send(AnnounceKind::CreatedDataReader(discovered_reader_data))
                .ok();
        }

        Ok(())
    }

    pub fn get_qos(&self, domain_participant: &DdsDomainParticipant) -> DdsResult<DataReaderQos> {
        Ok(domain_participant
            .get_subscriber(self.parent_subcriber)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_stateful_data_reader(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_qos())
    }

    pub fn set_listener(
        &self,
        _a_listener: Option<Box<dyn AnyDataReaderListener + Send + Sync>>,
        _mask: &[StatusKind],
    ) -> DdsResult<()> {
        todo!()
        // *self.0.get()?.get_status_listener_lock() = StatusListener::new(a_listener, mask);
        // Ok(())
    }

    pub fn get_statuscondition(
        &self,
        domain_participant: &DdsDomainParticipant,
    ) -> DdsResult<DdsShared<DdsRwLock<StatusConditionImpl>>> {
        Ok(domain_participant
            .get_subscriber(self.parent_subcriber)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_stateful_data_reader(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_statuscondition())
    }

    pub fn get_status_changes(
        &self,
        domain_participant: &DdsDomainParticipant,
    ) -> DdsResult<Vec<StatusKind>> {
        Ok(domain_participant
            .get_subscriber(self.parent_subcriber)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_stateful_data_reader(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_status_changes())
    }

    pub fn enable(&self, domain_participant: &mut DdsDomainParticipant) -> DdsResult<()> {
        if !domain_participant
            .get_subscriber(self.parent_subcriber)
            .ok_or(DdsError::AlreadyDeleted)?
            .is_enabled()
        {
            return Err(DdsError::PreconditionNotMet(
                "Parent subscriber disabled".to_string(),
            ));
        }

        domain_participant
            .get_subscriber(self.parent_subcriber)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_stateful_data_reader(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .enable()?;

        let data_reader = domain_participant
            .get_subscriber(self.parent_subcriber)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_stateful_data_reader(self.this)
            .ok_or(DdsError::AlreadyDeleted)?;
        let topic = domain_participant
            .topic_list()
            .iter()
            .find(|t| {
                t.get_name() == data_reader.get_topic_name()
                    && t.get_type_name() == data_reader.get_type_name()
            })
            .cloned()
            .expect("Topic must exist");
        let subscriber_qos = domain_participant
            .get_subscriber(self.parent_subcriber)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_qos();
        let discovered_reader_data = domain_participant
            .get_subscriber(self.parent_subcriber)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_stateful_data_reader(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .as_discovered_reader_data(&topic.get_qos(), &subscriber_qos);
        domain_participant
            .announce_sender()
            .send(AnnounceKind::CreatedDataReader(discovered_reader_data))
            .ok();

        Ok(())
    }

    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        Ok(self.this.into())
    }
}
