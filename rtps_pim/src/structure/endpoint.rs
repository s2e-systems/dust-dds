use crate::structure;

use super::{
    types::{ReliabilityKind, TopicKind},
    RTPSEntity,
};

pub trait RTPSEndpoint<PSM: structure::Types>: RTPSEntity<PSM> {
    fn topic_kind(&self) -> TopicKind;
    fn reliability_level(&self) -> ReliabilityKind;
    fn unicast_locator_list(&self) -> &[PSM::Locator];
    fn multicast_locator_list(&self) -> &[PSM::Locator];
}
