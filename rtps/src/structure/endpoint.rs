use crate::structure::RtpsEntity;
use crate::types::ReliabilityKind;
use rust_dds_interface::types::TopicKind;

pub struct RtpsEndpoint {
    pub entity: RtpsEntity,
    pub topic_kind: TopicKind,
    pub reliability_level: ReliabilityKind,
}

impl RtpsEndpoint {
    pub fn new(entity: RtpsEntity, topic_kind: TopicKind, reliability_level: ReliabilityKind) -> Self {
        Self {
            entity,
            topic_kind,
            reliability_level,
        }
    }
}