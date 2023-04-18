use std::sync::{mpsc::SyncSender, RwLockWriteGuard};

use crate::{
    implementation::{
        rtps::{
            endpoint::RtpsEndpoint,
            group::RtpsGroup,
            messages::submessages::{AckNackSubmessage, NackFragSubmessage},
            stateful_writer::RtpsStatefulWriter,
            stateless_writer::RtpsStatelessWriter,
            types::{
                EntityId, EntityKey, Guid, Locator, TopicKind, USER_DEFINED_WRITER_NO_KEY,
                USER_DEFINED_WRITER_WITH_KEY,
            },
            writer::RtpsWriter,
        },
        utils::{
            iterator::{DdsDrainIntoIterator, DdsListIntoIterator},
            shared_object::{DdsRwLock, DdsShared},
        },
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind},
        status::StatusKind,
        time::{Duration, DURATION_ZERO},
    },
    publication::publisher_listener::PublisherListener,
    topic_definition::type_support::DdsType,
};

use super::{
    any_data_writer_listener::AnyDataWriterListener,
    dds_data_writer::DdsDataWriter,
    domain_participant_impl::AnnounceKind,
    message_receiver::{MessageReceiver, PublisherMessageReceiver},
    status_condition_impl::StatusConditionImpl,
    status_listener::StatusListener,
};

pub struct DdsPublisher {
    qos: DdsRwLock<PublisherQos>,
    rtps_group: RtpsGroup,
    stateless_data_writer_list: DdsRwLock<Vec<DdsShared<DdsDataWriter<RtpsStatelessWriter>>>>,
    stateful_data_writer_list: DdsRwLock<Vec<DdsShared<DdsDataWriter<RtpsStatefulWriter>>>>,
    enabled: DdsRwLock<bool>,
    status_listener: DdsRwLock<StatusListener<dyn PublisherListener + Send + Sync>>,
    data_max_size_serialized: usize,
    status_condition: DdsShared<DdsRwLock<StatusConditionImpl>>,
    announce_sender: SyncSender<AnnounceKind>,
    user_defined_data_writer_counter: DdsRwLock<u8>,
    default_datawriter_qos: DdsRwLock<DataWriterQos>,
}

