use super::{
    types::{Locator, ReliabilityKind, TopicKind},
};

pub trait RtpsEndpointAttributes {
    fn topic_kind(&self) -> &TopicKind;
    fn reliability_level(&self) -> &ReliabilityKind;
    fn unicast_locator_list(&self) -> &[Locator];
    fn multicast_locator_list(&self) -> &[Locator];
}
