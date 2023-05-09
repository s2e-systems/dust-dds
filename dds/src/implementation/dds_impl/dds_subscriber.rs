use std::sync::{mpsc::Sender, RwLockWriteGuard};

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
            stateless_reader::RtpsStatelessReader,
            types::{Guid, GuidPrefix},
        },
        utils::shared_object::{DdsRwLock, DdsShared},
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind, SubscriberQos, TopicQos},
        status::StatusKind,
        time::Time,
    },
    subscription::{subscriber::Subscriber, subscriber_listener::SubscriberListener},
};

use super::{
    dds_data_reader::{DdsDataReader, UserDefinedReaderDataSubmessageReceivedResult},
    message_receiver::MessageReceiver,
    node_kind::SubscriberNodeKind,
    node_listener_subscriber::ListenerSubscriberNode,
    status_condition_impl::StatusConditionImpl,
    status_listener::{ListenerTriggerKind, StatusListener},
};

pub struct DdsSubscriber {
    qos: DdsRwLock<SubscriberQos>,
    rtps_group: RtpsGroup,
    stateless_data_reader_list: Vec<DdsShared<DdsDataReader<RtpsStatelessReader>>>,
    stateful_data_reader_list: Vec<DdsShared<DdsDataReader<RtpsStatefulReader>>>,
    enabled: DdsRwLock<bool>,
    status_condition: DdsShared<DdsRwLock<StatusConditionImpl>>,
    data_on_readers_status_changed_flag: DdsRwLock<bool>,
    status_listener: DdsRwLock<StatusListener<dyn SubscriberListener + Send + Sync>>,
    user_defined_data_reader_counter: DdsRwLock<u8>,
    default_data_reader_qos: DdsRwLock<DataReaderQos>,
}