impl DdsPublisher {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        qos: PublisherQos,
        rtps_group: RtpsGroup,
        listener: Option<Box<dyn PublisherListener + Send + Sync>>,
        mask: &[StatusKind],
        data_max_size_serialized: usize,
        announce_sender: SyncSender<AnnounceKind>,
    ) -> DdsShared<Self> {
        DdsShared::new(DdsPublisher {
            qos: DdsRwLock::new(qos),
            rtps_group,
            stateless_data_writer_list: DdsRwLock::new(Vec::new()),
            stateful_data_writer_list: DdsRwLock::new(Vec::new()),
            enabled: DdsRwLock::new(false),
            status_listener: DdsRwLock::new(StatusListener::new(listener, mask)),
            data_max_size_serialized,
            status_condition: DdsShared::new(DdsRwLock::new(StatusConditionImpl::default())),
            announce_sender,
            user_defined_data_writer_counter: DdsRwLock::new(0),
            default_datawriter_qos: DdsRwLock::new(DataWriterQos::default()),
        })
    }

    pub fn enable(&self) -> DdsResult<()> {
        *self.enabled.write_lock() = true;

        Ok(())
    }

    pub fn is_enabled(&self) -> bool {
        *self.enabled.read_lock()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_datawriter<Foo>(
        &self,
        type_name: &'static str,
        topic_name: String,
        qos: QosKind<DataWriterQos>,
        a_listener: Option<Box<dyn AnyDataWriterListener + Send + Sync>>,
        mask: &[StatusKind],
        default_unicast_locator_list: &[Locator],
        default_multicast_locator_list: &[Locator],
    ) -> DdsResult<DdsShared<DdsDataWriter<RtpsStatefulWriter>>>
    where
        Foo: DdsType,
    {
        let qos = match qos {
            QosKind::Default => self.get_default_datawriter_qos(),
            QosKind::Specific(q) => q,
        };
        qos.is_consistent()?;

        let entity_kind = match Foo::has_key() {
            true => USER_DEFINED_WRITER_WITH_KEY,
            false => USER_DEFINED_WRITER_NO_KEY,
        };

        let entity_key = EntityKey::new([
            <[u8; 3]>::from(self.rtps_group.guid().entity_id().entity_key())[0],
            self.get_unique_writer_id(),
            0,
        ]);

        let entity_id = EntityId::new(entity_key, entity_kind);

        let guid = Guid::new(self.rtps_group.guid().prefix(), entity_id);

        let topic_kind = match Foo::has_key() {
            true => TopicKind::WithKey,
            false => TopicKind::NoKey,
        };

        let rtps_writer_impl = RtpsStatefulWriter::new(RtpsWriter::new(
            RtpsEndpoint::new(
                guid,
                topic_kind,
                default_unicast_locator_list,
                default_multicast_locator_list,
            ),
            true,
            Duration::new(0, 200_000_000),
            DURATION_ZERO,
            DURATION_ZERO,
            self.data_max_size_serialized,
            qos,
        ));

        let data_writer_shared =
            DdsDataWriter::new(rtps_writer_impl, a_listener, mask, type_name, topic_name);

        self.stateful_data_writer_list
            .write_lock()
            .push(data_writer_shared.clone());

        Ok(data_writer_shared)
    }

    pub fn get_unique_writer_id(&self) -> u8 {
        todo!()
    }

    pub fn stateful_datawriter_add(
        &self,
        data_writer: DdsShared<DdsDataWriter<RtpsStatefulWriter>>,
    ) {
        self.stateful_data_writer_list
            .write_lock()
            .push(data_writer)
    }

    pub fn stateful_datawriter_drain(
        &self,
    ) -> DdsDrainIntoIterator<DdsShared<DdsDataWriter<RtpsStatefulWriter>>> {
        DdsDrainIntoIterator::new(self.stateful_data_writer_list.write_lock())
    }

    pub fn stateful_datawriter_delete(&self, data_writer_handle: InstanceHandle) {
        self.stateful_data_writer_list
            .write_lock()
            .retain(|x| InstanceHandle::from(x.guid()) != data_writer_handle);
    }

    pub fn stateful_data_writer_list(
        &self,
    ) -> DdsListIntoIterator<DdsShared<DdsDataWriter<RtpsStatefulWriter>>> {
        DdsListIntoIterator::new(self.stateful_data_writer_list.read_lock())
    }

    pub fn get_status_listener_lock(
        &self,
    ) -> RwLockWriteGuard<StatusListener<dyn PublisherListener + Send + Sync>> {
        self.status_listener.write_lock()
    }

    pub fn set_default_datawriter_qos(&self, qos: QosKind<DataWriterQos>) -> DdsResult<()> {
        match qos {
            QosKind::Default => {
                *self.default_datawriter_qos.write_lock() = DataWriterQos::default()
            }
            QosKind::Specific(q) => {
                q.is_consistent()?;
                *self.default_datawriter_qos.write_lock() = q;
            }
        }
        Ok(())
    }

    pub fn get_default_datawriter_qos(&self) -> DataWriterQos {
        self.default_datawriter_qos.read_lock().clone()
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

    pub fn get_statuscondition(&self) -> DdsShared<DdsRwLock<StatusConditionImpl>> {
        self.status_condition.clone()
    }

    pub fn get_status_changes(&self) -> Vec<StatusKind> {
        self.status_condition.read_lock().get_status_changes()
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.rtps_group.guid().into()
    }
}

impl PublisherMessageReceiver for DdsShared<DdsPublisher> {
    fn on_acknack_submessage_received(
        &self,
        acknack_submessage: &AckNackSubmessage,
        message_receiver: &MessageReceiver,
    ) {
        for data_writer in self.stateful_data_writer_list.read_lock().iter() {
            data_writer.on_acknack_submessage_received(acknack_submessage, message_receiver);
        }
    }

    fn on_nack_frag_submessage_received(
        &self,
        nackfrag_submessage: &NackFragSubmessage,
        message_receiver: &MessageReceiver,
    ) {
        for data_writer in self.stateful_data_writer_list.read_lock().iter() {
            data_writer.on_nack_frag_submessage_received(nackfrag_submessage, message_receiver);
        }
    }
}
