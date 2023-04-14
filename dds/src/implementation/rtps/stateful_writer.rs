use serde::Serialize;

use crate::{
    implementation::data_representation_inline_qos::{
        parameter_id_values::PID_STATUS_INFO,
        types::{
            STATUS_INFO_DISPOSED, STATUS_INFO_DISPOSED_UNREGISTERED, STATUS_INFO_UNREGISTERED,
        },
    },
    infrastructure::{
        error::DdsResult,
        instance::{InstanceHandle, HANDLE_NIL},
        qos::DataWriterQos,
        qos_policy::{DurabilityQosPolicyKind, ReliabilityQosPolicyKind},
        time::{Duration, Time, DURATION_ZERO},
    },
    topic_definition::type_support::DdsSerializedKey,
};

use super::{
    history_cache::{RtpsParameter, RtpsWriterCacheChange},
    messages::{
        submessages::{AckNackSubmessage, NackFragSubmessage},
        types::ParameterId,
    },
    reader_proxy::{
        ChangeForReaderStatusKind, RtpsChangeForReader, RtpsReaderProxy,
        WriterAssociatedReaderProxy,
    },
    types::{ChangeKind, DurabilityKind, Guid, GuidPrefix, Locator},
    writer::RtpsWriter,
};

pub const DEFAULT_HEARTBEAT_PERIOD: Duration = Duration::new(2, 0);
pub const DEFAULT_NACK_RESPONSE_DELAY: Duration = Duration::new(0, 200);
pub const DEFAULT_NACK_SUPPRESSION_DURATION: Duration = DURATION_ZERO;

pub struct RtpsStatefulWriter {
    writer: RtpsWriter,
    matched_readers: Vec<RtpsReaderProxy>,
}

impl RtpsStatefulWriter {
    pub fn new(writer: RtpsWriter) -> Self {
        Self {
            writer,
            matched_readers: Vec::new(),
        }
    }

    pub fn guid(&self) -> Guid {
        self.writer.guid()
    }

    pub fn unicast_locator_list(&self) -> &[Locator] {
        self.writer.unicast_locator_list()
    }

    pub fn multicast_locator_list(&self) -> &[Locator] {
        self.writer.multicast_locator_list()
    }

    pub fn push_mode(&self) -> bool {
        self.writer.push_mode()
    }

    pub fn heartbeat_period(&self) -> Duration {
        self.writer.heartbeat_period()
    }

    pub fn data_max_size_serialized(&self) -> usize {
        self.writer.data_max_size_serialized()
    }

    pub fn new_change(
        &mut self,
        kind: ChangeKind,
        data: Vec<u8>,
        inline_qos: Vec<RtpsParameter>,
        handle: InstanceHandle,
        timestamp: Time,
    ) -> RtpsWriterCacheChange {
        self.writer
            .new_change(kind, data, inline_qos, handle, timestamp)
    }

    pub fn change_list(&self) -> &[RtpsWriterCacheChange] {
        self.writer.change_list()
    }

    pub fn add_change(&mut self, change: RtpsWriterCacheChange) {
        let sequence_number = change.sequence_number();
        match self.writer.get_qos().durability.kind {
            DurabilityQosPolicyKind::Volatile => {
                if !self.matched_readers.is_empty() {
                    self.writer.add_change(change);
                }
            }
            DurabilityQosPolicyKind::TransientLocal => self.writer.add_change(change),
        }

        for reader_proxy in &mut self.matched_readers {
            let status = if self.writer.push_mode() {
                ChangeForReaderStatusKind::Unsent
            } else {
                ChangeForReaderStatusKind::Unacknowledged
            };
            reader_proxy
                .changes_for_reader_mut()
                .push(RtpsChangeForReader::new(status, true, sequence_number))
        }
    }

    pub fn remove_change<F>(&mut self, f: F)
    where
        F: FnMut(&RtpsWriterCacheChange) -> bool,
    {
        self.writer.remove_change(f)
    }

    pub fn matched_reader_add(&mut self, mut a_reader_proxy: RtpsReaderProxy) {
        if !self
            .matched_readers
            .iter()
            .any(|x| x.remote_reader_guid() == a_reader_proxy.remote_reader_guid())
        {
            let status = if self.writer.push_mode() {
                ChangeForReaderStatusKind::Unsent
            } else {
                ChangeForReaderStatusKind::Unacknowledged
            };

            let is_relevant = match a_reader_proxy.durability() {
                DurabilityKind::Volatile => false,
                DurabilityKind::TransientLocal => true,
            };

            for change in self.writer.change_list() {
                a_reader_proxy
                    .changes_for_reader_mut()
                    .push(RtpsChangeForReader::new(
                        status,
                        is_relevant,
                        change.sequence_number(),
                    ));
            }

            self.matched_readers.push(a_reader_proxy)
        }
    }

    pub fn matched_reader_remove(&mut self, a_reader_guid: Guid) {
        self.matched_readers
            .retain(|x| x.remote_reader_guid() != a_reader_guid)
    }

