use crate::{
    implementation::rtps::utils::clock::{Timer, TimerConstructor},
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::DataWriterQos,
        qos_policy::ReliabilityQosPolicyKind,
        time::{Duration, Time, DURATION_ZERO},
    },
    topic_definition::type_support::{DdsSerialize, DdsType},
};

use super::{
    history_cache::{RtpsWriterCacheChange, WriterHistoryCache},
    messages::{
        overall_structure::RtpsMessageHeader,
        submessage_elements::GuidPrefixSubmessageElement,
        submessages::{
            AckNackSubmessage, GapSubmessage, HeartbeatSubmessage, InfoDestinationSubmessage,
        },
        types::ProtocolId,
        RtpsMessage, RtpsSubmessageKind,
    },
    reader_proxy::{ChangeForReaderStatusKind, RtpsChangeForReader, RtpsReaderProxy},
    transport::TransportWrite,
    types::{Count, Guid, GuidPrefix, Locator, SequenceNumber, PROTOCOLVERSION, VENDOR_ID_S2E},
    writer::RtpsWriter,
};

pub const DEFAULT_HEARTBEAT_PERIOD: Duration = Duration::new(2, 0);
pub const DEFAULT_NACK_RESPONSE_DELAY: Duration = Duration::new(0, 200);
pub const DEFAULT_NACK_SUPPRESSION_DURATION: Duration = DURATION_ZERO;

pub struct RtpsStatefulWriter<T> {
    writer: RtpsWriter,
    matched_readers: Vec<RtpsReaderProxy>,
    heartbeat_timer: T,
    heartbeat_count: Count,
}

impl<T: TimerConstructor> RtpsStatefulWriter<T> {
    pub fn new(writer: RtpsWriter) -> Self {
        Self {
            writer,
            matched_readers: Vec::new(),
            heartbeat_timer: T::new(),
            heartbeat_count: Count::new(0),
        }
    }
}

