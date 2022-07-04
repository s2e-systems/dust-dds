use rtps_pim::{
    behavior::{
        reader::{
            reader::RtpsReaderAttributes,
            stateful_reader::{
                RtpsStatefulReaderAttributes, RtpsStatefulReaderConstructor,
                RtpsStatefulReaderOperations,
            },
            writer_proxy::{RtpsWriterProxyAttributes, RtpsWriterProxyOperations},
        },
        stateful_reader_behavior::{
            BestEffortStatefulReaderReceiveDataBehavior, BestEffortWriterProxyReceiveGapBehavior,
            ReliableStatefulReaderReceiveDataBehavior, ReliableWriterProxyReceiveGapBehavior,
            ReliableWriterProxyReceiveHeartbeat, ReliableWriterProxySendAckNack,
            RtpsStatefulReaderReceiveDataSubmessage, RtpsStatefulReaderReceiveHeartbeatSubmessage,
            RtpsStatefulReaderSendSubmessages,
        },
        types::Duration,
    },
    messages::{
        submessage_elements::Parameter,
        submessages::{AckNackSubmessage, DataSubmessage, GapSubmessage, HeartbeatSubmessage},
        types::Count,
    },
    structure::{
        endpoint::RtpsEndpointAttributes,
        entity::RtpsEntityAttributes,
        types::{Guid, GuidPrefix, Locator, ReliabilityKind, SequenceNumber, TopicKind},
    },
};

use super::{
    rtps_endpoint_impl::RtpsEndpointImpl, rtps_history_cache_impl::RtpsHistoryCacheImpl,
    rtps_reader_impl::RtpsReaderImpl, rtps_writer_proxy_impl::RtpsWriterProxyImpl,
};

pub struct RtpsStatefulReaderImpl {
    reader: RtpsReaderImpl,
    matched_writers: Vec<RtpsWriterProxyImpl>,
}

impl RtpsStatefulReaderImpl {
    pub fn process_gap_submessage(
        &mut self,
        gap_submessage: &GapSubmessage<Vec<SequenceNumber>>,
        source_guid_prefix: GuidPrefix,
    ) {
        let writer_guid = Guid::new(source_guid_prefix, gap_submessage.writer_id.value);
        let reliability_level = self.reliability_level();
        if let Some(writer_proxy) = self
            .matched_writers
            .iter_mut()
            .find(|x| x.remote_writer_guid() == writer_guid)
        {
            match reliability_level {
                ReliabilityKind::BestEffort => {
                    BestEffortWriterProxyReceiveGapBehavior::receive_gap(
                        writer_proxy,
                        gap_submessage,
                    )
                }
                ReliabilityKind::Reliable => {
                    ReliableWriterProxyReceiveGapBehavior::receive_gap(writer_proxy, gap_submessage)
                }
            }
        }
    }
}

impl RtpsEntityAttributes for RtpsStatefulReaderImpl {
    fn guid(&self) -> Guid {
        self.reader.guid()
    }
}

impl RtpsEndpointAttributes for RtpsStatefulReaderImpl {
    fn topic_kind(&self) -> TopicKind {
        self.reader.topic_kind()
    }

    fn reliability_level(&self) -> ReliabilityKind {
        self.reader.reliability_level()
    }

    fn unicast_locator_list(&self) -> &[Locator] {
        self.reader.unicast_locator_list()
    }

    fn multicast_locator_list(&self) -> &[Locator] {
        self.reader.multicast_locator_list()
    }
}

impl RtpsReaderAttributes for RtpsStatefulReaderImpl {
    type HistoryCacheType = RtpsHistoryCacheImpl;

    fn heartbeat_response_delay(&self) -> Duration {
        self.reader.heartbeat_response_delay()
    }

    fn heartbeat_suppression_duration(&self) -> Duration {
        self.reader.heartbeat_suppression_duration()
    }

    fn reader_cache(&mut self) -> &mut Self::HistoryCacheType {
        self.reader.reader_cache()
    }

    fn expects_inline_qos(&self) -> bool {
        self.reader.expects_inline_qos()
    }
}

impl<'a> RtpsStatefulReaderAttributes<'a> for RtpsStatefulReaderImpl {
    type WriterProxyListType = &'a mut [RtpsWriterProxyImpl];

    fn matched_writers(&'a mut self) -> Self::WriterProxyListType {
        self.matched_writers.as_mut_slice()
    }
}

