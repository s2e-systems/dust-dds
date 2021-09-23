use super::{
    types::{ReliabilityKind, TopicKind},
    RtpsEntity,
};

pub struct RtpsEndpoint<L> {
    pub entity: RtpsEntity,
    pub topic_kind: TopicKind,
    pub reliability_level: ReliabilityKind,
    pub unicast_locator_list: L,
    pub multicast_locator_list: L,
}
