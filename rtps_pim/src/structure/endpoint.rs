use super::types::{LocatorPIM, ReliabilityKind, TopicKind};

pub trait RTPSEndpoint<PSM> {
    fn topic_kind(&self) -> &TopicKind;
    fn reliability_level(&self) -> &ReliabilityKind;
    fn unicast_locator_list(&self) -> &[PSM::LocatorType]
    where
        PSM: LocatorPIM;
    fn multicast_locator_list(&self) -> &[PSM::LocatorType]
    where
        PSM: LocatorPIM;
}
