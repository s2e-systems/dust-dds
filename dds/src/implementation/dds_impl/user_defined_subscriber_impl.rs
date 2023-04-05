use std::sync::mpsc::SyncSender;

use fnmatch_regex::glob_to_regex;

use crate::{
    domain::domain_participant_listener::DomainParticipantListener,
    implementation::{
        data_representation_builtin_endpoints::discovered_writer_data::DiscoveredWriterData,
        rtps::{
            group::RtpsGroup,
            messages::{
                overall_structure::RtpsMessageHeader,
                submessages::{
                    DataFragSubmessage, DataSubmessage, GapSubmessage, HeartbeatFragSubmessage,
                    HeartbeatSubmessage,
                },
            },
            transport::TransportWrite,
            types::{GuidPrefix, Locator},
        },
        utils::{
            condvar::DdsCondvar,
            shared_object::{DdsRwLock, DdsShared},
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind, SubscriberQos, TopicQos},
        status::{SampleLostStatus, StatusKind},
        time::Time,
    },
    subscription::{subscriber::Subscriber, subscriber_listener::SubscriberListener},
    topic_definition::type_support::{DdsDeserialize, DdsType},
};

use super::{
    any_data_reader_listener::AnyDataReaderListener,
    domain_participant_impl::AnnounceKind,
    entity_kind::SubscriberNodeKind,
    listener_subscriber::ListenerSubscriberNode,
    message_receiver::{MessageReceiver, SubscriberSubmessageReceiver},
    reader_factory::ReaderFactory,
    status_condition_impl::StatusConditionImpl,
    status_listener::StatusListener,
    topic_impl::TopicImpl,
    user_defined_data_reader_impl::{
        UserDefinedDataReaderImpl, UserDefinedReaderDataSubmessageReceivedResult,
    },
};

pub struct UserDefinedSubscriberImpl {
    qos: DdsRwLock<SubscriberQos>,
    rtps_group: RtpsGroup,
    data_reader_list: DdsRwLock<Vec<DdsShared<UserDefinedDataReaderImpl>>>,
    reader_factory: DdsRwLock<ReaderFactory>,
    enabled: DdsRwLock<bool>,
    user_defined_data_send_condvar: DdsCondvar,
    status_condition: DdsShared<DdsRwLock<StatusConditionImpl>>,
    data_on_readers_status_changed_flag: DdsRwLock<bool>,
    status_listener: DdsRwLock<StatusListener<dyn SubscriberListener + Send + Sync>>,
    announce_sender: SyncSender<AnnounceKind>,
}

impl UserDefinedSubscriberImpl {
    pub fn new(
        qos: SubscriberQos,
        rtps_group: RtpsGroup,
        listener: Option<Box<dyn SubscriberListener + Send + Sync>>,
        mask: &[StatusKind],
        user_defined_data_send_condvar: DdsCondvar,
        announce_sender: SyncSender<AnnounceKind>,
    ) -> DdsShared<Self> {
        DdsShared::new(UserDefinedSubscriberImpl {
            qos: DdsRwLock::new(qos),
            rtps_group,
            data_reader_list: DdsRwLock::new(Vec::new()),
            reader_factory: DdsRwLock::new(ReaderFactory::new()),
            enabled: DdsRwLock::new(false),
            user_defined_data_send_condvar,
            status_condition: DdsShared::new(DdsRwLock::new(StatusConditionImpl::default())),
            data_on_readers_status_changed_flag: DdsRwLock::new(false),
            status_listener: DdsRwLock::new(StatusListener::new(listener, mask)),
            announce_sender,
        })
    }

    pub fn copy_from_topic_qos(
        _a_datareader_qos: &mut DataReaderQos,
        _a_topic_qos: &TopicQos,
    ) -> DdsResult<()> {
        todo!()
    }
}

impl DdsShared<UserDefinedSubscriberImpl> {
    pub fn is_enabled(&self) -> bool {
        *self.enabled.read_lock()
    }

    pub fn is_empty(&self) -> bool {
        self.data_reader_list.read_lock().is_empty()
    }

    pub fn create_datareader<Foo>(
        &self,
        a_topic: &DdsShared<TopicImpl>,
        qos: QosKind<DataReaderQos>,
        a_listener: Option<Box<dyn AnyDataReaderListener + Send + Sync>>,
        mask: &[StatusKind],
        default_unicast_locator_list: &[Locator],
        default_multicast_locator_list: &[Locator],
    ) -> DdsResult<DdsShared<UserDefinedDataReaderImpl>>
    where
        Foo: DdsType + for<'de> DdsDeserialize<'de>,
    {
        let rtps_reader = self.reader_factory.write_lock().create_reader::<Foo>(
            &self.rtps_group,
            Foo::has_key(),
            qos,
            default_unicast_locator_list,
            default_multicast_locator_list,
        )?;

        let data_reader_shared = UserDefinedDataReaderImpl::new(
            rtps_reader,
            a_topic.clone(),
            a_listener,
            mask,
            self.downgrade(),
            self.user_defined_data_send_condvar.clone(),
            self.announce_sender.clone(),
        );

        self.data_reader_list
            .write_lock()
            .push(data_reader_shared.clone());

        if *self.enabled.read_lock()
            && self
                .qos
                .read_lock()
                .entity_factory
                .autoenable_created_entities
        {
            data_reader_shared.enable()?;
        }

        Ok(data_reader_shared)
    }

