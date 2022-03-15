use rtps_pim::{
    behavior::{
        reader::{stateless_reader::{RtpsStatelessReaderAttributes, RtpsStatelessReaderConstructor}, reader::RtpsReaderAttributes},
        stateless_reader_behavior::BestEffortStatelessReaderBehavior,
        types::Duration,
    },
    messages::{submessage_elements::Parameter, submessages::DataSubmessage},
    structure::{types::{Guid, GuidPrefix, Locator, ReliabilityKind, TopicKind, ENTITYID_UNKNOWN}, entity::RtpsEntityAttributes, endpoint::RtpsEndpointAttributes},
};

use crate::rtps_history_cache_impl::RtpsHistoryCacheImpl;

use super::{rtps_endpoint_impl::RtpsEndpointImpl, rtps_reader_impl::RtpsReaderImpl};

pub struct RtpsStatelessReaderImpl(pub RtpsReaderImpl);

impl RtpsStatelessReaderImpl {
    pub fn process_submessage(
        &mut self,
        data: &DataSubmessage<Vec<Parameter>, &[u8]>,
        source_guid_prefix: GuidPrefix,
    ) {
        if data.reader_id.value == ENTITYID_UNKNOWN
            || data.reader_id.value == self.0.endpoint.entity.guid.entity_id()
        {
            BestEffortStatelessReaderBehavior::receive_data(
                &mut self.0.reader_cache,
                source_guid_prefix,
                data,
            )
        }
    }
}

impl RtpsEntityAttributes for RtpsStatelessReaderImpl {
    fn guid(&self) -> Guid {
        self.0.endpoint.entity.guid
    }
}

impl RtpsEndpointAttributes for RtpsStatelessReaderImpl {
    fn topic_kind(&self) -> TopicKind {
        self.0.endpoint.topic_kind
    }

    fn reliability_level(&self) -> ReliabilityKind {
        self.0.endpoint.reliability_level
    }

    fn unicast_locator_list(&self) -> &[Locator] {
        &self.0.endpoint.unicast_locator_list
    }

    fn multicast_locator_list(&self) -> &[Locator] {
        &self.0.endpoint.multicast_locator_list
    }
}

impl RtpsReaderAttributes for RtpsStatelessReaderImpl {
    type HistoryCacheType = RtpsHistoryCacheImpl;

    fn heartbeat_response_delay(&self) -> Duration {
        self.0.heartbeat_response_delay
    }

    fn heartbeat_suppression_duration(&self) -> Duration {
        self.0.heartbeat_suppression_duration
    }

    fn reader_cache(&mut self) -> &mut Self::HistoryCacheType {
        &mut self.0.reader_cache
    }

    fn expects_inline_qos(&self) -> bool {
        self.0.expects_inline_qos
    }
}

impl RtpsStatelessReaderAttributes for RtpsStatelessReaderImpl {}

impl RtpsStatelessReaderConstructor for RtpsStatelessReaderImpl {
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
        if reliability_level == ReliabilityKind::Reliable {
            panic!("Reliable stateless reader is not implemented");
        }

        Self(RtpsReaderImpl::new(
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
        ))
    }
}

#[cfg(test)]
mod tests {
    use rtps_pim::{
        messages::submessage_elements::{
            EntityIdSubmessageElement, ParameterListSubmessageElement,
            SequenceNumberSubmessageElement, SerializedDataSubmessageElement,
        },
        structure::{
            cache_change::RtpsCacheChangeAttributes,
            entity::RtpsEntityAttributes,
            history_cache::RtpsHistoryCacheAttributes,
            types::{EntityId, USER_DEFINED_READER_NO_KEY, USER_DEFINED_WRITER_NO_KEY},
        },
    };

    use super::*;

    #[test]
    fn process_submessage_test() {
        let mut reader = RtpsStatelessReaderImpl::new(
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

        let data: DataSubmessage<Vec<Parameter>, &[u8]> = DataSubmessage {
            endianness_flag: true,
            inline_qos_flag: true,
            data_flag: true,
            key_flag: false,
            non_standard_payload_flag: false,
            reader_id: EntityIdSubmessageElement {
                value: reader.0.guid().entity_id,
            },
            writer_id: EntityIdSubmessageElement {
                value: EntityId::new([6, 1, 2], USER_DEFINED_WRITER_NO_KEY),
            },
            writer_sn: SequenceNumberSubmessageElement { value: 1 },
            inline_qos: ParameterListSubmessageElement { parameter: vec![] },
            serialized_payload: SerializedDataSubmessageElement {
                value: &[1, 0, 2, 5],
            },
        };

        let source_guid_prefix = GuidPrefix([6; 12]);

        reader.process_submessage(&data, source_guid_prefix);

        assert_eq!(1, reader.0.reader_cache.changes().len());
        let change = &reader.0.reader_cache.changes()[0];
        assert_eq!(source_guid_prefix, change.writer_guid.prefix);
        assert_eq!(data.writer_sn.value, change.sequence_number);
        assert_eq!(data.serialized_payload.value, change.data_value());
    }

    #[test]
    #[should_panic]
    fn reliable_stateless_data_reader_not_constructible() {
        RtpsStatelessReaderImpl::new(
            Guid::new(
                GuidPrefix([3; 12]),
                EntityId::new([4, 1, 3], USER_DEFINED_READER_NO_KEY),
            ),
            TopicKind::NoKey,
            ReliabilityKind::Reliable,
            &[],
            &[],
            Duration::new(0, 0),
            Duration::new(0, 0),
            false,
        );
    }

    #[test]
    fn best_effort_stateless_data_reader_all_data_received_when_not_sent_in_order() {
        let reader_guid = Guid::new(
            GuidPrefix([3; 12]),
            EntityId::new([4, 1, 3], USER_DEFINED_READER_NO_KEY),
        );

        let source_guid = Guid::new(
            GuidPrefix([6; 12]),
            EntityId::new([6, 1, 2], USER_DEFINED_WRITER_NO_KEY),
        );

        let mut reader = RtpsStatelessReaderImpl::new(
            reader_guid,
            TopicKind::NoKey,
            ReliabilityKind::BestEffort,
            &[],
            &[],
            Duration::new(0, 0),
            Duration::new(0, 0),
            false,
        );

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
                value: source_guid.entity_id,
            },
            writer_sn: SequenceNumberSubmessageElement { value: seq_num },
            inline_qos: ParameterListSubmessageElement { parameter: vec![] },
            serialized_payload: SerializedDataSubmessageElement { value: data },
        };

        reader.process_submessage(&make_data(2, &[2, 8, 4, 5]), source_guid.prefix);
        reader.process_submessage(&make_data(0, &[2, 7, 1, 8]), source_guid.prefix);
        reader.process_submessage(&make_data(3, &[9, 0, 4, 5]), source_guid.prefix);
        reader.process_submessage(&make_data(1, &[2, 8, 1, 8]), source_guid.prefix);

        assert_eq!(4, reader.0.reader_cache.changes().len());
    }
}
