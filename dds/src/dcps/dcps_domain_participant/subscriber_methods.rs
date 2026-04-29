use alloc::{string::String, vec::Vec};

use crate::{
    dcps::{
        dcps_domain_participant::{
            DataReaderEntity, DcpsDomainParticipant, RtpsReaderKind, SubscriberEntity,
            TopicDescriptionKind, get_topic_kind,
        },
        listeners::{
            data_reader_listener::DcpsDataReaderListener,
            subscriber_listener::DcpsSubscriberListener,
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind, SubscriberQos},
        qos_policy::ReliabilityQosPolicyKind,
        status::StatusKind,
    },
    rtps::stateful_reader::RtpsStatefulReader,
    runtime::DdsRuntime,
    transport::types::{
        EntityId, Guid, ReliabilityKind, TopicKind, USER_DEFINED_READER_NO_KEY,
        USER_DEFINED_READER_WITH_KEY,
    },
};

impl DcpsDomainParticipant {
    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(self, dcps_listener, runtime))]
    pub fn create_data_reader(
        &mut self,
        subscriber_handle: &InstanceHandle,
        topic_name: String,
        qos: QosKind<DataReaderQos>,
        dcps_listener: Option<DcpsDataReaderListener>,
        mask: Vec<StatusKind>,
        runtime: &impl DdsRuntime,
    ) -> DdsResult<InstanceHandle> {
        let Some(topic) = self
            .domain_participant
            .topic_description_list
            .iter()
            .find(|x| x.topic_name() == topic_name)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let topic = match topic {
            TopicDescriptionKind::Topic(t) => t,
            TopicDescriptionKind::ContentFilteredTopic(content_filtered_topic) => {
                if let Some(TopicDescriptionKind::Topic(topic)) = self
                    .domain_participant
                    .topic_description_list
                    .iter()
                    .find(|x| x.topic_name() == content_filtered_topic.related_topic_name)
                {
                    topic
                } else {
                    return Err(DdsError::AlreadyDeleted);
                }
            }
        };

        let topic_kind = get_topic_kind(topic.type_support);

        let type_support = topic.type_support;
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| &x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let qos = match qos {
            QosKind::Default => subscriber.default_data_reader_qos.clone(),
            QosKind::Specific(q) => {
                if q.is_consistent().is_ok() {
                    q
                } else {
                    return Err(DdsError::InconsistentPolicy);
                }
            }
        };

        let entity_kind = match topic_kind {
            TopicKind::NoKey => USER_DEFINED_READER_NO_KEY,
            TopicKind::WithKey => USER_DEFINED_READER_WITH_KEY,
        };
        let entity_id = EntityId::new(
            [
                subscriber.instance_handle[12],
                self.reader_counter.to_ne_bytes()[0],
                self.reader_counter.to_ne_bytes()[1],
            ],
            entity_kind,
        );
        let reader_handle = InstanceHandle::new([
            self.domain_participant.instance_handle[0],
            self.domain_participant.instance_handle[1],
            self.domain_participant.instance_handle[2],
            self.domain_participant.instance_handle[3],
            self.domain_participant.instance_handle[4],
            self.domain_participant.instance_handle[5],
            self.domain_participant.instance_handle[6],
            self.domain_participant.instance_handle[7],
            self.domain_participant.instance_handle[8],
            self.domain_participant.instance_handle[9],
            self.domain_participant.instance_handle[10],
            self.domain_participant.instance_handle[11],
            entity_id.entity_key()[0],
            entity_id.entity_key()[1],
            entity_id.entity_key()[2],
            entity_id.entity_kind(),
        ]);
        self.reader_counter += 1;
        let reliablity_kind = match qos.reliability.kind {
            ReliabilityQosPolicyKind::BestEffort => ReliabilityKind::BestEffort,
            ReliabilityQosPolicyKind::Reliable => ReliabilityKind::Reliable,
        };
        let guid_prefix = Guid::from(*self.domain_participant.instance_handle.as_ref()).prefix();
        let guid = Guid::new(guid_prefix, entity_id);

        let transport_reader =
            RtpsReaderKind::Stateful(RtpsStatefulReader::new(guid, reliablity_kind));

        let listener_mask = mask.to_vec();
        let listener_sender = dcps_listener.map(|l| l.spawn(&runtime.spawner()));
        let data_reader = DataReaderEntity::new(
            reader_handle,
            qos,
            topic_name,
            type_support,
            listener_sender,
            listener_mask,
            transport_reader,
        );

        let data_reader_handle = data_reader.instance_handle;

        subscriber.data_reader_list.push(data_reader);

        if subscriber.enabled && subscriber.qos.entity_factory.autoenable_created_entities {
            self.enable_data_reader(subscriber_handle, &data_reader_handle, runtime)?;
        }
        Ok(data_reader_handle)
    }

    #[tracing::instrument(skip(self, runtime))]
    pub fn delete_data_reader(
        &mut self,
        subscriber_handle: &InstanceHandle,
        datareader_handle: &InstanceHandle,
        runtime: &impl DdsRuntime,
    ) -> DdsResult<()> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x: &&mut SubscriberEntity| &x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        if let Some(index) = subscriber
            .data_reader_list
            .iter()
            .position(|x| &x.instance_handle == datareader_handle)
        {
            let data_reader = subscriber.data_reader_list.remove(index);
            self.announce_deleted_data_reader(data_reader, runtime);
        } else {
            return Err(DdsError::AlreadyDeleted);
        };
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn lookup_data_reader(
        &mut self,
        subscriber_handle: &InstanceHandle,
        topic_name: String,
    ) -> DdsResult<Option<InstanceHandle>> {
        if !self
            .domain_participant
            .topic_description_list
            .iter()
            .any(|x| x.topic_name() == topic_name)
        {
            return Err(DdsError::BadParameter);
        }

        // Built-in subscriber is identified by the handle of the participant itself
        if &self.domain_participant.instance_handle == subscriber_handle {
            Ok(self
                .domain_participant
                .builtin_subscriber
                .data_reader_list
                .iter_mut()
                .find(|dr| dr.topic_name == topic_name)
                .map(|x| x.instance_handle))
        } else {
            let Some(s) = self
                .domain_participant
                .user_defined_subscriber_list
                .iter_mut()
                .find(|x| &x.instance_handle == subscriber_handle)
            else {
                return Err(DdsError::AlreadyDeleted);
            };
            Ok(s.data_reader_list
                .iter_mut()
                .find(|dr| dr.topic_name == topic_name)
                .map(|x| x.instance_handle))
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn set_default_data_reader_qos(
        &mut self,
        subscriber_handle: &InstanceHandle,
        qos: QosKind<DataReaderQos>,
    ) -> DdsResult<()> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| &x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let qos = match qos {
            QosKind::Default => DataReaderQos::default(),
            QosKind::Specific(q) => q,
        };
        qos.is_consistent()?;
        subscriber.default_data_reader_qos = qos;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_default_data_reader_qos(
        &mut self,
        subscriber_handle: &InstanceHandle,
    ) -> DdsResult<DataReaderQos> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| &x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(subscriber.default_data_reader_qos.clone())
    }

    #[tracing::instrument(skip(self))]
    pub fn set_subscriber_qos(
        &mut self,
        subscriber_handle: &InstanceHandle,
        qos: QosKind<SubscriberQos>,
    ) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => self.domain_participant.default_subscriber_qos.clone(),
            QosKind::Specific(q) => q,
        };

        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| &x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        if subscriber.enabled {
            subscriber.qos.check_immutability(&qos)?;
        }
        subscriber.qos = qos;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_subscriber_qos(
        &mut self,
        subscriber_handle: &InstanceHandle,
    ) -> DdsResult<SubscriberQos> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| &x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(subscriber.qos.clone())
    }

    #[tracing::instrument(skip(self, listener_sender_task, runtime))]
    pub fn set_subscriber_listener(
        &mut self,
        subscriber_handle: &InstanceHandle,
        listener_sender_task: Option<DcpsSubscriberListener>,
        mask: Vec<StatusKind>,
        runtime: &impl DdsRuntime,
    ) -> DdsResult<()> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| &x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let listener_sender = listener_sender_task.map(|l| l.spawn(&runtime.spawner()));
        subscriber.listener_sender = listener_sender;
        subscriber.listener_mask = mask;
        Ok(())
    }
}