impl<T> RtpsStatefulWriter<T> {
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
            for change in self.writer.writer_cache().changes() {
                a_reader_proxy
                    .changes_for_reader_mut()
                    .push(RtpsChangeForReader::new(
                        status,
                        true,
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
                if !(cc.is_relevant() && cc.status() == ChangeForReaderStatusKind::Acknowledged) {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    pub fn register_instance_w_timestamp<Foo>(
        &mut self,
        instance: &Foo,
        timestamp: Time,
    ) -> DdsResult<Option<InstanceHandle>>
    where
        Foo: DdsType + DdsSerialize,
    {
        self.writer
            .register_instance_w_timestamp(instance, timestamp)
    }

    pub fn write_w_timestamp<Foo>(
        &mut self,
        data: &Foo,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()>
    where
        Foo: DdsType + DdsSerialize,
    {
        let change = self.writer.new_write_change(data, handle, timestamp)?;
        self.add_change(change);

        Ok(())
    }

    fn add_change(&mut self, change: RtpsWriterCacheChange) {
        let sequence_number = change.sequence_number();
        self.writer.writer_cache_mut().add_change(change);

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

    pub fn get_key_value<Foo>(&self, key_holder: &mut Foo, handle: InstanceHandle) -> DdsResult<()>
    where
        Foo: DdsType,
    {
        self.writer.get_key_value(key_holder, handle)
    }

    pub fn dispose_w_timestamp<Foo>(
        &mut self,
        data: &Foo,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()>
    where
        Foo: DdsType,
    {
        let change = self.writer.new_dispose_change(data, handle, timestamp)?;
        self.add_change(change);

        Ok(())
    }

    pub fn unregister_instance_w_timestamp<Foo>(
        &mut self,
        instance: &Foo,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()>
    where
        Foo: DdsType + DdsSerialize,
    {
        let change = self
            .writer
            .new_unregister_change(instance, handle, timestamp)?;
        self.add_change(change);
        Ok(())
    }

    pub fn lookup_instance<Foo>(&self, instance: &Foo) -> Option<InstanceHandle>
    where
        Foo: DdsType,
    {
        self.writer.lookup_instance(instance)
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
}

impl<T> RtpsStatefulWriter<T>
where
    T: Timer,
{
    pub fn send_message(&mut self, transport: &mut impl TransportWrite) {
        match self.writer.get_qos().reliability.kind {
            ReliabilityQosPolicyKind::BestEffort => self.send_message_best_effort(transport),
            ReliabilityQosPolicyKind::Reliable => self.send_submessage_reliable(transport),
        }
    }

    fn send_message_best_effort(&mut self, transport: &mut impl TransportWrite) {
        for reader_proxy in self.matched_readers.iter_mut() {
            let info_dst = InfoDestinationSubmessage {
                endianness_flag: true,
                guid_prefix: GuidPrefixSubmessageElement {
                    value: reader_proxy.remote_reader_guid().prefix(),
                },
            };
            let mut submessages = vec![RtpsSubmessageKind::InfoDestination(info_dst)];

            if !reader_proxy.unsent_changes().is_empty() {
                // Note: The readerId is set to the remote reader ID as described in 8.4.9.2.12 Transition T12
                // in confront to ENTITYID_UNKNOWN as described in 8.4.9.2.4 Transition T4

                while !reader_proxy.unsent_changes().is_empty() {
                    let change = reader_proxy.next_unsent_change(self.writer.writer_cache());
                    // "a_change.status := UNDERWAY;" should be done by next_requested_change() as
                    // it's not done here to avoid the change being a mutable reference
                    // Also the post-condition:
                    // "( a_change BELONGS-TO the_reader_proxy.unsent_changes() ) == FALSE"
                    // should be full-filled by next_unsent_change()
                    if change.is_relevant() {
                        let (info_ts_submessage, mut data_submessage) = change.into();
                        data_submessage.reader_id = reader_proxy.remote_reader_guid().entity_id();
                        submessages.push(RtpsSubmessageKind::InfoTimestamp(info_ts_submessage));
                        submessages.push(RtpsSubmessageKind::Data(data_submessage));
                    } else {
                        let mut gap_submessage: GapSubmessage = change.into();
                        gap_submessage.reader_id = reader_proxy.remote_reader_guid().entity_id();
                        submessages.push(RtpsSubmessageKind::Gap(gap_submessage));
                    }
                }
            }
            // Send messages only if more than INFO_DST is added
            if submessages.len() > 1 {
                let header = RtpsMessageHeader {
                    protocol: ProtocolId::PROTOCOL_RTPS,
                    version: PROTOCOLVERSION,
                    vendor_id: VENDOR_ID_S2E,
                    guid_prefix: GuidPrefixSubmessageElement {
                        value: self.writer.guid().prefix(),
                    },
                };

                let rtps_message = RtpsMessage {
                    header,
                    submessages,
                };

                for locator in reader_proxy.unicast_locator_list() {
                    transport.write(&rtps_message, *locator)
                }
            }
        }
    }

    fn send_submessage_reliable(&mut self, transport: &mut impl TransportWrite) {
        let time_for_heartbeat = self.heartbeat_timer.elapsed()
            >= std::time::Duration::from_secs(self.writer.heartbeat_period().sec() as u64)
                + std::time::Duration::from_nanos(self.writer.heartbeat_period().nanosec() as u64);
        if time_for_heartbeat {
            self.heartbeat_timer.reset();
            self.heartbeat_count = self.heartbeat_count.wrapping_add(1);
        }
        for reader_proxy in self.matched_readers.iter_mut() {
            let info_dst = InfoDestinationSubmessage {
                endianness_flag: true,
                guid_prefix: GuidPrefixSubmessageElement {
                    value: reader_proxy.remote_reader_guid().prefix(),
                },
            };
            let mut submessages = vec![RtpsSubmessageKind::InfoDestination(info_dst)];
            // Top part of the state machine - Figure 8.19 RTPS standard
            if !reader_proxy.unsent_changes().is_empty() {
                // Note: The readerId is set to the remote reader ID as described in 8.4.9.2.12 Transition T12
                // in confront to ENTITYID_UNKNOWN as described in 8.4.9.2.4 Transition T4

                while !reader_proxy.unsent_changes().is_empty() {
                    let change = reader_proxy.next_unsent_change(self.writer.writer_cache());
                    // "a_change.status := UNDERWAY;" should be done by next_requested_change() as
                    // it's not done here to avoid the change being a mutable reference
                    // Also the post-condition:
                    // "( a_change BELONGS-TO the_reader_proxy.unsent_changes() ) == FALSE"
                    // should be full-filled by next_unsent_change()
                    if change.is_relevant() {
                        let (info_ts_submessage, mut data_submessage) = change.into();
                        data_submessage.reader_id = reader_proxy.remote_reader_guid().entity_id();
                        submessages.push(RtpsSubmessageKind::InfoTimestamp(info_ts_submessage));
                        submessages.push(RtpsSubmessageKind::Data(data_submessage));
                    } else {
                        let mut gap_submessage: GapSubmessage = change.into();
                        gap_submessage.reader_id = reader_proxy.remote_reader_guid().entity_id();
                        submessages.push(RtpsSubmessageKind::Gap(gap_submessage));
                    }
                }

                self.heartbeat_timer.reset();
                self.heartbeat_count = self.heartbeat_count.wrapping_add(1);
                let heartbeat = HeartbeatSubmessage {
                    endianness_flag: true,
                    final_flag: false,
                    liveliness_flag: false,
                    reader_id: reader_proxy.remote_reader_guid().entity_id(),
                    writer_id: self.writer.guid().entity_id(),
                    first_sn: self
                        .writer
                        .writer_cache()
                        .get_seq_num_min()
                        .unwrap_or(SequenceNumber::new(1)),
                    last_sn: self
                        .writer
                        .writer_cache()
                        .get_seq_num_max()
                        .unwrap_or_else(|| SequenceNumber::new(0)),
                    count: self.heartbeat_count,
                };

                submessages.push(RtpsSubmessageKind::Heartbeat(heartbeat));
            } else if reader_proxy.unacked_changes().is_empty() {
                // Idle
            } else if time_for_heartbeat {
                let heartbeat = HeartbeatSubmessage {
                    endianness_flag: true,
                    final_flag: false,
                    liveliness_flag: false,
                    reader_id: reader_proxy.remote_reader_guid().entity_id(),
                    writer_id: self.writer.guid().entity_id(),
                    first_sn: self
                        .writer
                        .writer_cache()
                        .get_seq_num_min()
                        .unwrap_or(SequenceNumber::new(1)),
                    last_sn: self
                        .writer
                        .writer_cache()
                        .get_seq_num_max()
                        .unwrap_or(SequenceNumber::new(0)),
                    count: self.heartbeat_count,
                };

                submessages.push(RtpsSubmessageKind::Heartbeat(heartbeat));
            }

            // Middle-part of the state-machine - Figure 8.19 RTPS standard
            if !reader_proxy.requested_changes().is_empty() {
                let reader_id = reader_proxy.remote_reader_guid().entity_id();

                while !reader_proxy.requested_changes().is_empty() {
                    let change_for_reader =
                        reader_proxy.next_requested_change(self.writer.writer_cache());
                    // "a_change.status := UNDERWAY;" should be done by next_requested_change() as
                    // it's not done here to avoid the change being a mutable reference
                    // Also the post-condition:
                    // a_change BELONGS-TO the_reader_proxy.requested_changes() ) == FALSE
                    // should be full-filled by next_requested_change()
                    if change_for_reader.is_relevant() {
                        let (info_ts_submessage, mut data_submessage) = change_for_reader.into();
                        data_submessage.reader_id = reader_id;
                        submessages.push(RtpsSubmessageKind::InfoTimestamp(info_ts_submessage));
                        submessages.push(RtpsSubmessageKind::Data(data_submessage));
                    } else {
                        let mut gap_submessage: GapSubmessage = change_for_reader.into();
                        gap_submessage.reader_id = reader_id;
                        submessages.push(RtpsSubmessageKind::Gap(gap_submessage));
                    }
                }
                self.heartbeat_timer.reset();
                self.heartbeat_count = self.heartbeat_count.wrapping_add(1);
                let heartbeat = HeartbeatSubmessage {
                    endianness_flag: true,
                    final_flag: false,
                    liveliness_flag: false,
                    reader_id: reader_proxy.remote_reader_guid().entity_id(),
                    writer_id: self.writer.guid().entity_id(),
                    first_sn: self
                        .writer
                        .writer_cache()
                        .get_seq_num_min()
                        .unwrap_or(SequenceNumber::new(1)),
                    last_sn: self
                        .writer
                        .writer_cache()
                        .get_seq_num_max()
                        .unwrap_or(SequenceNumber::new(0)),
                    count: self.heartbeat_count,
                };

                submessages.push(RtpsSubmessageKind::Heartbeat(heartbeat));
            }
            // Send messages only if more than INFO_DST is added
            if submessages.len() > 1 {
                let header = RtpsMessageHeader {
                    protocol: ProtocolId::PROTOCOL_RTPS,
                    version: PROTOCOLVERSION,
                    vendor_id: VENDOR_ID_S2E,
                    guid_prefix: GuidPrefixSubmessageElement {
                        value: self.writer.guid().prefix(),
                    },
                };

                let rtps_message = RtpsMessage {
                    header,
                    submessages,
                };

                for locator in reader_proxy.unicast_locator_list() {
                    transport.write(&rtps_message, *locator)
                }
            }
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
                reader_proxy.reliable_receive_acknack(acknack_submessage);
            }
        }
    }
}