    pub fn delete_datareader(&self, a_datareader_handle: InstanceHandle) -> DdsResult<()> {
        let data_reader_list = &mut self.data_reader_list.write_lock();
        let data_reader_list_position = data_reader_list
            .iter()
            .position(|x| x.get_instance_handle() == a_datareader_handle)
            .ok_or_else(|| {
                DdsError::PreconditionNotMet(
                    "Data reader can only be deleted from its parent subscriber".to_string(),
                )
            })?;

        let data_reader = data_reader_list.remove(data_reader_list_position);

        if data_reader.is_enabled() {
            self.announce_sender
                .send(AnnounceKind::DeletedDataReader(
                    data_reader.get_instance_handle(),
                ))
                .ok();
        }

        Ok(())
    }

    pub fn lookup_datareader<Foo>(
        &self,
        topic_name: &str,
    ) -> DdsResult<DdsShared<UserDefinedDataReaderImpl>>
    where
        Foo: DdsType,
    {
        let data_reader_list = &self.data_reader_list.write_lock();

        data_reader_list
            .iter()
            .find_map(|data_reader_shared| {
                let data_reader_topic = data_reader_shared.get_topicdescription();

                if data_reader_topic.get_name() == topic_name
                    && data_reader_topic.get_type_name() == Foo::type_name()
                {
                    Some(data_reader_shared.clone())
                } else {
                    None
                }
            })
            .ok_or_else(|| DdsError::PreconditionNotMet("Not found".to_string()))
    }

    pub fn notify_datareaders(&self) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn get_sample_lost_status(&self) -> DdsResult<SampleLostStatus> {
        todo!()
    }

    pub fn delete_contained_entities(&self) -> DdsResult<()> {
        for data_reader in self.data_reader_list.write_lock().drain(..) {
            if data_reader.is_enabled() {
                self.announce_sender
                    .send(AnnounceKind::DeletedDataReader(
                        data_reader.get_instance_handle(),
                    ))
                    .ok();
            }
        }

        Ok(())
    }

    pub fn set_default_datareader_qos(&self, qos: QosKind<DataReaderQos>) -> DdsResult<()> {
        self.reader_factory
            .write_lock()
            .set_default_datareader_qos(qos)
    }

    pub fn get_default_datareader_qos(&self) -> DataReaderQos {
        self.reader_factory
            .read_lock()
            .get_default_datareader_qos()
            .clone()
    }

    pub fn update_communication_status(
        &self,
        now: Time,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) {
        for data_reader in self.data_reader_list.read_lock().iter() {
            data_reader.update_communication_status(
                now,
                &mut self.status_listener.write_lock(),
                participant_status_listener,
            );
        }
    }

    pub fn set_qos(&self, qos: QosKind<SubscriberQos>) -> DdsResult<()> {
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

    pub fn get_qos(&self) -> SubscriberQos {
        self.qos.read_lock().clone()
    }

    pub fn set_listener(
        &self,
        a_listener: Option<Box<dyn SubscriberListener + Send + Sync>>,
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
            for data_reader in self.data_reader_list.read_lock().iter() {
                data_reader.enable()?;
            }
        }

        Ok(())
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.rtps_group.guid().into()
    }

    pub fn add_matched_writer(
        &self,
        discovered_writer_data: &DiscoveredWriterData,
        default_unicast_locator_list: &[Locator],
        default_multicast_locator_list: &[Locator],
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) {
        let is_discovered_writer_regex_matched_to_subscriber = if let Ok(d) = glob_to_regex(
            &discovered_writer_data
                .publication_builtin_topic_data
                .partition
                .name,
        ) {
            d.is_match(&self.qos.read_lock().partition.name)
        } else {
            false
        };

        let is_subscriber_regex_matched_to_discovered_writer =
            if let Ok(d) = glob_to_regex(&self.qos.read_lock().partition.name) {
                d.is_match(
                    &discovered_writer_data
                        .publication_builtin_topic_data
                        .partition
                        .name,
                )
            } else {
                false
            };

        let is_partition_string_matched = discovered_writer_data
            .publication_builtin_topic_data
            .partition
            .name
            == self.qos.read_lock().partition.name;

        if is_discovered_writer_regex_matched_to_subscriber
            || is_subscriber_regex_matched_to_discovered_writer
            || is_partition_string_matched
        {
            for data_reader in self.data_reader_list.read_lock().iter() {
                data_reader.add_matched_writer(
                    discovered_writer_data,
                    default_unicast_locator_list,
                    default_multicast_locator_list,
                    &mut self.status_listener.write_lock(),
                    participant_status_listener,
                )
            }
        }
    }

