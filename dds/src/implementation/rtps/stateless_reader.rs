use std::convert::TryFrom;

use dds_transport::{messages::submessages::DataSubmessage, types::Locator};

use crate::{dcps_psm::Duration, implementation::rtps::history_cache::RtpsHistoryCacheImpl};

use super::{
    endpoint::RtpsEndpointImpl,
    reader::RtpsReaderImpl,
    types::{EntityId, Guid, GuidPrefix, ReliabilityKind, TopicKind, ENTITYID_UNKNOWN},
};

pub struct RtpsStatelessReaderImpl(RtpsReaderImpl);

impl RtpsStatelessReaderImpl {
    pub fn guid(&self) -> Guid {
        self.0.guid()
    }

    pub fn receive_data(&mut self, source_guid_prefix: GuidPrefix, data: &DataSubmessage<'_>) {
        if let Ok(a_change) = TryFrom::try_from((source_guid_prefix, data)) {
            self.reader_cache().add_change(a_change);
        }
    }
}

impl RtpsStatelessReaderImpl {
    pub fn topic_kind(&self) -> TopicKind {
        self.0.topic_kind()
    }

    pub fn reliability_level(&self) -> ReliabilityKind {
        self.0.reliability_level()
    }

    pub fn unicast_locator_list(&self) -> &[Locator] {
        self.0.unicast_locator_list()
    }

    pub fn multicast_locator_list(&self) -> &[Locator] {
        self.0.multicast_locator_list()
    }
}

impl RtpsStatelessReaderImpl {
    pub fn heartbeat_response_delay(&self) -> Duration {
        self.0.heartbeat_response_delay()
    }

    pub fn heartbeat_suppression_duration(&self) -> Duration {
        self.0.heartbeat_suppression_duration()
    }

    pub fn reader_cache(&mut self) -> &mut RtpsHistoryCacheImpl {
        self.0.reader_cache()
    }

    pub fn expects_inline_qos(&self) -> bool {
        self.0.expects_inline_qos()
    }
}

impl RtpsStatelessReaderImpl {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
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
            panic!("Reliable stateless reader is not supported");
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

impl RtpsStatelessReaderImpl {
    pub fn on_data_submessage_received(
        &mut self,
        data_submessage: &DataSubmessage<'_>,
        source_guid_prefix: GuidPrefix,
    ) {
        let data_reader_id: EntityId = data_submessage.reader_id.value.into();
        if data_reader_id == ENTITYID_UNKNOWN || data_reader_id == self.guid().entity_id() {
            self.receive_data(source_guid_prefix, data_submessage)
        }
    }
}

#[cfg(test)]
mod tests {
    use dds_transport::messages::submessage_elements::{
        EntityIdSubmessageElement, ParameterListSubmessageElement, SequenceNumberSubmessageElement,
        SerializedDataSubmessageElement,
    };

    use crate::implementation::rtps::types::{
        USER_DEFINED_READER_NO_KEY, USER_DEFINED_WRITER_NO_KEY,
    };

    use super::*;

    #[test]
    fn on_data_submessage_received_test() {
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

        let data: DataSubmessage<'_> = DataSubmessage {
            endianness_flag: true,
            inline_qos_flag: true,
            data_flag: true,
            key_flag: false,
            non_standard_payload_flag: false,
            reader_id: EntityIdSubmessageElement {
                value: reader.0.guid().entity_id.into(),
            },
            writer_id: EntityIdSubmessageElement {
                value: EntityId::new([6, 1, 2], USER_DEFINED_WRITER_NO_KEY).into(),
            },
            writer_sn: SequenceNumberSubmessageElement { value: 1 },
            inline_qos: ParameterListSubmessageElement { parameter: vec![] },
            serialized_payload: SerializedDataSubmessageElement {
                value: &[1, 0, 2, 5],
            },
        };

        let source_guid_prefix = GuidPrefix([6; 12]);

        reader.on_data_submessage_received(&data, source_guid_prefix);

        assert_eq!(1, reader.0.reader_cache().changes().len());
        let change = &reader.0.reader_cache().changes()[0];
        assert_eq!(source_guid_prefix, change.writer_guid().prefix);
        assert_eq!(data.writer_sn.value, change.sequence_number());
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
                value: reader_guid.entity_id.into(),
            },
            writer_id: EntityIdSubmessageElement {
                value: source_guid.entity_id.into(),
            },
            writer_sn: SequenceNumberSubmessageElement { value: seq_num },
            inline_qos: ParameterListSubmessageElement { parameter: vec![] },
            serialized_payload: SerializedDataSubmessageElement { value: data },
        };

        reader.on_data_submessage_received(&make_data(2, &[2, 8, 4, 5]), source_guid.prefix);
        reader.on_data_submessage_received(&make_data(0, &[2, 7, 1, 8]), source_guid.prefix);
        reader.on_data_submessage_received(&make_data(3, &[9, 0, 4, 5]), source_guid.prefix);
        reader.on_data_submessage_received(&make_data(1, &[2, 8, 1, 8]), source_guid.prefix);

        assert_eq!(4, reader.0.reader_cache().changes().len());
    }
}
