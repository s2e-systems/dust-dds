
use crate::types::{GUID, ReliabilityKind, GuidPrefix};
use crate::types::constants::ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER;
use crate::structure::{RtpsEndpoint, RtpsEntity};
use crate::behavior::types::Duration;
use crate::behavior::{StatefulWriter, RtpsWriter};

use rust_dds_interface::types::TopicKind;
use rust_dds_interface::history_cache::HistoryCache;

pub struct SedpBuiltinPublicationWriter;

impl SedpBuiltinPublicationWriter {
    pub fn new(guid_prefix: GuidPrefix) -> StatefulWriter {
        let sedp_builtin_publications_writer_guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER);

        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::Reliable;

        let push_mode = true;
        let writer_cache = HistoryCache::default();
        let data_max_sized_serialized = None;
        let heartbeat_period = Duration::from_millis(100);
        let nack_response_delay = Duration::from_millis(100);
        let nack_suppression_duration = Duration::from_millis(100);

        StatefulWriter::new(
            sedp_builtin_publications_writer_guid, topic_kind, reliability_level, push_mode, writer_cache, data_max_sized_serialized,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration
        )
    }
}