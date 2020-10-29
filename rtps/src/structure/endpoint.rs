use crate::structure::{RtpsEntity, RtpsRun, RtpsCommunication};
use crate::types::{Locator,GuidPrefix, ReliabilityKind, TopicKind};
use crate::messages::RtpsSubmessage;

pub trait RtpsEndpoint : RtpsEntity + RtpsRun + RtpsCommunication{
    fn unicast_locator_list(&self) -> Vec<Locator>;
    fn multicast_locator_list(&self) -> Vec<Locator>;
    fn reliability_level(&self) -> ReliabilityKind;
    fn topic_kind(&self) -> &TopicKind;
}