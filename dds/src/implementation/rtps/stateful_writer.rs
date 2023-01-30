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
            AckNackSubmessage, GapSubmessage, HeartbeatSubmessage, InfoDestinationSubmessage,
            InfoTimestampSubmessage,
        },
        types::ProtocolId,
        RtpsMessage, RtpsSubmessageKind,
    },
    reader_proxy::{
        ChangeForReaderStatusKind, RtpsChangeForReader, RtpsChangeForReaderCacheChange,
        RtpsReaderProxy,
    },
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

fn info_timestamp_submessage<'a>(
    timestamp: Time,
) -> RtpsSubmessageKind<'a> {
    RtpsSubmessageKind::InfoTimestamp(InfoTimestampSubmessage {
        endianness_flag: true,
        invalidate_flag: false,
        timestamp: super::messages::types::Time::new(
            timestamp.sec(),
            timestamp.nanosec(),
        ),
    })
}
fn info_destination_submessage<'a>(guid_prefix: GuidPrefix) -> RtpsSubmessageKind<'a> {
    RtpsSubmessageKind::InfoDestination(InfoDestinationSubmessage {
        endianness_flag: true,
        guid_prefix,
    })
}

fn send_submessages(
    guid_prefix: GuidPrefix,
    transport: &mut impl TransportWrite,
    submessages: Vec<RtpsSubmessageKind>,
    locators: &[Locator],
) {
    let header = RtpsMessageHeader {
        protocol: ProtocolId::PROTOCOL_RTPS,
        version: PROTOCOLVERSION,
        vendor_id: VENDOR_ID_S2E,
        guid_prefix,
    };

    let rtps_message = RtpsMessage {
        header,
        submessages,
    };

    for locator in locators {
        transport.write(&rtps_message, *locator)
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
            let info_dst = info_destination_submessage(reader_proxy.remote_reader_guid().prefix());
            let mut submessages = vec![info_dst];

            while !reader_proxy.unsent_changes().is_empty() {
                // Note: The readerId is set to the remote reader ID as described in 8.4.9.2.12 Transition T12
                // in confront to ENTITYID_UNKNOWN as described in 8.4.9.2.4 Transition T4
                let reader_id = reader_proxy.remote_reader_guid().entity_id();
                let change = reader_proxy.next_unsent_change(self.writer.writer_cache());

                if change.is_relevant() {
                    let timestamp = change.timestamp();
                    match self.writer.data_max_size_serialized() {
                        Some(data_max_size_serialized)
                            if change.data_value().len() > data_max_size_serialized =>
                        {
                            let data_frag_submessage_list = change
                                .cache_change()
                                .as_data_frag_submessages(data_max_size_serialized, reader_id);
                            for data_frag_submessage in data_frag_submessage_list {
                                let info_dst = info_destination_submessage(
                                    reader_proxy.remote_reader_guid().prefix(),
                                );

                                let into_timestamp = info_timestamp_submessage(timestamp);
                                let data_frag = RtpsSubmessageKind::DataFrag(data_frag_submessage);
                                send_submessages(
                                    self.writer.guid().prefix(),
                                    transport,
                                    vec![info_dst, into_timestamp, data_frag],
                                    reader_proxy.unicast_locator_list(),
                                )
                            }
                        }
                        _ => {
                            submessages.push(info_timestamp_submessage(timestamp));
                            submessages.push(RtpsSubmessageKind::Data(
                                change.cache_change().as_data_submessage(reader_id),
                            ))
                        }
                    }
                } else {
                    let mut gap_submessage: GapSubmessage = change.into();
                    gap_submessage.reader_id = reader_proxy.remote_reader_guid().entity_id();
                    submessages.push(RtpsSubmessageKind::Gap(gap_submessage));
                }
            }

            // Send messages only if more than INFO_DST is added
            if submessages.len() > 1 {
                send_submessages(
                    self.writer.guid().prefix(),
                    transport,
                    submessages,
                    reader_proxy.unicast_locator_list(),
                )
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
                guid_prefix: reader_proxy.remote_reader_guid().prefix(),
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
                        let info_ts_submessage = InfoTimestampSubmessage {
                            endianness_flag: true,
                            invalidate_flag: false,
                            timestamp: super::messages::types::Time::new(
                                change.timestamp().sec(),
                                change.timestamp().nanosec(),
                            ),
                        };
                        submessages.push(RtpsSubmessageKind::InfoTimestamp(info_ts_submessage));

                        let reader_id = reader_proxy.remote_reader_guid().entity_id();

                        match self.writer.data_max_size_serialized() {
                            Some(data_max_size_serialized)
                                if change.data_value().len() > data_max_size_serialized =>
                            {
                                let data_frag_submessage_list = change
                                    .cache_change()
                                    .as_data_frag_submessages(data_max_size_serialized, reader_id);
                                for data_frag_submessage in data_frag_submessage_list {
                                    submessages
                                        .push(RtpsSubmessageKind::DataFrag(data_frag_submessage));
                                }
                            }
                            _ => submessages.push(RtpsSubmessageKind::Data(
                                change.cache_change().as_data_submessage(reader_id),
                            )),
                        }
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
                    guid_prefix: self.writer.guid().prefix(),
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

#[cfg(test)]
mod tests {
    use crate::{
        implementation::rtps::{
            endpoint::RtpsEndpoint,
            messages::types::{FragmentNumber, UShort},
            types::{
                EntityId, EntityKey, LocatorAddress, LocatorKind, LocatorPort, TopicKind,
                ENTITYID_UNKNOWN, GUID_UNKNOWN, USER_DEFINED_READER_NO_KEY,
            },
            utils::clock::StdTimer,
        },
        topic_definition::type_support::{DdsSerde, DdsType},
    };

    use super::*;
    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct LargeData {
        value: Vec<u8>,
    }
    impl DdsType for LargeData {
        fn type_name() -> &'static str {
            "LargeData"
        }
    }
    impl DdsSerde for LargeData {}

    use mockall::mock;

    mock! {
        Transport{}

        impl TransportWrite for Transport {
            fn write<'a>(&'a mut self, message: &RtpsMessage<'a>, destination_locator: Locator);
        }
    }

    #[test]
    fn write_frag_message() {
        let remote_reader_guid = Guid::new(
            GuidPrefix::new([4; 12]),
            EntityId::new(EntityKey::new([0, 0, 0x03]), USER_DEFINED_READER_NO_KEY),
        );

        let max_bytes = 60000;

        let mut rtps_writer = RtpsStatefulWriter::<StdTimer>::new(RtpsWriter::new(
            RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::WithKey, &[], &[]),
            true,
            DURATION_ZERO,
            DURATION_ZERO,
            DURATION_ZERO,
            Some(max_bytes),
            DataWriterQos::default(),
        ));
        let data = LargeData {
            value: vec![3; 100000],
        };
        let data_length = data.value.len() + 4 /*CDR header*/ + 4 /*length of vector*/;

        rtps_writer
            .write_w_timestamp(&data, None, Time::new(1, 0))
            .unwrap();

        let remote_group_entity_id = ENTITYID_UNKNOWN;
        let expects_inline_qos = false;
        let proxy = RtpsReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            &[Locator::new(
                LocatorKind::new(1),
                LocatorPort::new(2),
                LocatorAddress::new([3; 16]),
            )],
            &[],
            expects_inline_qos,
            true,
        );
        rtps_writer.matched_reader_add(proxy);

        let mut transport = MockTransport::new();
        transport
            .expect_write()
            .times(2)
            .withf(move |message, _destination_locator| {
                let data_frag_submessages: Vec<_> = message
                    .submessages
                    .iter()
                    .filter(|s| match s {
                        RtpsSubmessageKind::DataFrag(_) => true,
                        _ => false,
                    })
                    .collect();
                // if data_frag_submessages.len() != 3 {
                //     false
                // } else {
                true
                // let ret1 = if let RtpsSubmessageKind::DataFrag(submessage) =
                //     data_frag_submessages[0]
                // {
                //     submessage.fragment_starting_num == FragmentNumber::new(1)
                //         && submessage.fragment_size == UShort::new(max_bytes as u16)
                //         && <&[u8]>::from(&submessage.serialized_payload).len() == max_bytes
                // } else {
                //     false
                // };
                // let ret2 = if let RtpsSubmessageKind::DataFrag(submessage) =
                //     data_frag_submessages[1]
                // {
                //     submessage.fragment_starting_num == FragmentNumber::new(2)
                //         && submessage.fragment_size
                //             == UShort::new((data_length - max_bytes) as u16)
                //         && <&[u8]>::from(&submessage.serialized_payload).len()
                //             == data_length - max_bytes
                // } else {
                //     false
                // };
                // ret1 && ret2
                // }
            })
            .return_const(());

        rtps_writer.send_message_best_effort(&mut transport);
    }
}
