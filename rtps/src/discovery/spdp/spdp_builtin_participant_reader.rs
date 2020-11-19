
use crate::types::{GUID, ReliabilityKind, GuidPrefix};
use crate::types::constants::ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR;
use crate::structure::{RtpsEndpoint, RtpsEntity};
use crate::behavior::{StatelessReader, RtpsReader};

use rust_dds_interface::types::TopicKind;
use rust_dds_interface::history_cache::HistoryCache;

pub struct SpdpBuiltinParticipantReader;

impl SpdpBuiltinParticipantReader {
    pub fn new(guid_prefix: GuidPrefix) -> StatelessReader {

        let guid = GUID::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR);
        let entity = RtpsEntity::new(guid);
        
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::BestEffort;
        let endpoint = RtpsEndpoint::new(entity, topic_kind, reliability_level);
        let expects_inline_qos = false;

        let reader_cache = HistoryCache::default();
        let reader = RtpsReader::new(endpoint, reader_cache, expects_inline_qos);
        StatelessReader::new(reader)
    }
}