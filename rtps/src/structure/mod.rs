/// 
/// This module contains the elements described in Section 8.2 of the DDSI-RTPS version 2.3 standard
///  

mod cache_change;
mod history_cache;
mod entity;
mod participant;
mod group;
mod endpoint;

pub trait RtpsRun{
    fn run(&mut self);
}

pub trait RtpsCommunication {
    fn try_push_message(&mut self, src_locator: crate::types::Locator, src_guid_prefix: crate::types::GuidPrefix, submessage: &mut Option<crate::messages::RtpsSubmessage>);
}

pub use cache_change::CacheChange;
pub use history_cache::HistoryCache;
pub use entity::RtpsEntity;
pub use participant::RtpsParticipant;
pub use group::RtpsGroup;
pub use endpoint::RtpsEndpoint;