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
        submessages::{
            AckNackSubmessage, GapSubmessage, HeartbeatFragSubmessage, HeartbeatSubmessage,
            InfoDestinationSubmessage, InfoTimestampSubmessage, NackFragSubmessage,
        },
        types::FragmentNumber,
        RtpsMessage, RtpsSubmessageKind,
    },
    reader_proxy::{ChangeForReaderStatusKind, RtpsChangeForReader, RtpsReaderProxy},
    transport::TransportWrite,
    types::{Count, EntityId, Guid, GuidPrefix, Locator, SequenceNumber},
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
    heartbeat_frag_count: Count,
}

impl<T: TimerConstructor> RtpsStatefulWriter<T> {
    pub fn new(writer: RtpsWriter) -> Self {
        Self {
            writer,
            matched_readers: Vec::new(),
            heartbeat_timer: T::new(),
            heartbeat_count: Count::new(0),
            heartbeat_frag_count: Count::new(0),
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

fn info_timestamp_submessage<'a>(timestamp: Time) -> RtpsSubmessageKind<'a> {
    RtpsSubmessageKind::InfoTimestamp(InfoTimestampSubmessage {
        endianness_flag: true,
        invalidate_flag: false,
        timestamp: super::messages::types::Time::new(timestamp.sec(), timestamp.nanosec()),
    })
}
fn info_destination_submessage<'a>(guid_prefix: GuidPrefix) -> RtpsSubmessageKind<'a> {
    RtpsSubmessageKind::InfoDestination(InfoDestinationSubmessage {
        endianness_flag: true,
        guid_prefix,
    })
}
fn heartbeat_submessage<'a>(
    reader_id: EntityId,
    writer_id: EntityId,
    writer_cache: &WriterHistoryCache,
    count: Count,
) -> RtpsSubmessageKind<'a> {
    RtpsSubmessageKind::Heartbeat(HeartbeatSubmessage {
        endianness_flag: true,
        final_flag: false,
        liveliness_flag: false,
        reader_id,
        writer_id,
        first_sn: writer_cache
            .get_seq_num_min()
            .unwrap_or(SequenceNumber::new(1)),
        last_sn: writer_cache
            .get_seq_num_max()
            .unwrap_or_else(|| SequenceNumber::new(0)),
        count,
    })
}

fn heartbeat_frag<'a>(
    reader_id: EntityId,
    writer_id: EntityId,
    writer_sn: SequenceNumber,
    last_fragment_num: FragmentNumber,
    count: Count,
) -> RtpsSubmessageKind<'a> {
    RtpsSubmessageKind::HeartbeatFrag(HeartbeatFragSubmessage {
        endianness_flag: true,
        reader_id,
        writer_id,
        writer_sn,
        last_fragment_num,
        count,
    })
}

