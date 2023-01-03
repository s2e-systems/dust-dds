use std::sync::atomic::{self, AtomicU8};

use crate::{
    implementation::{
        data_representation_builtin_endpoints::discovered_reader_data::DiscoveredReaderData,
        rtps::{
            endpoint::RtpsEndpoint,
            group::RtpsGroupImpl,
            messages::submessages::AckNackSubmessage,
            stateful_writer::RtpsStatefulWriter,
            transport::TransportWrite,
            types::{
                EntityId, EntityKey, Guid, Locator, TopicKind, USER_DEFINED_WRITER_NO_KEY,
                USER_DEFINED_WRITER_WITH_KEY,
            },
            writer::RtpsWriter,
        },
        utils::{
            condvar::DdsCondvar,
            shared_object::{DdsRwLock, DdsShared, DdsWeak},
        },
    },
    infrastructure::{
        condition::StatusCondition,
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind, TopicQos},
        status::StatusKind,
        time::{Duration, DURATION_ZERO},
    },
    publication::publisher_listener::PublisherListener,
    topic_definition::type_support::DdsType,
};

use super::{
    any_data_writer_listener::AnyDataWriterListener,
    domain_participant_impl::DomainParticipantImpl,
    message_receiver::{MessageReceiver, PublisherMessageReceiver},
    topic_impl::TopicImpl,
    user_defined_data_writer::UserDefinedDataWriter,
};

pub struct UserDefinedPublisher {
    qos: DdsRwLock<PublisherQos>,
    rtps_group: RtpsGroupImpl,
    data_writer_list: DdsRwLock<Vec<DdsShared<UserDefinedDataWriter>>>,
    user_defined_data_writer_counter: AtomicU8,
    default_datawriter_qos: DataWriterQos,
    enabled: DdsRwLock<bool>,
    user_defined_data_send_condvar: DdsCondvar,
    parent_participant: DdsWeak<DomainParticipantImpl>,
    listener: DdsRwLock<Option<Box<dyn PublisherListener + Send + Sync>>>,
    listener_status_mask: DdsRwLock<Vec<StatusKind>>,
}

impl UserDefinedPublisher {
    pub fn new(
        qos: PublisherQos,
        rtps_group: RtpsGroupImpl,
        listener: Option<Box<dyn PublisherListener + Send + Sync>>,
        mask: &[StatusKind],
        parent_participant: DdsWeak<DomainParticipantImpl>,
        user_defined_data_send_condvar: DdsCondvar,
    ) -> DdsShared<Self> {
        DdsShared::new(UserDefinedPublisher {
            qos: DdsRwLock::new(qos),
            rtps_group,
            data_writer_list: DdsRwLock::new(Vec::new()),
            user_defined_data_writer_counter: AtomicU8::new(0),
            default_datawriter_qos: DataWriterQos::default(),
            enabled: DdsRwLock::new(false),
            user_defined_data_send_condvar,
            parent_participant,
            listener: DdsRwLock::new(listener),
            listener_status_mask: DdsRwLock::new(mask.to_vec()),
        })
    }
}

impl DdsShared<UserDefinedPublisher> {
    pub fn is_empty(&self) -> bool {
        self.data_writer_list.read_lock().is_empty()
    }

    pub fn is_enabled(&self) -> bool {
        *self.enabled.read_lock()
    }

