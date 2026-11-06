use alloc::{string::String, sync::Arc};

use crate::{
    dcps::{
        dcps_domain_participant::{
            participant_entity::DcpsDomainParticipant,
            user_defined_data_writer::UserDefinedDataWriter,
        },
        listeners::{
            data_writer_listener::DcpsDataWriterListener, publisher_listener::DcpsPublisherListener,
        },
        status_mask::StatusMask,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind},
        qos_policy::PresentationQosPolicyAccessScopeKind,
        time::Time,
    },
    rtps::{
        cache_change::CacheChangeWrite,
        stateful_writer::RtpsStatefulWriter,
        types::{ChangeKind, TopicKind, WriterGroupInfo},
    },
    rtps_messages::submessage_elements::ParameterListWrite,
    runtime::Spawner,
    transport::types::{EntityId, Guid, USER_DEFINED_WRITER_NO_KEY, USER_DEFINED_WRITER_WITH_KEY},
};

impl DcpsDomainParticipant {
    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(self, dcps_listener, spawner, transport_participant))]
    pub fn create_data_writer(
        &mut self,
        publisher_handle: &InstanceHandle,
        topic_name: String,
        qos: QosKind<DataWriterQos>,
        dcps_listener: Option<DcpsDataWriterListener>,
        listener_mask: StatusMask,
        spawner: &impl Spawner,
        now: Time,
        enable_type_information: bool,
        transport_participant: &mut impl crate::transport::interface::RtpsParticipant,
    ) -> DdsResult<InstanceHandle> {
        let Some(topic) = self
            .domain_participant
            .locally_created_topic_list
            .iter()
            .find(|x| x.topic_name.as_ref() == topic_name.as_str())
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let topic_kind = self
            .domain_participant
            .type_register
            .get_dynamic_type(&topic.type_information.complete.typeid_with_size.type_id)
            .map(|dt| TopicKind::from(&dt))
            .unwrap_or(TopicKind::NoKey);

        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| &x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        if publisher.coherent_changes_depth > 0
            && publisher.qos.presentation.coherent_access
            && publisher.qos.presentation.access_scope == PresentationQosPolicyAccessScopeKind::Group
        {
            return Err(DdsError::PreconditionNotMet(String::from(
                "Cannot create data writer while a group coherent set is active",
            )));
        }

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
        let fragment_size = transport_participant.fragment_size();
        let transport_writer =
            RtpsStatefulWriter::new(Guid::new(guid.prefix(), entity_id), fragment_size);
        let key_holder_type = super::topic_entity::get_topic_type_support(
            &topic.topic_name,
            &self.domain_participant.content_filtered_topic_list,
            &self.domain_participant.locally_created_topic_list,
            &self.domain_participant.type_register,
        )
        .as_ref()
        .map(crate::dcps::xtypes_glue::key_and_instance_handle::KeyHolderType::new)
        .unwrap_or_default();

        let listener_sender = dcps_listener.map(|l| l.spawn(spawner));
        let mut data_writer = UserDefinedDataWriter::new(
            writer_handle,
            transport_writer,
            topic.topic_name.clone(),
            listener_sender,
            listener_mask,
            qos,
            key_holder_type,
        );
        if publisher.coherent_changes_depth > 0
            && publisher.qos.presentation.coherent_access
            && publisher.qos.presentation.access_scope
                != PresentationQosPolicyAccessScopeKind::Instance
        {
            data_writer.is_in_coherent_set = true;
        }
        let data_writer_handle = data_writer.instance_handle;

        publisher.data_writer_list.push(data_writer);

        if publisher.enabled && publisher.qos.entity_factory.autoenable_created_entities {
            self.enable_data_writer(
                publisher_handle,
                &writer_handle,
                now,
                enable_type_information,
                transport_participant,
            )?;
        }

        Ok(data_writer_handle)
    }

    #[tracing::instrument(skip(self, transport_participant))]
    pub fn delete_data_writer(
        &mut self,
        publisher_handle: &InstanceHandle,
        datawriter_handle: &InstanceHandle,
        now: Time,
        transport_participant: &mut impl crate::transport::interface::RtpsParticipant,
    ) -> DdsResult<()> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| &x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        if publisher.coherent_changes_depth > 0
            && publisher.qos.presentation.coherent_access
            && publisher.qos.presentation.access_scope == PresentationQosPolicyAccessScopeKind::Group
        {
            return Err(DdsError::PreconditionNotMet(String::from(
                "Cannot delete data writer while a group coherent set is active",
            )));
        }

        if let Some(index) = publisher
            .data_writer_list
            .iter()
            .position(|x| &x.instance_handle == datawriter_handle)
        {
            let data_writer = publisher.data_writer_list.remove(index);
            self.announce_deleted_data_writer(data_writer, now, transport_participant);
            Ok(())
        } else {
            Err(DdsError::AlreadyDeleted)
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn begin_coherent_changes(&mut self, publisher_handle: &InstanceHandle) -> DdsResult<()> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| &x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        publisher.coherent_changes_depth += 1;
        if publisher.coherent_changes_depth == 1 && publisher.qos.presentation.coherent_access {
            match publisher.qos.presentation.access_scope {
                PresentationQosPolicyAccessScopeKind::Instance
                | PresentationQosPolicyAccessScopeKind::Topic => {
                    for writer in &mut publisher.data_writer_list {
                        writer.is_in_coherent_set = true;
                        writer.first_coherent_sequence_number = None;
                    }
                }
                PresentationQosPolicyAccessScopeKind::Group => {
                    publisher.group_coherent_set = None;
                    for writer in &mut publisher.data_writer_list {
                        writer.is_in_coherent_set = true;
                        writer.first_coherent_sequence_number = None;
                    }
                }
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip(self, transport_participant))]
    pub fn end_coherent_changes(
        &mut self,
        publisher_handle: &InstanceHandle,
        now: Time,
        transport_participant: &mut impl crate::transport::interface::RtpsParticipant,
    ) -> DdsResult<()> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| &x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        if publisher.coherent_changes_depth == 0 {
            return Err(DdsError::PreconditionNotMet(String::from(
                "end_coherent_changes called without matching begin_coherent_changes",
            )));
        }

        publisher.coherent_changes_depth -= 1;
        if publisher.coherent_changes_depth == 0 {
            if publisher.qos.presentation.coherent_access {
                match publisher.qos.presentation.access_scope {
                    PresentationQosPolicyAccessScopeKind::Instance
                    | PresentationQosPolicyAccessScopeKind::Topic => {
                        for writer in &mut publisher.data_writer_list {
                            if writer.first_coherent_sequence_number.is_some() {
                                writer.last_change_sequence_number += 1;
                                let change = CacheChangeWrite {
                                    writer_guid: writer.transport_writer.guid(),
                                    sequence_number: writer.last_change_sequence_number,
                                    source_timestamp: now.into(),
                                    inline_qos: ParameterListWrite {
                                        kind: ChangeKind::Alive,
                                        instance_handle: None,
                                        coherent_set: None,
                                        group_coherent_set: None,
                                        group_seq_num: None,
                                        writer_group_info: None,
                                    },
                                    data_value: Arc::from([]),
                                };
                                writer.transport_writer.add_change(change);
                                writer
                                    .transport_writer
                                    .write_message(transport_participant, now);
                            }
                            writer.is_in_coherent_set = false;
                            writer.first_coherent_sequence_number = None;
                        }
                    }
                    PresentationQosPolicyAccessScopeKind::Group => {
                        if publisher.group_coherent_set.is_some() {
                            publisher.group_seq_num += 1;
                            let group_seq_num = publisher.group_seq_num;
                            let group_coherent_set = publisher.group_coherent_set;

                            let entity_ids: Vec<EntityId> = publisher
                                .data_writer_list
                                .iter()
                                .filter(|w| w.enabled)
                                .map(|w| w.transport_writer.guid().entity_id())
                                .collect();
                            let writer_group_info = Some(WriterGroupInfo::new(&entity_ids));

                            for writer in &mut publisher.data_writer_list {
                                writer.last_change_sequence_number += 1;
                                let change = CacheChangeWrite {
                                    writer_guid: writer.transport_writer.guid(),
                                    sequence_number: writer.last_change_sequence_number,
                                    source_timestamp: now.into(),
                                    inline_qos: ParameterListWrite {
                                        kind: ChangeKind::Alive,
                                        instance_handle: None,
                                        coherent_set: writer.first_coherent_sequence_number,
                                        group_coherent_set,
                                        group_seq_num: Some(group_seq_num),
                                        writer_group_info,
                                    },
                                    data_value: Arc::from([]),
                                };
                                writer.transport_writer.add_change(change);
                                writer
                                    .transport_writer
                                    .write_message(transport_participant, now);
                                writer.is_in_coherent_set = false;
                                writer.first_coherent_sequence_number = None;
                            }
                            publisher.group_coherent_set = None;
                        } else {
                            for writer in &mut publisher.data_writer_list {
                                writer.is_in_coherent_set = false;
                                writer.first_coherent_sequence_number = None;
                            }
                        }
                    }
                }
            } else {
                for writer in &mut publisher.data_writer_list {
                    writer.is_in_coherent_set = false;
                    writer.first_coherent_sequence_number = None;
                }
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_default_datawriter_qos(
        &mut self,
        publisher_handle: &InstanceHandle,
    ) -> DdsResult<DataWriterQos> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| &x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        Ok(publisher.default_datawriter_qos.clone())
    }

    #[tracing::instrument(skip(self))]
    pub fn set_default_datawriter_qos(
        &mut self,
        publisher_handle: &InstanceHandle,
        qos: QosKind<DataWriterQos>,
    ) -> DdsResult<()> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| &x.instance_handle == publisher_handle)
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
        publisher_handle: &InstanceHandle,
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
            .find(|x| &x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        publisher.qos = qos;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_publisher_qos(
        &mut self,
        publisher_handle: &InstanceHandle,
    ) -> DdsResult<PublisherQos> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| &x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(publisher.qos.clone())
    }

    #[tracing::instrument(skip(self, dcps_listener, spawner))]
    pub fn set_publisher_listener(
        &mut self,
        publisher_handle: &InstanceHandle,
        dcps_listener: Option<DcpsPublisherListener>,
        listener_mask: StatusMask,
        spawner: &impl Spawner,
    ) -> DdsResult<()> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| &x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let listener_sender = dcps_listener.map(|l| l.spawn(spawner));
        publisher.listener_sender = listener_sender;
        publisher.listener_mask = listener_mask;
        Ok(())
    }
}