impl<T> RtpsStatefulWriter<T>
where
    T: Timer,
{
    pub fn send_message(&mut self, header: RtpsMessageHeader, transport: &mut impl TransportWrite) {
        match self.writer.get_qos().reliability.kind {
            ReliabilityQosPolicyKind::BestEffort => {
                self.send_message_best_effort(header, transport)
            }
            ReliabilityQosPolicyKind::Reliable => self.send_submessage_reliable(header, transport),
        }
    }

    fn send_message_best_effort(
        &mut self,
        header: RtpsMessageHeader,
        transport: &mut impl TransportWrite,
    ) {
        for reader_proxy in self.matched_readers.iter_mut() {
            let info_dst = info_destination_submessage(reader_proxy.remote_reader_guid().prefix());
            let mut submessages = vec![info_dst];

            while !reader_proxy.unsent_changes().is_empty() {
                // Note: The readerId is set to the remote reader ID as described in 8.4.9.2.12 Transition T12
                // in confront to ENTITYID_UNKNOWN as described in 8.4.9.2.4 Transition T4
                let reader_id = reader_proxy.remote_reader_guid().entity_id();
                let change = reader_proxy.next_unsent_change(self.writer.writer_cache());

                if change.is_relevant() {
                    let timestamp = change.timestamp();

                    if change.data_value().len() > self.writer.data_max_size_serialized() {
                        let data_frag_submessage_list =
                            change.cache_change().as_data_frag_submessages(
                                self.writer.data_max_size_serialized(),
                                reader_id,
                            );
                        for data_frag_submessage in data_frag_submessage_list {
                            let info_dst = info_destination_submessage(
                                reader_proxy.remote_reader_guid().prefix(),
                            );

                            let into_timestamp = info_timestamp_submessage(timestamp);
                            let data_frag = RtpsSubmessageKind::DataFrag(data_frag_submessage);

                            let submessages = vec![info_dst, into_timestamp, data_frag];

                            transport.write(
                                &RtpsMessage::new(header, submessages),
                                reader_proxy.unicast_locator_list(),
                            )
                        }
                    } else {
                        submessages.push(info_timestamp_submessage(timestamp));
                        submessages.push(RtpsSubmessageKind::Data(
                            change.cache_change().as_data_submessage(reader_id),
                        ))
                    }
                } else {
                    let mut gap_submessage: GapSubmessage = change.into();
                    gap_submessage.reader_id = reader_proxy.remote_reader_guid().entity_id();
                    submessages.push(RtpsSubmessageKind::Gap(gap_submessage));
                }
            }

            // Send messages only if more than INFO_DST is added
            if submessages.len() > 1 {
                transport.write(
                    &RtpsMessage::new(header, submessages),
                    reader_proxy.unicast_locator_list(),
                )
            }
        }
    }

    fn send_submessage_reliable(
        &mut self,
        header: RtpsMessageHeader,
        transport: &mut impl TransportWrite,
    ) {
        let writer_id = self.writer.guid().entity_id();

        let time_for_heartbeat = self.heartbeat_timer.elapsed()
            >= std::time::Duration::from_secs(self.writer.heartbeat_period().sec() as u64)
                + std::time::Duration::from_nanos(self.writer.heartbeat_period().nanosec() as u64);
        if time_for_heartbeat {
            self.heartbeat_timer.reset();
            self.heartbeat_count = self.heartbeat_count.wrapping_add(1);
        }
        for reader_proxy in self.matched_readers.iter_mut() {
            let reader_id = reader_proxy.remote_reader_guid().entity_id();

            let info_dst = info_destination_submessage(reader_proxy.remote_reader_guid().prefix());

            let mut submessages = vec![info_dst];

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
                        let timestamp = change.timestamp();

                        if change.data_value().len() > self.writer.data_max_size_serialized() {
                            let data_frag_submessage_list =
                                change.cache_change().as_data_frag_submessages(
                                    self.writer.data_max_size_serialized(),
                                    reader_id,
                                );
                            let total_number_of_fragments = data_frag_submessage_list.len();
                            for (index, data_frag_submessage) in
                                data_frag_submessage_list.into_iter().enumerate()
                            {
                                let info_dst = info_destination_submessage(
                                    reader_proxy.remote_reader_guid().prefix(),
                                );
                                let writer_sn = data_frag_submessage.writer_sn;
                                let into_timestamp = info_timestamp_submessage(timestamp);
                                let data_frag = RtpsSubmessageKind::DataFrag(data_frag_submessage);
                                let hearbeat = if index + 1 == total_number_of_fragments {
                                    self.heartbeat_count = self.heartbeat_count.wrapping_add(1);
                                    heartbeat_submessage(
                                        reader_id,
                                        writer_id,
                                        self.writer.writer_cache(),
                                        self.heartbeat_count,
                                    )
                                } else {
                                    self.heartbeat_frag_count =
                                        self.heartbeat_frag_count.wrapping_add(1);
                                    heartbeat_frag(
                                        reader_id,
                                        writer_id,
                                        writer_sn,
                                        FragmentNumber::new((index + 1) as u32),
                                        self.heartbeat_frag_count,
                                    )
                                };

                                let submessages =
                                    vec![info_dst, into_timestamp, data_frag, hearbeat];

                                transport.write(
                                    &RtpsMessage::new(header, submessages),
                                    reader_proxy.unicast_locator_list(),
                                )
                            }
                        } else {
                            submessages.push(info_timestamp_submessage(timestamp));
                            submessages.push(RtpsSubmessageKind::Data(
                                change.cache_change().as_data_submessage(reader_id),
                            ))
                        }
                    } else {
                        let mut gap_submessage: GapSubmessage = change.into();
                        gap_submessage.reader_id = reader_id;
                        submessages.push(RtpsSubmessageKind::Gap(gap_submessage));
                    }
                }

                self.heartbeat_timer.reset();
                self.heartbeat_count = self.heartbeat_count.wrapping_add(1);
                let heartbeat = heartbeat_submessage(
                    reader_id,
                    writer_id,
                    self.writer.writer_cache(),
                    self.heartbeat_count,
                );
                submessages.push(heartbeat);
            } else if reader_proxy.unacked_changes().is_empty() {
                // Idle
            } else if time_for_heartbeat {
                let heartbeat = heartbeat_submessage(
                    reader_id,
                    writer_id,
                    self.writer.writer_cache(),
                    self.heartbeat_count,
                );
                submessages.push(heartbeat);
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
                        let timestamp = change_for_reader.timestamp();
                        if change_for_reader.data_value().len()
                            > self.writer.data_max_size_serialized()
                        {
                            let data_frag_submessage_list =
                                change_for_reader.cache_change().as_data_frag_submessages(
                                    self.writer.data_max_size_serialized(),
                                    reader_id,
                                );
                            let total_number_of_fragments = data_frag_submessage_list.len();
                            for (index, data_frag_submessage) in
                                data_frag_submessage_list.into_iter().enumerate()
                            {
                                let info_dst = info_destination_submessage(
                                    reader_proxy.remote_reader_guid().prefix(),
                                );
                                let writer_sn = data_frag_submessage.writer_sn;
                                let into_timestamp = info_timestamp_submessage(timestamp);
                                let data_frag = RtpsSubmessageKind::DataFrag(data_frag_submessage);

                                let hearbeat = if index + 1 == total_number_of_fragments {
                                    self.heartbeat_count = self.heartbeat_count.wrapping_add(1);
                                    heartbeat_submessage(
                                        reader_id,
                                        writer_id,
                                        self.writer.writer_cache(),
                                        self.heartbeat_count,
                                    )
                                } else {
                                    self.heartbeat_frag_count =
                                        self.heartbeat_frag_count.wrapping_add(1);
                                    heartbeat_frag(
                                        reader_id,
                                        writer_id,
                                        writer_sn,
                                        FragmentNumber::new((index + 1) as u32),
                                        self.heartbeat_frag_count,
                                    )
                                };

                                let submessages =
                                    vec![info_dst, into_timestamp, data_frag, hearbeat];

                                transport.write(
                                    &RtpsMessage::new(header, submessages),
                                    reader_proxy.unicast_locator_list(),
                                )
                            }
                        } else {
                            let info_ts_submessage = info_timestamp_submessage(timestamp);
                            let data_submessage = RtpsSubmessageKind::Data(
                                change_for_reader
                                    .cache_change()
                                    .as_data_submessage(reader_id),
                            );
                            submessages.push(info_ts_submessage);
                            submessages.push(data_submessage);
                        }
                    } else {
                        let mut gap_submessage: GapSubmessage = change_for_reader.into();
                        gap_submessage.reader_id = reader_id;
                        submessages.push(RtpsSubmessageKind::Gap(gap_submessage));
                    }
                }
                self.heartbeat_timer.reset();
                self.heartbeat_count = self.heartbeat_count.wrapping_add(1);
                let heartbeat = heartbeat_submessage(
                    reader_id,
                    writer_id,
                    self.writer.writer_cache(),
                    self.heartbeat_count,
                );
                submessages.push(heartbeat);
            }
            // Send messages only if more or equal than INFO_DST and HEARTBEAT is added
            if submessages.len() >= 2 {
                transport.write(
                    &RtpsMessage::new(header, submessages),
                    reader_proxy.unicast_locator_list(),
                )
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
                reader_proxy.reliable_receive_nack_frag(nackfrag_submessage);
            }
        }
    }
}
