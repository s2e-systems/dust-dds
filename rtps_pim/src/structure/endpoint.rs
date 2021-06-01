use super::{
    types::{EntityIdPIM, GUIDPIM, GuidPrefixPIM, LocatorPIM, ReliabilityKind, TopicKind},
    RTPSEntity,
};

pub trait RTPSEndpoint<PSM: GuidPrefixPIM + EntityIdPIM + LocatorPIM + GUIDPIM<PSM>>:
    RTPSEntity<PSM>
{
    fn topic_kind(&self) -> TopicKind;
    fn reliability_level(&self) -> ReliabilityKind;
    fn unicast_locator_list(&self) -> &[PSM::LocatorType];
    fn multicast_locator_list(&self) -> &[PSM::LocatorType];
}
