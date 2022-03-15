use rtps_pim::{
    behavior::{
        reader::{
            reader::RtpsReaderAttributes,
            stateful_reader::{
                RtpsStatefulReaderAttributes, RtpsStatefulReaderConstructor,
                RtpsStatefulReaderOperations,
            },
            writer_proxy::RtpsWriterProxyAttributes,
        },
        stateful_reader_behavior::{
            BestEffortStatefulReaderBehavior, ReliableStatefulReaderBehavior,
        },
        types::Duration,
    },
    messages::{submessage_elements::Parameter, submessages::DataSubmessage},
    structure::{
        endpoint::RtpsEndpointAttributes,
        entity::RtpsEntityAttributes,
        types::{Guid, GuidPrefix, Locator, ReliabilityKind, TopicKind},
    },
};

use super::{
    rtps_endpoint_impl::RtpsEndpointImpl, rtps_history_cache_impl::RtpsHistoryCacheImpl,
    rtps_reader_impl::RtpsReaderImpl, rtps_writer_proxy_impl::RtpsWriterProxyImpl,
};

pub struct RtpsStatefulReaderImpl {
    pub reader: RtpsReaderImpl,
    pub matched_writers: Vec<RtpsWriterProxyImpl>,
}

impl RtpsStatefulReaderImpl {
    pub fn process_submessage(
        &mut self,
        data: &DataSubmessage<Vec<Parameter>, &[u8]>,
        source_guid_prefix: GuidPrefix,
    ) {
        let writer_guid = Guid::new(source_guid_prefix, data.writer_id.value);

        if let Some(writer_proxy) = self
            .matched_writers
            .iter_mut()
            .find(|x| x.remote_writer_guid() == writer_guid)
        {
            match self.reader.endpoint.reliability_level {
                ReliabilityKind::BestEffort => BestEffortStatefulReaderBehavior::receive_data(
                    writer_proxy,
                    &mut self.reader.reader_cache,
                    source_guid_prefix,
                    data,
                ),
                ReliabilityKind::Reliable => ReliableStatefulReaderBehavior::receive_data(
                    writer_proxy,
                    &mut self.reader.reader_cache,
                    source_guid_prefix,
                    data,
                ),
            }
        }
    }
}

impl RtpsEntityAttributes for RtpsStatefulReaderImpl {
    fn guid(&self) -> Guid {
        self.reader.endpoint.entity.guid
    }
}

impl RtpsEndpointAttributes for RtpsStatefulReaderImpl {
    fn topic_kind(&self) -> TopicKind {
        self.reader.endpoint.topic_kind
    }

    fn reliability_level(&self) -> ReliabilityKind {
        self.reader.endpoint.reliability_level
    }

    fn unicast_locator_list(&self) -> &[Locator] {
        &self.reader.endpoint.unicast_locator_list
    }

    fn multicast_locator_list(&self) -> &[Locator] {
        &self.reader.endpoint.multicast_locator_list
    }
}

impl RtpsReaderAttributes for RtpsStatefulReaderImpl {
    type HistoryCacheType = RtpsHistoryCacheImpl;

    fn heartbeat_response_delay(&self) -> Duration {
        self.reader.heartbeat_response_delay
    }

    fn heartbeat_suppression_duration(&self) -> Duration {
        self.reader.heartbeat_suppression_duration
    }

    fn reader_cache(&mut self) -> &mut Self::HistoryCacheType {
        &mut self.reader.reader_cache
    }

    fn expects_inline_qos(&self) -> bool {
        self.reader.expects_inline_qos
    }
}

impl RtpsStatefulReaderAttributes for RtpsStatefulReaderImpl {
    type WriterProxyType = RtpsWriterProxyImpl;

    fn matched_writers(&self) -> &[Self::WriterProxyType] {
        &self.matched_writers
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

#[cfg(test)]
mod tests {
    use rtps_pim::{
        behavior::reader::writer_proxy::RtpsWriterProxyConstructor,
        messages::submessage_elements::{
            EntityIdSubmessageElement, ParameterListSubmessageElement,
            SequenceNumberSubmessageElement, SerializedDataSubmessageElement,
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
            writer_sn: SequenceNumberSubmessageElement { value: 1 },
            inline_qos: ParameterListSubmessageElement { parameter: vec![] },
            serialized_payload: SerializedDataSubmessageElement {
                value: &[1, 0, 2, 5],
            },
        };

        reader.process_submessage(&data, source_guid.prefix);

        assert_eq!(1, reader.reader.reader_cache.changes().len());
        let change = &reader.reader.reader_cache.changes()[0];
        assert_eq!(source_guid, change.writer_guid);
        assert_eq!(data.writer_sn.value, change.sequence_number);
        assert_eq!(data.serialized_payload.value, change.data_value());
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

        reader.process_submessage(&make_data(2, &[2, 8, 4, 5]), writer_guid.prefix);
        reader.process_submessage(&make_data(0, &[2, 7, 1, 8]), writer_guid.prefix);
        reader.process_submessage(&make_data(3, &[9, 0, 4, 5]), writer_guid.prefix);
        reader.process_submessage(&make_data(1, &[2, 8, 1, 8]), writer_guid.prefix);

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

        reader.process_submessage(&make_data(2, &[2, 8, 4, 5]), writer_guid.prefix);
        reader.process_submessage(&make_data(0, &[2, 7, 1, 8]), writer_guid.prefix);
        reader.process_submessage(&make_data(3, &[9, 0, 4, 5]), writer_guid.prefix);
        reader.process_submessage(&make_data(1, &[2, 8, 1, 8]), writer_guid.prefix);

        assert_eq!(2, reader.reader_cache().changes().len());
    }
}
