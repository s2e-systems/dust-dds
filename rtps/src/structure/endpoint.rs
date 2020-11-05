use crate::structure::RtpsEntity;
use crate::types::{Locator, ReliabilityKind};
use rust_dds_interface::types::TopicKind;

pub trait RtpsEndpoint : RtpsEntity  {
    fn unicast_locator_list(&self) -> Vec<Locator>;
    fn multicast_locator_list(&self) -> Vec<Locator>;
    fn reliability_level(&self) -> ReliabilityKind;
    fn topic_kind(&self) -> &TopicKind;
    
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_mut_any(&mut self) -> &mut dyn std::any::Any;
}

impl dyn RtpsEndpoint {
    pub fn get<T:RtpsEndpoint>(&self) -> Option<&T> {
        self.as_any().downcast_ref()
    }

    pub fn get_mut<T:RtpsEndpoint>(&mut self) -> Option<&mut T> {
        self.as_mut_any().downcast_mut()
    }
}