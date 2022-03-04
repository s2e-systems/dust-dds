use super::types::{Locator, ReliabilityKind, TopicKind};

pub trait RtpsEndpointAttributes {
    fn unicast_locator_list(&self) -> &[Locator];
    fn multicast_locator_list(&self) -> &[Locator];
    fn reliability_level(&self) -> ReliabilityKind;
    fn topic_kind(&self) -> TopicKind;
}
