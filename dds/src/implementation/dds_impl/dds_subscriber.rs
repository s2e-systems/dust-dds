use std::sync::RwLockWriteGuard;

use crate::{
    domain::domain_participant_listener::DomainParticipantListener,
    implementation::{
        rtps::{
            group::RtpsGroup,
            messages::submessages::{
                DataFragSubmessage, DataSubmessage, GapSubmessage, HeartbeatFragSubmessage,
                HeartbeatSubmessage,
            },
            stateful_reader::RtpsStatefulReader,
            types::{GuidPrefix, Locator},
        },
        utils::{
            iterator::{DdsDrainIntoIterator, DdsListIntoIterator},
            shared_object::{DdsRwLock, DdsShared},
        },
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind, SubscriberQos, TopicQos},
        status::StatusKind,
        time::Time,
    },
    subscription::{subscriber::Subscriber, subscriber_listener::SubscriberListener},
    topic_definition::type_support::{DdsDeserialize, DdsType},
};

use super::{
    any_data_reader_listener::AnyDataReaderListener,
    dds_data_reader::{DdsDataReader, UserDefinedReaderDataSubmessageReceivedResult},
    message_receiver::{MessageReceiver, SubscriberSubmessageReceiver},
    node_kind::SubscriberNodeKind,
    node_listener_subscriber::ListenerSubscriberNode,
    reader_factory::ReaderFactory,
    status_condition_impl::StatusConditionImpl,
    status_listener::StatusListener,
};

pub struct DdsSubscriber {
    qos: DdsRwLock<SubscriberQos>,
    rtps_group: RtpsGroup,
    data_reader_list: DdsRwLock<Vec<DdsShared<DdsDataReader<RtpsStatefulReader>>>>,
    reader_factory: DdsRwLock<ReaderFactory>,
    enabled: DdsRwLock<bool>,
    status_condition: DdsShared<DdsRwLock<StatusConditionImpl>>,
    data_on_readers_status_changed_flag: DdsRwLock<bool>,
    status_listener: DdsRwLock<StatusListener<dyn SubscriberListener + Send + Sync>>,
}

impl DdsSubscriber {
    pub fn new(
        qos: SubscriberQos,
        rtps_group: RtpsGroup,
        listener: Option<Box<dyn SubscriberListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsShared<Self> {
        DdsShared::new(DdsSubscriber {
            qos: DdsRwLock::new(qos),
            rtps_group,
            data_reader_list: DdsRwLock::new(Vec::new()),
            reader_factory: DdsRwLock::new(ReaderFactory::new()),
            enabled: DdsRwLock::new(false),
            status_condition: DdsShared::new(DdsRwLock::new(StatusConditionImpl::default())),
            data_on_readers_status_changed_flag: DdsRwLock::new(false),
            status_listener: DdsRwLock::new(StatusListener::new(listener, mask)),
        })
    }

    pub fn copy_from_topic_qos(
        _a_datareader_qos: &mut DataReaderQos,
        _a_topic_qos: &TopicQos,
    ) -> DdsResult<()> {
        todo!()
    }

    pub fn get_status_listener_lock(
        &self,
    ) -> RwLockWriteGuard<StatusListener<dyn SubscriberListener + Send + Sync>> {
        self.status_listener.write_lock()
    }

    pub fn is_enabled(&self) -> bool {
        *self.enabled.read_lock()
    }

    pub fn get_qos(&self) -> SubscriberQos {
        self.qos.read_lock().clone()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_datareader<Foo>(
        &self,
        type_name: &'static str,
        topic_name: String,
        qos: QosKind<DataReaderQos>,
        a_listener: Option<Box<dyn AnyDataReaderListener + Send + Sync>>,
        mask: &[StatusKind],
        default_unicast_locator_list: &[Locator],
        default_multicast_locator_list: &[Locator],
    ) -> DdsResult<DdsShared<DdsDataReader<RtpsStatefulReader>>>
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

        let data_reader_shared =
            DdsDataReader::new(rtps_reader, type_name, topic_name, a_listener, mask);

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

    pub fn delete_datareader(&self, a_datareader_handle: InstanceHandle) {
        self.data_reader_list
            .write_lock()
            .retain(|x| x.get_instance_handle() != a_datareader_handle)
    }

    pub fn data_reader_list(
        &self,
    ) -> DdsListIntoIterator<DdsShared<DdsDataReader<RtpsStatefulReader>>> {
        DdsListIntoIterator::new(self.data_reader_list.read_lock())
    }

    pub fn data_reader_drain(
        &self,
    ) -> DdsDrainIntoIterator<DdsShared<DdsDataReader<RtpsStatefulReader>>> {
        DdsDrainIntoIterator::new(self.data_reader_list.write_lock())
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
}

impl DdsShared<DdsSubscriber> {
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
                .as_mut()
                .expect("Listener should be some")
                .on_data_on_readers(&Subscriber::new(SubscriberNodeKind::Listener(
                    ListenerSubscriberNode::new(),
                )))
        } else if participant_status_listener.is_enabled(data_on_readers_status_kind) {
            participant_status_listener
                .listener_mut()
                .as_mut()
                .expect("Listener should be some")
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

impl SubscriberSubmessageReceiver for DdsShared<DdsSubscriber> {
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
