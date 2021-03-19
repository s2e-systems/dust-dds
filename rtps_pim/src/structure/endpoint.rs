use crate::types::{Locator, ReliabilityKind, TopicKind};

use super::RTPSEntity;

pub trait RTPSEndpoint: RTPSEntity {
    type Locator: Locator;

    fn unicast_locator_list(&self) -> &[Self::Locator];
    fn multicast_locator_list(&self) -> &[Self::Locator];
    fn topic_kind(&self) -> &TopicKind;
    fn reliability_level(&self) -> &ReliabilityKind;
}