    pub fn create_datawriter<Foo>(
        &self,
        a_topic: &DdsShared<TopicImpl>,
        qos: QosKind<DataWriterQos>,
        a_listener: Option<Box<dyn AnyDataWriterListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<DdsShared<UserDefinedDataWriter>>
    where
        Foo: DdsType,
    {
        let topic_shared = a_topic;

        // /////// Build the GUID
        let guid = {
            let user_defined_data_writer_counter = self
                .user_defined_data_writer_counter
                .fetch_add(1, atomic::Ordering::SeqCst);

            let entity_kind = match Foo::has_key() {
                true => USER_DEFINED_WRITER_WITH_KEY,
                false => USER_DEFINED_WRITER_NO_KEY,
            };

            Guid::new(
                self.rtps_group.guid().prefix(),
                EntityId::new(
                    EntityKey::new([
                        <[u8; 3]>::from(self.rtps_group.guid().entity_id().entity_key())[0],
                        user_defined_data_writer_counter,
                        0,
                    ]),
                    entity_kind,
                ),
            )
        };

        // /////// Create data writer
        let data_writer_shared = {
            let qos = match qos {
                QosKind::Default => self.default_datawriter_qos.clone(),
                QosKind::Specific(q) => q,
            };
            qos.is_consistent()?;

            let topic_kind = match Foo::has_key() {
                true => TopicKind::WithKey,
                false => TopicKind::NoKey,
            };

            let rtps_writer_impl = RtpsStatefulWriter::new(RtpsWriter::new(
                RtpsEndpoint::new(
                    guid,
                    topic_kind,
                    self.get_participant().default_unicast_locator_list(),
                    self.get_participant().default_multicast_locator_list(),
                ),
                true,
                Duration::new(0, 200_000_000),
                DURATION_ZERO,
                DURATION_ZERO,
                None,
                qos,
            ));

            let data_writer_shared = UserDefinedDataWriter::new(
                rtps_writer_impl,
                a_listener,
                mask,
                topic_shared.clone(),
                self.downgrade(),
                self.user_defined_data_send_condvar.clone(),
            );

            self.data_writer_list
                .write_lock()
                .push(data_writer_shared.clone());

            data_writer_shared
        };

        if *self.enabled.read_lock()
            && self
                .qos
                .read_lock()
                .entity_factory
                .autoenable_created_entities
        {
            data_writer_shared.enable()?;
        }
        Ok(data_writer_shared)
    }

    pub fn delete_datawriter(&self, data_writer_handle: InstanceHandle) -> DdsResult<()> {
        let data_writer_list = &mut self.data_writer_list.write_lock();
        let data_writer_list_position = data_writer_list
            .iter()
            .position(|x| x.get_instance_handle() == data_writer_handle)
            .ok_or_else(|| {
                DdsError::PreconditionNotMet(
                    "Data writer can only be deleted from its parent publisher".to_string(),
                )
            })?;
        let data_writer = data_writer_list.remove(data_writer_list_position);

        // The writer creation is announced only on enabled so its deletion must be announced only if it is enabled
        if data_writer.is_enabled() {
            self.get_participant()
                .announce_deleted_datawriter(data_writer.as_discovered_writer_data())?;
        }

        Ok(())
    }

    pub fn lookup_datawriter<Foo>(
        &self,
        topic: &DdsShared<TopicImpl>,
    ) -> DdsResult<DdsShared<UserDefinedDataWriter>>
    where
        Foo: DdsType,
    {
        self.data_writer_list
            .write_lock()
            .iter()
            .find_map(|data_writer_shared| {
                let data_writer_topic = data_writer_shared.get_topic();

                if data_writer_topic.get_name() == topic.get_name()
                    && data_writer_topic.get_type_name() == Foo::type_name()
                {
                    Some(data_writer_shared.clone())
                } else {
                    None
                }
            })
            .ok_or_else(|| DdsError::PreconditionNotMet("Not found".to_string()))
    }

    pub fn suspend_publications(&self) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn resume_publications(&self) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn begin_coherent_changes(&self) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn end_coherent_changes(&self) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn wait_for_acknowledgments(&self, _max_wait: Duration) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn delete_contained_entities(&self) -> DdsResult<()> {
        for data_writer in self.data_writer_list.write_lock().drain(..) {
            // The writer creation is announced only on enabled so its deletion must be announced only if it is enabled
            if data_writer.is_enabled() {
                self.get_participant()
                    .announce_deleted_datawriter(data_writer.as_discovered_writer_data())?;
            }
        }

        Ok(())
    }

    pub fn set_default_datawriter_qos(&self, _qos: QosKind<DataWriterQos>) -> DdsResult<()> {
        todo!()
    }

    pub fn get_default_datawriter_qos(&self) -> DdsResult<DataWriterQos> {
        todo!()
    }

    pub fn copy_from_topic_qos(
        &self,
        _a_datawriter_qos: &mut DataWriterQos,
        _a_topic_qos: &TopicQos,
    ) -> DdsResult<()> {
        todo!()
    }

    pub fn get_participant(&self) -> DdsShared<DomainParticipantImpl> {
        self.parent_participant
            .upgrade()
            .expect("Parent participant of publisher must exist")
    }

    pub fn set_qos(&self, qos: QosKind<PublisherQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => Default::default(),
            QosKind::Specific(q) => q,
        };

        if *self.enabled.read_lock() {
            self.qos.read_lock().check_immutability(&qos)?;
        }

        *self.qos.write_lock() = qos;

        Ok(())
    }

