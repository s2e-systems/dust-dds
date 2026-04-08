use alloc::{string::String, vec::Vec};

use crate::{
    dcps::{
        actor::{Actor, ActorAddress},
        dcps_domain_participant::{
            DataWriterEntity, DcpsDomainParticipant, RtpsWriterKind, TopicDescriptionKind,
            get_topic_kind,
        },
        listeners::{
            data_writer_listener::DcpsDataWriterListener, publisher_listener::DcpsPublisherListener,
        },
        status_condition::DcpsStatusCondition,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind},
        status::StatusKind,
    },
    rtps::stateful_writer::RtpsStatefulWriter,
    runtime::DdsRuntime,
    transport::types::{
        EntityId, Guid, TopicKind, USER_DEFINED_WRITER_NO_KEY, USER_DEFINED_WRITER_WITH_KEY,
    },
};

impl<R: DdsRuntime> DcpsDomainParticipant<R> {
    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(self, dcps_listener))]
    pub async fn create_data_writer(
        &mut self,
        publisher_handle: InstanceHandle,
        topic_name: String,
        qos: QosKind<DataWriterQos>,
        dcps_listener: Option<DcpsDataWriterListener>,
        mask: Vec<StatusKind>,
    ) -> DdsResult<(InstanceHandle, ActorAddress<DcpsStatusCondition>)> {
        let Some(TopicDescriptionKind::Topic(topic)) = self
            .domain_participant
            .topic_description_list
            .iter()
            .find(|x| x.topic_name() == topic_name)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let topic_kind = get_topic_kind(topic.type_support);
        let type_support = topic.type_support;
        let type_name = topic.type_name.clone();

        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let entity_kind = match topic_kind {
            TopicKind::WithKey => USER_DEFINED_WRITER_WITH_KEY,
            TopicKind::NoKey => USER_DEFINED_WRITER_NO_KEY,
        };

        let entity_id = EntityId::new(
            [
                publisher.instance_handle[12],
                self.writer_counter.to_le_bytes()[0],
                self.writer_counter.to_le_bytes()[1],
            ],
            entity_kind,
        );

        let writer_handle = InstanceHandle::new([
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
        self.writer_counter += 1;

        let status_condition =
            Actor::spawn::<R>(DcpsStatusCondition::default(), &self.spawner_handle);
        let writer_status_condition_address = status_condition.address();
        let qos = match qos {
            QosKind::Default => publisher.default_datawriter_qos.clone(),
            QosKind::Specific(q) => {
                if q.is_consistent().is_ok() {
                    q
                } else {
                    return Err(DdsError::InconsistentPolicy);
                }
            }
        };
        let guid = Guid::from(*self.domain_participant.instance_handle.as_ref());
        let transport_writer = RtpsStatefulWriter::new(
            Guid::new(guid.prefix(), entity_id),
            self.transport.fragment_size,
        );
        let listener_sender = dcps_listener.map(|l| l.spawn::<R>(&self.spawner_handle));
        let data_writer = DataWriterEntity::new(
            writer_handle,
            RtpsWriterKind::Stateful(transport_writer),
            topic_name,
            type_name,
            type_support,
            status_condition,
            listener_sender,
            mask,
            qos,
        );
        let data_writer_handle = data_writer.instance_handle;

        publisher.data_writer_list.push(data_writer);

        if publisher.enabled && publisher.qos.entity_factory.autoenable_created_entities {
            self.enable_data_writer(publisher_handle, writer_handle)
                .await?;
        }

        Ok((data_writer_handle, writer_status_condition_address))
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete_data_writer(
        &mut self,
        publisher_handle: InstanceHandle,
        datawriter_handle: InstanceHandle,
    ) -> DdsResult<()> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        if let Some(index) = publisher
            .data_writer_list
            .iter()
            .position(|x| x.instance_handle == datawriter_handle)
        {
            let data_writer = publisher.data_writer_list.remove(index);
            self.announce_deleted_data_writer(data_writer).await;
            Ok(())
        } else {
            return Err(DdsError::AlreadyDeleted);
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn get_default_datawriter_qos(
        &mut self,
        publisher_handle: InstanceHandle,
    ) -> DdsResult<DataWriterQos> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        Ok(publisher.default_datawriter_qos.clone())
    }

    #[tracing::instrument(skip(self))]
    pub fn set_default_datawriter_qos(
        &mut self,
        publisher_handle: InstanceHandle,
        qos: QosKind<DataWriterQos>,
    ) -> DdsResult<()> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let qos = match qos {
            QosKind::Default => DataWriterQos::default(),
            QosKind::Specific(q) => q,
        };
        qos.is_consistent()?;
        publisher.default_datawriter_qos = qos;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn set_publisher_qos(
        &mut self,
        publisher_handle: InstanceHandle,
        qos: QosKind<PublisherQos>,
    ) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => self.domain_participant.default_publisher_qos.clone(),
            QosKind::Specific(q) => q,
        };
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        publisher.qos = qos;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_publisher_qos(
        &mut self,
        publisher_handle: InstanceHandle,
    ) -> DdsResult<PublisherQos> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(publisher.qos.clone())
    }

    #[tracing::instrument(skip(self, dcps_listener))]
    pub fn set_publisher_listener(
        &mut self,
        publisher_handle: InstanceHandle,
        dcps_listener: Option<DcpsPublisherListener>,
        mask: Vec<StatusKind>,
    ) -> DdsResult<()> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let listener_sender = dcps_listener.map(|l| l.spawn::<R>(&self.spawner_handle));
        publisher.listener_sender = listener_sender;
        publisher.listener_mask = mask;
        Ok(())
    }
}