impl DdsSubscriber {
    pub fn new(
        qos: SubscriberQos,
        rtps_group: RtpsGroup,
        listener: Option<Box<dyn SubscriberListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> Self {
        DdsSubscriber {
            qos: DdsRwLock::new(qos),
            rtps_group,
            stateless_data_reader_list: Vec::new(),
            stateful_data_reader_list: Vec::new(),
            enabled: DdsRwLock::new(false),
            status_condition: DdsShared::new(DdsRwLock::new(StatusConditionImpl::default())),
            data_on_readers_status_changed_flag: DdsRwLock::new(false),
            status_listener: DdsRwLock::new(StatusListener::new(listener, mask)),
            user_defined_data_reader_counter: DdsRwLock::new(0),
            default_data_reader_qos: DdsRwLock::new(Default::default()),
        }
    }

    pub fn guid(&self) -> Guid {
        self.rtps_group.guid()
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

    pub fn get_unique_reader_id(&self) -> u8 {
        let mut counter_lock = self.user_defined_data_reader_counter.write_lock();
        let counter = *counter_lock;
        *counter_lock += 1;
        counter
    }

    pub fn stateless_data_reader_add(
        &mut self,
        data_reader: DdsShared<DdsDataReader<RtpsStatelessReader>>,
    ) {
        self.stateless_data_reader_list.push(data_reader)
    }

    pub fn _stateless_data_reader_delete(&mut self, a_datareader_handle: InstanceHandle) {
        self.stateless_data_reader_list
            .retain(|x| x._get_instance_handle() != a_datareader_handle)
    }

    pub fn stateless_data_reader_list(&self) -> &[DdsShared<DdsDataReader<RtpsStatelessReader>>] {
        &self.stateless_data_reader_list
    }

    pub fn get_stateless_data_reader(
        &self,
        data_reader: Guid,
    ) -> Option<&DdsShared<DdsDataReader<RtpsStatelessReader>>> {
        self.stateless_data_reader_list
            .iter()
            .find(|s| s.guid() == data_reader)
    }

    pub fn stateful_data_reader_add(
        &mut self,
        data_reader: DdsShared<DdsDataReader<RtpsStatefulReader>>,
    ) {
        self.stateful_data_reader_list.push(data_reader)
    }

    pub fn stateful_data_reader_delete(&mut self, a_datareader_handle: InstanceHandle) {
        self.stateful_data_reader_list
            .retain(|x| x.get_instance_handle() != a_datareader_handle)
    }

    pub fn stateful_data_reader_list(&self) -> &[DdsShared<DdsDataReader<RtpsStatefulReader>>] {
        &self.stateful_data_reader_list
    }

    pub fn get_stateful_data_reader(
        &self,
        data_reader: Guid,
    ) -> Option<&DdsShared<DdsDataReader<RtpsStatefulReader>>> {
        self.stateful_data_reader_list
            .iter()
            .find(|s| s.guid() == data_reader)
    }

    pub fn stateful_data_reader_drain(
        &mut self,
    ) -> std::vec::Drain<DdsShared<DdsDataReader<RtpsStatefulReader>>> {
        self.stateful_data_reader_list.drain(..)
    }

    pub fn set_default_datareader_qos(&self, qos: QosKind<DataReaderQos>) -> DdsResult<()> {
        match qos {
            QosKind::Default => {
                *self.default_data_reader_qos.write_lock() = DataReaderQos::default()
            }
            QosKind::Specific(q) => {
                q.is_consistent()?;
                *self.default_data_reader_qos.write_lock() = q;
            }
        }
        Ok(())
    }

    pub fn get_default_datareader_qos(&self) -> DataReaderQos {
        self.default_data_reader_qos.read_lock().clone()
    }

    pub fn update_communication_status(
        &self,
        now: Time,
        parent_participant_guid: Guid,
        listener_sender: &Sender<ListenerTriggerKind>,
    ) {
        for data_reader in self.stateful_data_reader_list.iter() {
            data_reader.update_communication_status(
                now,
                parent_participant_guid,
                self.guid(),
                listener_sender,
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
            for data_reader in self.stateful_data_reader_list.iter() {
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
        parent_subcriber_guid: Guid,
        parent_participant_guid: Guid,
        listener_sender: &Sender<ListenerTriggerKind>,
    ) {
        self.trigger_on_data_on_readers_listener(
            &mut self.status_listener.write_lock(),
            participant_status_listener,
            parent_subcriber_guid,
            parent_participant_guid,
            listener_sender,
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
        parent_subcriber_guid: Guid,
        parent_participant_guid: Guid,
        listener_sender: &Sender<ListenerTriggerKind>,
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
            for data_reader in self.stateful_data_reader_list.iter() {
                data_reader.on_data_available(
                    parent_subcriber_guid,
                    parent_participant_guid,
                    listener_sender,
                );
            }
        }
    }

    pub fn on_heartbeat_submessage_received(
        &self,
        heartbeat_submessage: &HeartbeatSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        for data_reader in self.stateful_data_reader_list.iter() {
            data_reader.on_heartbeat_submessage_received(heartbeat_submessage, source_guid_prefix)
        }
    }

    pub fn on_heartbeat_frag_submessage_received(
        &self,
        heartbeat_frag_submessage: &HeartbeatFragSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        for data_reader in self.stateful_data_reader_list.iter() {
            data_reader.on_heartbeat_frag_submessage_received(
                heartbeat_frag_submessage,
                source_guid_prefix,
            )
        }
    }

    pub fn on_data_submessage_received(
        &self,
        data_submessage: &DataSubmessage<'_>,
        message_receiver: &MessageReceiver,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
        parent_participant_guid: Guid,
        listener_sender: &Sender<ListenerTriggerKind>,
    ) {
        for stateless_data_reader in self.stateless_data_reader_list.iter() {
            stateless_data_reader.on_data_submessage_received(data_submessage, message_receiver);
        }

        for data_reader in self.stateful_data_reader_list.iter() {
            let data_submessage_received_result = data_reader.on_data_submessage_received(
                data_submessage,
                message_receiver,
                self.guid(),
                parent_participant_guid,
                listener_sender,
            );
            match data_submessage_received_result {
                UserDefinedReaderDataSubmessageReceivedResult::NoChange => (),
                UserDefinedReaderDataSubmessageReceivedResult::NewDataAvailable => {
                    *self.data_on_readers_status_changed_flag.write_lock() = true
                }
            }
        }
        if *self.data_on_readers_status_changed_flag.read_lock() {
            self.on_data_on_readers(
                participant_status_listener,
                self.guid(),
                parent_participant_guid,
                listener_sender,
            );
        }
    }

    pub fn on_data_frag_submessage_received(
        &self,
        data_frag_submessage: &DataFragSubmessage<'_>,
        message_receiver: &MessageReceiver,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
        parent_subcriber_guid: Guid,
        parent_participant_guid: Guid,
        listener_sender: &Sender<ListenerTriggerKind>,
    ) {
        for data_reader in self.stateful_data_reader_list.iter() {
            let data_submessage_received_result = data_reader.on_data_frag_submessage_received(
                data_frag_submessage,
                message_receiver,
                self.guid(),
                parent_participant_guid,
                listener_sender,
            );
            match data_submessage_received_result {
                UserDefinedReaderDataSubmessageReceivedResult::NoChange => (),
                UserDefinedReaderDataSubmessageReceivedResult::NewDataAvailable => {
                    *self.data_on_readers_status_changed_flag.write_lock() = true
                }
            }
        }
        if *self.data_on_readers_status_changed_flag.read_lock() {
            self.on_data_on_readers(
                participant_status_listener,
                parent_subcriber_guid,
                parent_participant_guid,
                listener_sender,
            );
        }
    }

    pub fn on_gap_submessage_received(
        &self,
        gap_submessage: &GapSubmessage,
        message_receiver: &MessageReceiver,
    ) {
        for data_reader in self.stateful_data_reader_list.iter() {
            data_reader
                .on_gap_submessage_received(gap_submessage, message_receiver.source_guid_prefix());
        }
    }
}
