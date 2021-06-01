use super::{
    types::{EntityIdPIM, GuidPrefixPIM, LocatorPIM, ReliabilityKind, TopicKind, GUIDPIM},
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