impl RtpsStatefulReaderConstructor for RtpsStatefulReaderImpl {
    fn new(
        guid: Guid,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
        heartbeat_response_delay: Duration,
        heartbeat_suppression_duration: Duration,
        expects_inline_qos: bool,
    ) -> Self {
        Self {
            reader: RtpsReaderImpl::new(
                RtpsEndpointImpl::new(
                    guid,
                    topic_kind,
                    reliability_level,
                    unicast_locator_list,
                    multicast_locator_list,
                ),
                heartbeat_response_delay,
                heartbeat_suppression_duration,
                expects_inline_qos,
            ),
            matched_writers: Vec::new(),
        }
    }
}

impl RtpsStatefulReaderOperations for RtpsStatefulReaderImpl {
    type WriterProxyType = RtpsWriterProxyImpl;

    fn matched_writer_add(&mut self, a_writer_proxy: Self::WriterProxyType) {
        self.matched_writers.push(a_writer_proxy);
    }

    fn matched_writer_remove<F>(&mut self, mut f: F)
    where
        F: FnMut(&Self::WriterProxyType) -> bool,
    {
        self.matched_writers.retain(|x| !f(x))
    }

    fn matched_writer_lookup(&mut self, a_writer_guid: Guid) -> Option<&mut Self::WriterProxyType> {
        self.matched_writers
            .iter_mut()
            .find(|x| x.remote_writer_guid() == a_writer_guid)
    }
}

impl RtpsStatefulReaderReceiveDataSubmessage<Vec<Parameter<'_>>, &'_ [u8]>
    for RtpsStatefulReaderImpl
{
    fn on_data_submessage_received(
        &mut self,
        data_submessage: &DataSubmessage<Vec<Parameter>, &[u8]>,
        source_guid_prefix: GuidPrefix,
    ) {
        let writer_guid = Guid::new(source_guid_prefix, data_submessage.writer_id.value);

        if let Some(writer_proxy) = self
            .matched_writers
            .iter_mut()
            .find(|x| x.remote_writer_guid() == writer_guid)
        {
            if data_submessage.writer_sn.value < writer_proxy.first_available_seq_num
                || data_submessage.writer_sn.value > writer_proxy.last_available_seq_num
                || writer_proxy
                    .missing_changes()
                    .contains(&data_submessage.writer_sn.value)
            {
                match self.reliability_level() {
                    ReliabilityKind::BestEffort => {
                        BestEffortStatefulReaderReceiveDataBehavior::receive_data(
                            self,
                            source_guid_prefix,
                            data_submessage,
                        )
                    }
                    ReliabilityKind::Reliable => {
                        ReliableStatefulReaderReceiveDataBehavior::receive_data(
                            self,
                            source_guid_prefix,
                            data_submessage,
                        )
                    }
                }
            }
        }
    }
}

impl RtpsStatefulReaderReceiveHeartbeatSubmessage for RtpsStatefulReaderImpl {
    fn on_heartbeat_submessage_received(
        &mut self,
        heartbeat_submessage: &HeartbeatSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        if self.reliability_level() == ReliabilityKind::Reliable {
            let writer_guid = Guid::new(source_guid_prefix, heartbeat_submessage.writer_id.value);

            if let Some(writer_proxy) = self
                .matched_writers
                .iter_mut()
                .find(|x| x.remote_writer_guid() == writer_guid)
            {
                if writer_proxy.last_received_heartbeat_count != heartbeat_submessage.count.value {
                    writer_proxy.last_received_heartbeat_count = heartbeat_submessage.count.value;

                    writer_proxy.must_send_acknacks = !heartbeat_submessage.final_flag
                        || (!heartbeat_submessage.liveliness_flag
                            && !writer_proxy.missing_changes().is_empty());

                    ReliableWriterProxyReceiveHeartbeat::receive_heartbeat(
                        writer_proxy,
                        heartbeat_submessage,
                    );
                }
            }
        }
    }
}

impl RtpsStatefulReaderSendSubmessages<Vec<SequenceNumber>> for RtpsStatefulReaderImpl {
    type WriterProxyType = RtpsWriterProxyImpl;

