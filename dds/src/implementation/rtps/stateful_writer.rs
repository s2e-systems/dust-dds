use crate::{
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
    history_cache::{RtpsWriterCacheChange, WriterHistoryCache},
    messages::{
        overall_structure::RtpsMessageHeader,
        submessages::{AckNackSubmessage, NackFragSubmessage},
    },
    reader_proxy::{ChangeForReaderStatusKind, RtpsChangeForReader, RtpsReaderProxy},
    transport::TransportWrite,
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

            for change in self.writer.writer_cache().changes() {
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

    fn add_change(&mut self, change: RtpsWriterCacheChange) {
        let sequence_number = change.sequence_number();
        match self.writer.get_qos().durability.kind {
            DurabilityQosPolicyKind::Volatile => {
                if !self.matched_readers.is_empty() {
                    self.writer.writer_cache_mut().add_change(change);
                }
            }
            DurabilityQosPolicyKind::TransientLocal => {
                self.writer.writer_cache_mut().add_change(change)
            }
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

    pub fn get_key_value(&self, handle: InstanceHandle) -> Option<&DdsSerializedKey> {
        self.writer.get_key_value(handle)
    }

    pub fn dispose_w_timestamp(
        &mut self,
        instance_serialized_key: Vec<u8>,
        handle: InstanceHandle,
        timestamp: Time,
    ) -> DdsResult<()> {
        let change = self
            .writer
            .new_dispose_change(instance_serialized_key, handle, timestamp)?;
        self.add_change(change);

        Ok(())
    }

    pub fn unregister_instance_w_timestamp(
        &mut self,
        instance_serialized_key: Vec<u8>,
        handle: InstanceHandle,
        timestamp: Time,
    ) -> DdsResult<()> {
        let change =
            self.writer
                .new_unregister_change(instance_serialized_key, handle, timestamp)?;
        self.add_change(change);
        Ok(())
    }

    pub fn lookup_instance(
        &self,
        instance_serialized_key: DdsSerializedKey,
    ) -> Option<InstanceHandle> {
        self.writer.lookup_instance(instance_serialized_key)
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

    pub fn set_qos(&mut self, qos: DataWriterQos) -> DdsResult<()> {
        self.writer.set_qos(qos)
    }

    pub fn get_qos(&self) -> &DataWriterQos {
        self.writer.get_qos()
    }

    pub fn writer_cache(&self) -> &WriterHistoryCache {
        self.writer.writer_cache()
    }

    pub fn send_message(
        &mut self,
        header: RtpsMessageHeader,
        transport: &mut impl TransportWrite,
        now: Time,
    ) {
        self.writer.remove_stale_changes(now);

        for reader_proxy in self.matched_readers.iter_mut() {
            reader_proxy.send_message(
                self.writer.writer_cache(),
                self.writer.guid().entity_id(),
                self.writer.data_max_size_serialized(),
                self.writer.heartbeat_period(),
                header,
                transport,
            );
        }
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
