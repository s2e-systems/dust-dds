use super::{
    types::{Locator, ReliabilityKind, TopicKind},
    entity::RtpsEntityAttributes
};

pub trait RtpsEndpointAttributes: RtpsEntityAttributes {
    fn topic_kind(&self) -> &TopicKind;
    fn reliability_level(&self) -> &ReliabilityKind;
    fn unicast_locator_list(&self) -> &[Locator];
    fn multicast_locator_list(&self) -> &[Locator];
}