    fn send_submessages(
        &mut self,
        mut send_acknack: impl FnMut(&Self::WriterProxyType, AckNackSubmessage<Vec<SequenceNumber>>),
    ) {
        let entity_id = self.guid().entity_id;
        for writer_proxy in self.matched_writers.iter_mut() {
            if writer_proxy.must_send_acknacks {
                if !writer_proxy.missing_changes().is_empty() {
                    writer_proxy.acknack_count =
                        Count(writer_proxy.acknack_count.0.wrapping_add(1));

                    ReliableWriterProxySendAckNack::send_ack_nack(
                        writer_proxy,
                        entity_id,
                        writer_proxy.acknack_count,
                        |wp, acknack| send_acknack(wp, acknack),
                    );
                }
                writer_proxy.must_send_acknacks = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rtps_pim::{
        behavior::reader::writer_proxy::{RtpsWriterProxyConstructor, RtpsWriterProxyOperations},
        messages::{
            submessage_elements::{
                CountSubmessageElement, EntityIdSubmessageElement, ParameterListSubmessageElement,
                SequenceNumberSubmessageElement, SerializedDataSubmessageElement,
            },
            types::Count,
        },
        structure::{
            cache_change::RtpsCacheChangeAttributes,
            history_cache::RtpsHistoryCacheAttributes,
            types::{EntityId, USER_DEFINED_READER_NO_KEY, USER_DEFINED_WRITER_NO_KEY},
        },
    };

    use super::*;

    #[test]
    fn process_submessage_test() {
        let mut reader = RtpsStatefulReaderImpl::new(
            Guid::new(
                GuidPrefix([3; 12]),
                EntityId::new([4, 1, 3], USER_DEFINED_READER_NO_KEY),
            ),
            TopicKind::NoKey,
            ReliabilityKind::BestEffort,
            &[],
            &[],
            Duration::new(0, 0),
            Duration::new(0, 0),
            false,
        );

        let source_guid = Guid::new(
            GuidPrefix([6; 12]),
            EntityId::new([4, 1, 3], USER_DEFINED_WRITER_NO_KEY),
        );

        let writer_proxy =
            RtpsWriterProxyImpl::new(source_guid, &[], &[], None, source_guid.entity_id);

        reader.matched_writer_add(writer_proxy);

        let writer_sn = 1;
        let serialized_payload_value = [1, 0, 2, 5];
        let data: DataSubmessage<Vec<Parameter>, &[u8]> = DataSubmessage {
            endianness_flag: true,
            inline_qos_flag: true,
            data_flag: true,
            key_flag: false,
            non_standard_payload_flag: false,
            reader_id: EntityIdSubmessageElement {
                value: reader.guid().entity_id,
            },
            writer_id: EntityIdSubmessageElement {
                value: source_guid.entity_id,
            },
            writer_sn: SequenceNumberSubmessageElement { value: writer_sn },
            inline_qos: ParameterListSubmessageElement { parameter: vec![] },
            serialized_payload: SerializedDataSubmessageElement {
                value: &serialized_payload_value,
            },
        };

        reader.on_data_submessage_received(&data, source_guid.prefix);

        assert_eq!(1, reader.reader.reader_cache().changes().len());
        let change = &reader.reader.reader_cache().changes()[0];
        assert_eq!(source_guid, change.writer_guid());
        assert_eq!(writer_sn, change.sequence_number());
        assert_eq!(serialized_payload_value, change.data_value());
    }

    #[test]
    fn reliable_stateful_data_reader_all_data_received_when_not_sent_in_order() {
        let writer_guid = Guid::new(
            GuidPrefix([0; 12]),
            EntityId::new([4, 1, 3], USER_DEFINED_WRITER_NO_KEY),
        );

        let reader_guid = Guid::new(
            GuidPrefix([1; 12]),
            EntityId::new([6, 1, 2], USER_DEFINED_READER_NO_KEY),
        );

        let mut reader = RtpsStatefulReaderImpl::new(
            reader_guid,
            TopicKind::NoKey,
            ReliabilityKind::Reliable,
            &[],
            &[],
            Duration::new(0, 0),
            Duration::new(0, 0),
            false,
        );

        reader.matched_writer_add(RtpsWriterProxyImpl::new(
            writer_guid,
            &[],
            &[],
            None,
            writer_guid.entity_id,
        ));

        let make_data = |seq_num, data: &'static [u8]| DataSubmessage {
            endianness_flag: true,
            inline_qos_flag: true,
            data_flag: true,
            key_flag: false,
            non_standard_payload_flag: false,
            reader_id: EntityIdSubmessageElement {
                value: reader_guid.entity_id,
            },
            writer_id: EntityIdSubmessageElement {
                value: writer_guid.entity_id,
            },
            writer_sn: SequenceNumberSubmessageElement { value: seq_num },
            inline_qos: ParameterListSubmessageElement { parameter: vec![] },
            serialized_payload: SerializedDataSubmessageElement { value: data },
        };

        reader.on_data_submessage_received(&make_data(2, &[2, 8, 4, 5]), writer_guid.prefix);
        reader.on_data_submessage_received(&make_data(0, &[2, 7, 1, 8]), writer_guid.prefix);
        reader.on_data_submessage_received(&make_data(3, &[9, 0, 4, 5]), writer_guid.prefix);
        reader.on_data_submessage_received(&make_data(1, &[2, 8, 1, 8]), writer_guid.prefix);

        assert_eq!(4, reader.reader_cache().changes().len());
    }

    #[test]
    fn best_effort_stateful_data_reader_data_only_received_in_order() {
        let writer_guid = Guid::new(
            GuidPrefix([0; 12]),
            EntityId::new([4, 1, 3], USER_DEFINED_WRITER_NO_KEY),
        );

        let reader_guid = Guid::new(
            GuidPrefix([1; 12]),
            EntityId::new([6, 1, 2], USER_DEFINED_READER_NO_KEY),
        );

        let mut reader = RtpsStatefulReaderImpl::new(
            reader_guid,
            TopicKind::NoKey,
            ReliabilityKind::BestEffort,
            &[],
            &[],
            Duration::new(0, 0),
            Duration::new(0, 0),
            false,
        );

        reader.matched_writer_add(RtpsWriterProxyImpl::new(
            writer_guid,
            &[],
            &[],
            None,
            writer_guid.entity_id,
        ));

        let make_data = |seq_num, data: &'static [u8]| DataSubmessage {
            endianness_flag: true,
            inline_qos_flag: true,
            data_flag: true,
            key_flag: false,
            non_standard_payload_flag: false,
            reader_id: EntityIdSubmessageElement {
                value: reader_guid.entity_id,
            },
            writer_id: EntityIdSubmessageElement {
                value: writer_guid.entity_id,
            },
            writer_sn: SequenceNumberSubmessageElement { value: seq_num },
            inline_qos: ParameterListSubmessageElement { parameter: vec![] },
            serialized_payload: SerializedDataSubmessageElement { value: data },
        };

        reader.on_data_submessage_received(&make_data(2, &[2, 8, 4, 5]), writer_guid.prefix);
        reader.on_data_submessage_received(&make_data(0, &[2, 7, 1, 8]), writer_guid.prefix);
        reader.on_data_submessage_received(&make_data(3, &[9, 0, 4, 5]), writer_guid.prefix);
        reader.on_data_submessage_received(&make_data(1, &[2, 8, 1, 8]), writer_guid.prefix);

        assert_eq!(2, reader.reader_cache().changes().len());
    }

    #[test]
    fn reliable_stateful_data_reader_receive_heartbeat() {
        let writer_guid = Guid::new(
            GuidPrefix([0; 12]),
            EntityId::new([4, 1, 3], USER_DEFINED_WRITER_NO_KEY),
        );

        let reader_guid = Guid::new(
            GuidPrefix([1; 12]),
            EntityId::new([6, 1, 2], USER_DEFINED_READER_NO_KEY),
        );

        let mut reader = RtpsStatefulReaderImpl::new(
            reader_guid,
            TopicKind::NoKey,
            ReliabilityKind::Reliable,
            &[],
            &[],
            Duration::new(0, 0),
            Duration::new(0, 0),
            false,
        );

        reader.matched_writer_add(RtpsWriterProxyImpl::new(
            writer_guid,
            &[],
            &[],
            None,
            writer_guid.entity_id,
        ));

        let make_data = |seq_num, data: &'static [u8]| DataSubmessage {
            endianness_flag: true,
            inline_qos_flag: true,
            data_flag: true,
            key_flag: false,
            non_standard_payload_flag: false,
            reader_id: EntityIdSubmessageElement {
                value: reader_guid.entity_id,
            },
            writer_id: EntityIdSubmessageElement {
                value: writer_guid.entity_id,
            },
            writer_sn: SequenceNumberSubmessageElement { value: seq_num },
            inline_qos: ParameterListSubmessageElement { parameter: vec![] },
            serialized_payload: SerializedDataSubmessageElement { value: data },
        };

        let make_heartbeat = |first_sn, last_sn, count| -> HeartbeatSubmessage {
            HeartbeatSubmessage {
                endianness_flag: true,
                final_flag: false,
                liveliness_flag: false,
                reader_id: EntityIdSubmessageElement {
                    value: reader_guid.entity_id,
                },
                writer_id: EntityIdSubmessageElement {
                    value: writer_guid.entity_id,
                },
                first_sn: SequenceNumberSubmessageElement { value: first_sn },
                last_sn: SequenceNumberSubmessageElement { value: last_sn },
                count: CountSubmessageElement { value: count },
            }
        };

        assert!(reader.matched_writers[0].missing_changes().is_empty());

        reader
            .on_heartbeat_submessage_received(&make_heartbeat(1, 0, Count(1)), writer_guid.prefix);
        assert!(reader.matched_writers[0].missing_changes().is_empty());

        reader
            .on_heartbeat_submessage_received(&make_heartbeat(1, 1, Count(2)), writer_guid.prefix);
        assert_eq!(vec![1], reader.matched_writers[0].missing_changes());

        reader.on_data_submessage_received(&make_data(1, &[]), writer_guid.prefix);
        assert!(reader.matched_writers[0].missing_changes().is_empty());

        reader
            .on_heartbeat_submessage_received(&make_heartbeat(1, 2, Count(3)), writer_guid.prefix);
        assert_eq!(vec![2], reader.matched_writers[0].missing_changes());

        reader.on_data_submessage_received(&make_data(4, &[]), writer_guid.prefix);
        reader
            .on_heartbeat_submessage_received(&make_heartbeat(1, 5, Count(4)), writer_guid.prefix);
        assert_eq!(vec![2, 3, 5], reader.matched_writers[0].missing_changes());

        reader
            .on_heartbeat_submessage_received(&make_heartbeat(2, 5, Count(5)), writer_guid.prefix);
        assert_eq!(vec![2, 3, 5], reader.matched_writers[0].missing_changes());
    }

