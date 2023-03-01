use fnmatch_regex::glob_to_regex;

use crate::{
    implementation::{
        data_representation_builtin_endpoints::discovered_reader_data::DiscoveredReaderData,
        rtps::{
            group::RtpsGroupImpl,
            messages::{
                overall_structure::RtpsMessageHeader,
                submessages::{AckNackSubmessage, NackFragSubmessage},
            },
            transport::TransportWrite,
            types::Locator,
        },
        utils::{
            condvar::DdsCondvar,
            shared_object::{DdsRwLock, DdsShared, DdsWeak},
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind, TopicQos},
        status::StatusKind,
        time::Duration,
    },
    publication::publisher_listener::PublisherListener,
    topic_definition::type_support::DdsType,
};

use super::{
    any_data_writer_listener::AnyDataWriterListener,
    writer_factory::WriterFactory,
    domain_participant_impl::DomainParticipantImpl,
    message_receiver::{MessageReceiver, PublisherMessageReceiver},
    status_condition_impl::StatusConditionImpl,
    topic_impl::TopicImpl,
    user_defined_data_writer::UserDefinedDataWriter,
};

pub struct UserDefinedPublisher {
    qos: DdsRwLock<PublisherQos>,
    rtps_group: RtpsGroupImpl,
    data_writer_list: DdsRwLock<Vec<DdsShared<UserDefinedDataWriter>>>,
    data_writer_factory: DdsRwLock<WriterFactory>,
    enabled: DdsRwLock<bool>,
    user_defined_data_send_condvar: DdsCondvar,
    parent_participant: DdsWeak<DomainParticipantImpl>,
    listener: DdsRwLock<Option<Box<dyn PublisherListener + Send + Sync>>>,
    listener_status_mask: DdsRwLock<Vec<StatusKind>>,
    data_max_size_serialized: usize,
    status_condition: DdsShared<DdsRwLock<StatusConditionImpl>>,
}

impl UserDefinedPublisher {
    pub fn new(
        qos: PublisherQos,
        rtps_group: RtpsGroupImpl,
        listener: Option<Box<dyn PublisherListener + Send + Sync>>,
        mask: &[StatusKind],
        parent_participant: DdsWeak<DomainParticipantImpl>,
        user_defined_data_send_condvar: DdsCondvar,
        data_max_size_serialized: usize,
    ) -> DdsShared<Self> {
        DdsShared::new(UserDefinedPublisher {
            qos: DdsRwLock::new(qos),
            rtps_group,
            data_writer_list: DdsRwLock::new(Vec::new()),
            data_writer_factory: DdsRwLock::new(WriterFactory::new()),
            enabled: DdsRwLock::new(false),
            user_defined_data_send_condvar,
            parent_participant,
            listener: DdsRwLock::new(listener),
            listener_status_mask: DdsRwLock::new(mask.to_vec()),
            data_max_size_serialized,
            status_condition: DdsShared::new(DdsRwLock::new(StatusConditionImpl::default())),
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
        let rtps_writer_impl = self.data_writer_factory.write_lock().create_writer(
            &self.rtps_group,
            Foo::has_key(),
            qos,
            self.get_participant().default_unicast_locator_list(),
            self.get_participant().default_multicast_locator_list(),
            self.data_max_size_serialized,
        )?;

        let data_writer_shared = UserDefinedDataWriter::new(
            rtps_writer_impl,
            a_listener,
            mask,
            a_topic.clone(),
            self.downgrade(),
            self.user_defined_data_send_condvar.clone(),
        );

        self.data_writer_list
            .write_lock()
            .push(data_writer_shared.clone());

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

    pub fn set_default_datawriter_qos(&self, qos: QosKind<DataWriterQos>) -> DdsResult<()> {
        self.data_writer_factory
            .write_lock()
            .set_default_datawriter_qos(qos)
    }

    pub fn get_default_datawriter_qos(&self) -> DataWriterQos {
        self.data_writer_factory
            .read_lock()
            .get_default_datawriter_qos()
            .clone()
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

    pub fn get_statuscondition(&self) -> DdsShared<DdsRwLock<StatusConditionImpl>> {
        self.status_condition.clone()
    }

    pub fn get_status_changes(&self) -> Vec<StatusKind> {
        self.status_condition.read_lock().get_status_changes()
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
        let is_discovered_reader_regex_matched_to_publisher = if let Ok(d) = glob_to_regex(
            &discovered_reader_data
                .subscription_builtin_topic_data
                .partition
                .name,
        ) {
            d.is_match(&self.qos.read_lock().partition.name)
        } else {
            false
        };

        let is_publisher_regex_matched_to_discovered_reader =
            if let Ok(d) = glob_to_regex(&self.qos.read_lock().partition.name) {
                d.is_match(
                    &discovered_reader_data
                        .subscription_builtin_topic_data
                        .partition
                        .name,
                )
            } else {
                false
            };

        let is_partition_string_matched = discovered_reader_data
            .subscription_builtin_topic_data
            .partition
            .name
            == self.qos.read_lock().partition.name;

        if is_discovered_reader_regex_matched_to_publisher
            || is_publisher_regex_matched_to_discovered_reader
            || is_partition_string_matched
        {
            for data_writer in self.data_writer_list.read_lock().iter() {
                data_writer.add_matched_reader(
                    discovered_reader_data,
                    default_unicast_locator_list,
                    default_multicast_locator_list,
                )
            }
        }
    }

    pub fn remove_matched_reader(&self, discovered_reader_handle: InstanceHandle) {
        for data_writer in self.data_writer_list.read_lock().iter() {
            data_writer.remove_matched_reader(discovered_reader_handle)
        }
    }

    pub fn send_message(&self, header: RtpsMessageHeader, transport: &mut impl TransportWrite) {
        for data_writer in self.data_writer_list.read_lock().iter() {
            data_writer.send_message(header, transport);
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

    fn on_nack_frag_submessage_received(
        &self,
        nackfrag_submessage: &NackFragSubmessage,
        message_receiver: &MessageReceiver,
    ) {
        for data_writer in self.data_writer_list.read_lock().iter() {
            data_writer.on_nack_frag_submessage_received(nackfrag_submessage, message_receiver);
        }
    }
}
