use rtps_pim::{
    behavior::{
        reader::stateless_reader::{RtpsStatelessReaderAttributes, RtpsStatelessReaderConstructor},
        stateless_reader_behavior::BestEffortStatelessReaderBehavior,
        types::Duration,
    },
    messages::{submessage_elements::Parameter, submessages::DataSubmessage},
    structure::types::{Guid, GuidPrefix, Locator, ReliabilityKind, TopicKind, ENTITYID_UNKNOWN},
};

use super::{rtps_endpoint_impl::RtpsEndpointImpl, rtps_reader_impl::RtpsReaderImpl};

pub type RtpsStatelessReaderImpl = RtpsReaderImpl;

impl RtpsStatelessReaderImpl {
    pub fn process_submessage(
        &mut self,
        data: &DataSubmessage<Vec<Parameter>, &[u8]>,
        source_guid_prefix: GuidPrefix,
    ) {
        if data.reader_id.value == ENTITYID_UNKNOWN
            || data.reader_id.value == self.endpoint.entity.guid.entity_id()
        {
            BestEffortStatelessReaderBehavior::receive_data(
                &mut self.reader_cache,
                source_guid_prefix,
                data,
            )
        }
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
        RtpsReaderImpl::new(
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
        )
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
            RtpsEndpointImpl::new(
                Guid::new(
                    GuidPrefix([3; 12]),
                    EntityId::new([4, 1, 3], USER_DEFINED_READER_NO_KEY),
                ),
                TopicKind::NoKey,
                ReliabilityKind::BestEffort,
                &[],
                &[],
            ),
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
                value: reader.guid().entity_id,
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

        assert_eq!(1, reader.reader_cache.changes().len());
        let change = &reader.reader_cache.changes()[0];
        assert_eq!(source_guid_prefix, change.writer_guid.prefix);
        assert_eq!(data.writer_sn.value, change.sequence_number);
        assert_eq!(data.serialized_payload.value, change.data_value());
    }
}