    #[test]
    fn reliable_stateful_data_reader_sends_acknack() {
        let writer_guid = Guid::new(
            GuidPrefix([0; 12]),
            EntityId::new([4, 1, 3], USER_DEFINED_WRITER_NO_KEY),
        );

        let reader_guid = Guid::new(
            GuidPrefix([1; 12]),
            EntityId::new([6, 1, 2], USER_DEFINED_READER_NO_KEY),
        );

        let mut reader = RtpsStatefulReaderImpl::new(
            reader_guid,
            TopicKind::NoKey,
            ReliabilityKind::Reliable,
            &[],
            &[],
            Duration::new(0, 0),
            Duration::new(0, 0),
            false,
        );

        reader.matched_writer_add(RtpsWriterProxyImpl::new(
            writer_guid,
            &[],
            &[],
            None,
            writer_guid.entity_id,
        ));

        let make_data = |seq_num, data: &'static [u8]| DataSubmessage {
            endianness_flag: true,
            inline_qos_flag: true,
            data_flag: true,
            key_flag: false,
            non_standard_payload_flag: false,
            reader_id: EntityIdSubmessageElement {
                value: reader_guid.entity_id,
            },
            writer_id: EntityIdSubmessageElement {
                value: writer_guid.entity_id,
            },
            writer_sn: SequenceNumberSubmessageElement { value: seq_num },
            inline_qos: ParameterListSubmessageElement { parameter: vec![] },
            serialized_payload: SerializedDataSubmessageElement { value: data },
        };

