
use crate::types::{GUID, ReliabilityKind, GuidPrefix};
use crate::types::constants::ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR;
use crate::structure::{RtpsEndpoint, RtpsEntity};
use crate::behavior::{StatelessReader, RtpsReader};

use rust_dds_interface::types::TopicKind;
use rust_dds_interface::history_cache::HistoryCache;

pub struct SpdpBuiltinParticipantWriter {
    stateless_reader: StatelessReader,
    reader_cache: HistoryCache,
}

impl SpdpBuiltinParticipantWriter {
    pub fn new(guid_prefix: GuidPrefix) -> Self {

        let guid = GUID::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR);
        let entity = RtpsEntity::new(guid);
        
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::BestEffort;
        let endpoint = RtpsEndpoint::new(entity, topic_kind, reliability_level);
        let expects_inline_qos = false;

        let reader = RtpsReader::new(endpoint, expects_inline_qos);
        let stateless_reader = StatelessReader::new(reader);

        let reader_cache = HistoryCache::default();

        Self {
            stateless_reader,
            reader_cache
        }
    }
}