    pub fn matched_reader_list(&mut self) -> Vec<WriterAssociatedReaderProxy> {
        let writer = &self.writer;
        self.matched_readers
            .iter_mut()
            .map(|x| WriterAssociatedReaderProxy::new(writer, x))
            .collect()
    }

    pub fn is_acked_by_all(&self, a_change: &RtpsWriterCacheChange) -> bool {
        for matched_reader in self.matched_readers.iter() {
            if let Some(cc) = matched_reader
                .changes_for_reader()
                .iter()
                .find(|x| x.sequence_number() == a_change.sequence_number())
            {
                if cc.is_relevant() && cc.status() != ChangeForReaderStatusKind::Acknowledged {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    pub fn register_instance_w_timestamp(
        &mut self,
        instance_serialized_key: DdsSerializedKey,
        timestamp: Time,
    ) -> DdsResult<Option<InstanceHandle>> {
        self.writer
            .register_instance_w_timestamp(instance_serialized_key, timestamp)
    }

    pub fn write_w_timestamp(
        &mut self,
        serialized_data: Vec<u8>,
        instance_serialized_key: DdsSerializedKey,
        _handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        let handle = self
            .writer
            .register_instance_w_timestamp(instance_serialized_key, timestamp)?
            .unwrap_or(HANDLE_NIL);
        let change = self.writer.new_change(
            ChangeKind::Alive,
            serialized_data,
            vec![],
            handle,
            timestamp,
        );
        self.add_change(change);

        Ok(())
    }

    pub fn get_key_value(&self, handle: InstanceHandle) -> Option<&DdsSerializedKey> {
        self.writer.get_key_value(handle)
    }

    pub fn dispose_w_timestamp(
        &mut self,
        instance_serialized_key: Vec<u8>,
        handle: InstanceHandle,
        timestamp: Time,
    ) -> DdsResult<()> {
        let mut serialized_status_info = Vec::new();
        let mut serializer =
            cdr::Serializer::<_, cdr::LittleEndian>::new(&mut serialized_status_info);
        STATUS_INFO_DISPOSED.serialize(&mut serializer).unwrap();

        let inline_qos = vec![RtpsParameter::new(
            ParameterId(PID_STATUS_INFO),
            serialized_status_info,
        )];

        let change = self.writer.new_change(
            ChangeKind::NotAliveDisposed,
            instance_serialized_key,
            inline_qos,
            handle,
            timestamp,
        );

        self.add_change(change);

        Ok(())
    }

    pub fn unregister_instance_w_timestamp(
        &mut self,
        instance_serialized_key: Vec<u8>,
        handle: InstanceHandle,
        timestamp: Time,
    ) -> DdsResult<()> {
        let mut serialized_status_info = Vec::new();
        let mut serializer =
            cdr::Serializer::<_, cdr::LittleEndian>::new(&mut serialized_status_info);
        if self
            .writer
            .get_qos()
            .writer_data_lifecycle
            .autodispose_unregistered_instances
        {
            STATUS_INFO_DISPOSED_UNREGISTERED
                .serialize(&mut serializer)
                .unwrap();
        } else {
            STATUS_INFO_UNREGISTERED.serialize(&mut serializer).unwrap();
        }

        let inline_qos = vec![RtpsParameter::new(
            ParameterId(PID_STATUS_INFO),
            serialized_status_info,
        )];

        let change = self.writer.new_change(
            ChangeKind::NotAliveUnregistered,
            instance_serialized_key,
            inline_qos,
            handle,
            timestamp,
        );

        self.add_change(change);
        Ok(())
    }

    pub fn lookup_instance(
        &self,
        instance_serialized_key: DdsSerializedKey,
    ) -> Option<InstanceHandle> {
        self.writer.lookup_instance(instance_serialized_key)
    }

    pub fn set_qos(&mut self, qos: DataWriterQos) -> DdsResult<()> {
        self.writer.set_qos(qos)
    }

    pub fn get_qos(&self) -> &DataWriterQos {
        self.writer.get_qos()
    }

    pub fn on_acknack_submessage_received(
        &mut self,
        acknack_submessage: &AckNackSubmessage,
        src_guid_prefix: GuidPrefix,
    ) {
        if self.writer.get_qos().reliability.kind == ReliabilityQosPolicyKind::Reliable {
            let reader_guid = Guid::new(src_guid_prefix, acknack_submessage.reader_id);

            if let Some(reader_proxy) = self
                .matched_readers
                .iter_mut()
                .find(|x| x.remote_reader_guid() == reader_guid)
            {
                reader_proxy.receive_acknack(acknack_submessage);
            }
        }
    }

    pub fn on_nack_frag_submessage_received(
        &mut self,
        nackfrag_submessage: &NackFragSubmessage,
        src_guid_prefix: GuidPrefix,
    ) {
        if self.writer.get_qos().reliability.kind == ReliabilityQosPolicyKind::Reliable {
            let reader_guid = Guid::new(src_guid_prefix, nackfrag_submessage.reader_id);

            if let Some(reader_proxy) = self
                .matched_readers
                .iter_mut()
                .find(|x| x.remote_reader_guid() == reader_guid)
            {
                reader_proxy.receive_nack_frag(nackfrag_submessage);
            }
        }
    }
}