    pub fn remove_matched_writer(
        &self,
        discovered_writer_handle: InstanceHandle,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) {
        for data_reader in self.data_reader_list.read_lock().iter() {
            data_reader.remove_matched_writer(
                discovered_writer_handle,
                &mut self.status_listener.write_lock(),
                participant_status_listener,
            )
        }
    }

    pub fn send_message(&self, header: RtpsMessageHeader, transport: &mut impl TransportWrite) {
        for data_reader in self.data_reader_list.read_lock().iter() {
            data_reader.send_message(header, transport);
        }
    }

    fn on_data_on_readers(
        &self,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) {
        self.trigger_on_data_on_readers_listener(
            &mut self.status_listener.write_lock(),
            participant_status_listener,
        );

        self.status_condition
            .write_lock()
            .add_communication_state(StatusKind::DataOnReaders);
    }

    fn trigger_on_data_on_readers_listener(
        &self,
        subscriber_status_listener: &mut StatusListener<dyn SubscriberListener + Send + Sync>,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) {
        let data_on_readers_status_kind = &StatusKind::DataOnReaders;

        if subscriber_status_listener.is_enabled(data_on_readers_status_kind) {
            subscriber_status_listener
                .listener_mut()
                .on_data_on_readers(&Subscriber::new(SubscriberNodeKind::Listener(
                    ListenerSubscriberNode::new(),
                )))
        } else if participant_status_listener.is_enabled(data_on_readers_status_kind) {
            participant_status_listener
                .listener_mut()
                .on_data_on_readers(&Subscriber::new(SubscriberNodeKind::Listener(
                    ListenerSubscriberNode::new(),
                )))
        } else {
            for data_reader in self.data_reader_list.read_lock().iter() {
                data_reader.on_data_available(participant_status_listener);
            }
        }
    }
}

impl SubscriberSubmessageReceiver for DdsShared<UserDefinedSubscriberImpl> {
    fn on_heartbeat_submessage_received(
        &self,
        heartbeat_submessage: &HeartbeatSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        for data_reader in self.data_reader_list.read_lock().iter() {
            data_reader.on_heartbeat_submessage_received(heartbeat_submessage, source_guid_prefix)
        }
    }

    fn on_heartbeat_frag_submessage_received(
        &self,
        heartbeat_frag_submessage: &HeartbeatFragSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        for data_reader in self.data_reader_list.read_lock().iter() {
            data_reader.on_heartbeat_frag_submessage_received(
                heartbeat_frag_submessage,
                source_guid_prefix,
            )
        }
    }

    fn on_data_submessage_received(
        &self,
        data_submessage: &DataSubmessage<'_>,
        message_receiver: &MessageReceiver,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) {
        for data_reader in self.data_reader_list.read_lock().iter() {
            let data_submessage_received_result = data_reader.on_data_submessage_received(
                data_submessage,
                message_receiver,
                &mut self.status_listener.write_lock(),
                participant_status_listener,
            );
            match data_submessage_received_result {
                UserDefinedReaderDataSubmessageReceivedResult::NoChange => (),
                UserDefinedReaderDataSubmessageReceivedResult::NewDataAvailable => {
                    *self.data_on_readers_status_changed_flag.write_lock() = true
                }
            }
        }
        if *self.data_on_readers_status_changed_flag.read_lock() {
            self.on_data_on_readers(participant_status_listener);
        }
    }

    fn on_data_frag_submessage_received(
        &self,
        data_frag_submessage: &DataFragSubmessage<'_>,
        message_receiver: &MessageReceiver,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) {
        for data_reader in self.data_reader_list.read_lock().iter() {
            let data_submessage_received_result = data_reader.on_data_frag_submessage_received(
                data_frag_submessage,
                message_receiver,
                &mut self.status_listener.write_lock(),
                participant_status_listener,
            );
            match data_submessage_received_result {
                UserDefinedReaderDataSubmessageReceivedResult::NoChange => (),
                UserDefinedReaderDataSubmessageReceivedResult::NewDataAvailable => {
                    *self.data_on_readers_status_changed_flag.write_lock() = true
                }
            }
        }
        if *self.data_on_readers_status_changed_flag.read_lock() {
            self.on_data_on_readers(participant_status_listener);
        }
    }

    fn on_gap_submessage_received(
        &self,
        gap_submessage: &GapSubmessage,
        message_receiver: &MessageReceiver,
    ) {
        for data_reader in self.data_reader_list.read_lock().iter() {
            data_reader
                .on_gap_submessage_received(gap_submessage, message_receiver.source_guid_prefix());
        }
    }
}
