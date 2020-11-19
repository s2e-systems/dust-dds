
use crate::types::{GUID, ReliabilityKind, GuidPrefix};
use crate::types::constants::ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER;
use crate::structure::{RtpsEndpoint, RtpsEntity};
use crate::behavior::{StatelessWriter, RtpsWriter};

use rust_dds_interface::types::TopicKind;
use rust_dds_interface::history_cache::HistoryCache;

pub struct SpdpBuiltinParticipantWriter {
    stateless_writer: StatelessWriter,
}

impl SpdpBuiltinParticipantWriter {
    pub fn new(guid_prefix: GuidPrefix) -> Self {
        let guid = GUID::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER);
        let entity = RtpsEntity::new(guid);
        
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::BestEffort;
        let endpoint = RtpsEndpoint::new(entity, topic_kind, reliability_level);
        
        let push_mode = true;
        let writer_cache = HistoryCache::default();
        let data_max_sized_serialized = None;
        let writer = RtpsWriter::new(endpoint, push_mode, writer_cache, data_max_sized_serialized); 
        let stateless_writer = StatelessWriter::new(writer);

        Self {
            stateless_writer,
        }
    }
}
