use crate::rtps::behavior::cache_change_from_data;
use crate::rtps::behavior::endpoint_traits::CacheChangeReceiver;
use crate::rtps::behavior::RtpsReader;
use crate::rtps::messages::submessages::Data;
use crate::rtps::messages::RtpsSubmessage;
use crate::rtps::types::constants::ENTITYID_UNKNOWN;
use crate::rtps::types::{GuidPrefix, ReliabilityKind, GUID};
use crate::rtps::structure::HistoryCache;
use crate::types::TopicKind;

pub struct StatelessReader {
    pub reader: RtpsReader,
}

impl StatelessReader {
    pub fn new(
        guid: GUID,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        reader_cache: HistoryCache,
        expects_inline_qos: bool,
    ) -> Self {
        assert!(
            reliability_level == ReliabilityKind::BestEffort,
            "Only BestEffort supported on stateless reader"
        );

        let reader = RtpsReader::new(
            guid,
            topic_kind,
            reliability_level,
            reader_cache,
            expects_inline_qos,
        );
        Self { reader }
    }

    fn waiting_state(
        &mut self,
        source_guid_prefix: GuidPrefix,
        submessage: &mut Option<RtpsSubmessage>,
    ) {
        if let Some(inner_submessage) = submessage {
            if let RtpsSubmessage::Data(data) = inner_submessage {
                if self.reader.endpoint.entity.guid.entity_id() == data.reader_id()
                    || data.reader_id() == ENTITYID_UNKNOWN
                {
                    if let RtpsSubmessage::Data(data) = submessage.take().unwrap() {
                        self.transition_t2(source_guid_prefix, data)
                    }
                }
            }
        }
    }

    fn transition_t2(&mut self, guid_prefix: GuidPrefix, data: Data) {
        let cache_change = cache_change_from_data(data, &guid_prefix);
        self.reader.reader_cache.add_change(cache_change);
    }
}

impl CacheChangeReceiver for StatelessReader {
    fn try_process_message(
        &mut self,
        source_guid_prefix: GuidPrefix,
        submessage: &mut Option<RtpsSubmessage>,
    ) {
        self.waiting_state(source_guid_prefix, submessage);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ChangeKind;
    use crate::rtps::types::constants::{
        ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER, ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
        ENTITYID_UNKNOWN,
    };

    use crate::rtps::messages::types::Endianness;
    use crate::rtps::messages::submessages::data_submessage::Payload;
    use crate::rtps::messages::submessages::submessage_elements::ParameterList;
    use crate::rtps::messages::types::KeyHash;
    use crate::rtps::behavior::change_kind_to_status_info;

    #[test]
    fn process_none_submessage() {
        let guid = GUID::new([1; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER);
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::BestEffort;
        let reader_cache = HistoryCache::default();
        let expects_inline_qos = false;
        let mut stateless_reader = StatelessReader::new(
            guid,
            topic_kind,
            reliability_level,
            reader_cache,
            expects_inline_qos,
        );

        let source_guid_prefix = [5; 12];
        let mut submessage = None;
        stateless_reader.try_process_message(source_guid_prefix, &mut submessage);
    }

    #[test]
    fn process_data_submessage() {
        let guid = GUID::new([1; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER);
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::BestEffort;
        let reader_cache = HistoryCache::default();
        let expects_inline_qos = false;
        let mut stateless_reader = StatelessReader::new(
            guid,
            topic_kind,
            reliability_level,
            reader_cache,
            expects_inline_qos,
        );

        let source_guid_prefix = [5; 12];
        let status_info = change_kind_to_status_info(ChangeKind::Alive);
        let key_hash = KeyHash([1;16]);
        let mut inline_qos = ParameterList::new();
        inline_qos.parameter.push(key_hash.into());
        inline_qos.parameter.push(status_info.into());
        let data_submessage = Data::new(
            Endianness::LittleEndian,
            ENTITYID_UNKNOWN,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
            1,
            Some(inline_qos),
            Payload::Data(vec![0, 1, 2]),
        );
        let expected_cache_change = cache_change_from_data(data_submessage.clone(), &source_guid_prefix);

        stateless_reader.try_process_message(source_guid_prefix, &mut Some(RtpsSubmessage::Data(data_submessage)));
        let received_change = stateless_reader.reader.reader_cache.get_change(1).unwrap();
        assert_eq!(received_change, &expected_cache_change);
    }
}