    pub fn get_qos(&self) -> PublisherQos {
        self.qos.read_lock().clone()
    }

    pub fn set_listener(
        &self,
        a_listener: Option<Box<dyn PublisherListener + Send + Sync>>,
        mask: &[StatusKind],
    ) {
        *self.listener.write_lock() = a_listener;
        *self.listener_status_mask.write_lock() = mask.to_vec();
    }

    pub fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        todo!()
    }

    pub fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }

    pub fn enable(&self) -> DdsResult<()> {
        if !self.get_participant().is_enabled() {
            return Err(DdsError::PreconditionNotMet(
                "Parent participant is disabled".to_string(),
            ));
        }

        *self.enabled.write_lock() = true;

        if self
            .qos
            .read_lock()
            .entity_factory
            .autoenable_created_entities
        {
            for data_writer in self.data_writer_list.read_lock().iter() {
                data_writer.enable()?;
            }
        }

        Ok(())
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.rtps_group.guid().into()
    }

    pub fn add_matched_reader(
        &self,
        discovered_reader_data: &DiscoveredReaderData,
        default_unicast_locator_list: &[Locator],
        default_multicast_locator_list: &[Locator],
    ) {
        for data_writer in self.data_writer_list.read_lock().iter() {
            data_writer.add_matched_reader(
                discovered_reader_data,
                default_unicast_locator_list,
                default_multicast_locator_list,
            )
        }
    }

    pub fn remove_matched_reader(&self, discovered_reader_handle: InstanceHandle) {
        for data_writer in self.data_writer_list.read_lock().iter() {
            data_writer.remove_matched_reader(discovered_reader_handle)
        }
    }

    pub fn send_message(&self, transport: &mut impl TransportWrite) {
        for data_writer in self.data_writer_list.read_lock().iter() {
            data_writer.send_message(transport);
        }
    }

    pub fn on_publication_matched(&self, writer: &DdsShared<UserDefinedDataWriter>) {
        match self.listener.write_lock().as_mut() {
            Some(l)
                if self
                    .listener_status_mask
                    .read_lock()
                    .contains(&StatusKind::PublicationMatched) =>
            {
                let status = writer.get_publication_matched_status();
                l.on_publication_matched(writer, status);
            }
            _ => self.get_participant().on_publication_matched(writer),
        }
    }

    pub fn on_offered_incompatible_qos(&self, writer: &DdsShared<UserDefinedDataWriter>) {
        match self.listener.write_lock().as_mut() {
            Some(l)
                if self
                    .listener_status_mask
                    .read_lock()
                    .contains(&StatusKind::OfferedIncompatibleQos) =>
            {
                let status = writer.get_offered_incompatible_qos_status();
                l.on_offered_incompatible_qos(writer, status)
            }
            _ => self.get_participant().on_offered_incompatible_qos(writer),
        }
    }
}

impl PublisherMessageReceiver for DdsShared<UserDefinedPublisher> {
    fn on_acknack_submessage_received(
        &self,
        acknack_submessage: &AckNackSubmessage,
        message_receiver: &MessageReceiver,
    ) {
        for data_writer in self.data_writer_list.read_lock().iter() {
            data_writer.on_acknack_submessage_received(acknack_submessage, message_receiver);
        }
    }
}