        let make_heartbeat = |first_sn, last_sn, count| -> HeartbeatSubmessage {
            HeartbeatSubmessage {
                endianness_flag: true,
                final_flag: false,
                liveliness_flag: false,
                reader_id: EntityIdSubmessageElement {
                    value: reader_guid.entity_id,
                },
                writer_id: EntityIdSubmessageElement {
                    value: writer_guid.entity_id,
                },
                first_sn: SequenceNumberSubmessageElement { value: first_sn },
                last_sn: SequenceNumberSubmessageElement { value: last_sn },
                count: CountSubmessageElement { value: count },
            }
        };

        assert!(reader.matched_writers[0].missing_changes().is_empty());

        reader
            .on_heartbeat_submessage_received(&make_heartbeat(1, 0, Count(1)), writer_guid.prefix);
        reader.send_submessages(|_, _| assert!(false));

        reader
            .on_heartbeat_submessage_received(&make_heartbeat(1, 1, Count(2)), writer_guid.prefix);
        let mut submessages = Vec::new();
        reader.send_submessages(|_, acknack| submessages.push(acknack));
        assert_eq!(1, submessages.len());

        // doesn't send a second time
        reader.send_submessages(|_, _| assert!(false));

        // resend when new heartbeat
        reader
            .on_heartbeat_submessage_received(&make_heartbeat(1, 1, Count(3)), writer_guid.prefix);
        let mut submessages = Vec::new();
        reader.send_submessages(|_, a| submessages.push(a));
        assert_eq!(1, submessages.len());

        // doesn't send if message received in the meantime
        reader
            .on_heartbeat_submessage_received(&make_heartbeat(1, 1, Count(4)), writer_guid.prefix);
        reader.on_data_submessage_received(&make_data(1, &[]), writer_guid.prefix);
        reader.send_submessages(|_, _| assert!(false));
    }
}
