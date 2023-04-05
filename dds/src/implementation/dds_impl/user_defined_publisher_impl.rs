use std::sync::mpsc::SyncSender;

use fnmatch_regex::glob_to_regex;

use crate::{
    domain::domain_participant_listener::DomainParticipantListener,
    implementation::{
        data_representation_builtin_endpoints::discovered_reader_data::DiscoveredReaderData,
        rtps::{
            group::RtpsGroup,
            messages::{
                overall_structure::RtpsMessageHeader,
                submessages::{AckNackSubmessage, NackFragSubmessage},
            },
            transport::TransportWrite,
            types::Locator,
        },
        utils::{
            condvar::DdsCondvar,
            shared_object::{DdsRwLock, DdsShared},
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind, TopicQos},
        status::StatusKind,
        time::{Duration, Time},
    },
    publication::publisher_listener::PublisherListener,
    topic_definition::type_support::DdsType,
};

use super::{
    any_data_writer_listener::AnyDataWriterListener,
    domain_participant_impl::AnnounceKind,
    message_receiver::{MessageReceiver, PublisherMessageReceiver},
    status_condition_impl::StatusConditionImpl,
    status_listener::StatusListener,
    topic_impl::TopicImpl,
    user_defined_data_writer_impl::UserDefinedDataWriterImpl,
    writer_factory::WriterFactory,
};

pub struct UserDefinedPublisherImpl {
    qos: DdsRwLock<PublisherQos>,
    rtps_group: RtpsGroup,
    data_writer_list: DdsRwLock<Vec<DdsShared<UserDefinedDataWriterImpl>>>,
    data_writer_factory: DdsRwLock<WriterFactory>,
    enabled: DdsRwLock<bool>,
    user_defined_data_send_condvar: DdsCondvar,
    status_listener: DdsRwLock<StatusListener<dyn PublisherListener + Send + Sync>>,
    data_max_size_serialized: usize,
    status_condition: DdsShared<DdsRwLock<StatusConditionImpl>>,
    announce_sender: SyncSender<AnnounceKind>,
}

impl UserDefinedPublisherImpl {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        qos: PublisherQos,
        rtps_group: RtpsGroup,
        listener: Option<Box<dyn PublisherListener + Send + Sync>>,
        mask: &[StatusKind],
        user_defined_data_send_condvar: DdsCondvar,
        data_max_size_serialized: usize,
        announce_sender: SyncSender<AnnounceKind>,
    ) -> DdsShared<Self> {
        DdsShared::new(UserDefinedPublisherImpl {
            qos: DdsRwLock::new(qos),
            rtps_group,
            data_writer_list: DdsRwLock::new(Vec::new()),
            data_writer_factory: DdsRwLock::new(WriterFactory::new()),
            enabled: DdsRwLock::new(false),
            user_defined_data_send_condvar,
            status_listener: DdsRwLock::new(StatusListener::new(listener, mask)),
            data_max_size_serialized,
            status_condition: DdsShared::new(DdsRwLock::new(StatusConditionImpl::default())),
            announce_sender,
        })
    }
}

impl DdsShared<UserDefinedPublisherImpl> {
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
        default_unicast_locator_list: &[Locator],
        default_multicast_locator_list: &[Locator],
    ) -> DdsResult<DdsShared<UserDefinedDataWriterImpl>>
    where
        Foo: DdsType,
    {
        let rtps_writer_impl = self.data_writer_factory.write_lock().create_writer(
            &self.rtps_group,
            Foo::has_key(),
            qos,
            default_unicast_locator_list,
            default_multicast_locator_list,
            self.data_max_size_serialized,
        )?;

        let data_writer_shared = UserDefinedDataWriterImpl::new(
            rtps_writer_impl,
            a_listener,
            mask,
            a_topic.clone(),
            self.downgrade(),
            self.user_defined_data_send_condvar.clone(),
            self.announce_sender.clone(),
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
            self.announce_sender
                .send(AnnounceKind::DeletedDataWriter(
                    data_writer.get_instance_handle(),
                ))
                .ok();
        }

        Ok(())
    }

    pub fn lookup_datawriter<Foo>(
        &self,
        topic: &DdsShared<TopicImpl>,
    ) -> DdsResult<DdsShared<UserDefinedDataWriterImpl>>
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
                self.announce_sender
                    .send(AnnounceKind::DeletedDataWriter(
                        data_writer.get_instance_handle(),
                    ))
                    .ok();
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
        *self.status_listener.write_lock() = StatusListener::new(a_listener, mask);
    }

    pub fn get_statuscondition(&self) -> DdsShared<DdsRwLock<StatusConditionImpl>> {
        self.status_condition.clone()
    }

    pub fn get_status_changes(&self) -> Vec<StatusKind> {
        self.status_condition.read_lock().get_status_changes()
    }

    pub fn enable(&self) -> DdsResult<()> {
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
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
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
                    &mut self.status_listener.write_lock(),
                    participant_status_listener,
                )
            }
        }
    }

    pub fn remove_matched_reader(
        &self,
        discovered_reader_handle: InstanceHandle,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) {
        for data_writer in self.data_writer_list.read_lock().iter() {
            data_writer.remove_matched_reader(
                discovered_reader_handle,
                &mut self.status_listener.write_lock(),
                participant_status_listener,
            )
        }
    }

    pub fn send_message(
        &self,
        header: RtpsMessageHeader,
        transport: &mut impl TransportWrite,
        now: Time,
    ) {
        for data_writer in self.data_writer_list.read_lock().iter() {
            data_writer.send_message(header, transport, now);
        }
    }
}

impl PublisherMessageReceiver for DdsShared<UserDefinedPublisherImpl> {
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
