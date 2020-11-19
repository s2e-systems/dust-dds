use std::sync::Mutex;

use crate::types::{Locator, GuidPrefix};
use crate::messages::RtpsSubmessage;

use rust_dds_interface::history_cache::HistoryCache;

pub enum DestinedMessages {
    SingleDestination{locator: Locator, messages: Vec<RtpsSubmessage>},
    MultiDestination{unicast_locator_list: Vec<Locator>, multicast_locator_list: Vec<Locator>, messages: Vec<RtpsSubmessage>}
}

pub trait CacheChangeSender {
    fn produce_messages(&self, writer_cache: &Mutex<HistoryCache>) -> Vec<DestinedMessages>;
}

pub trait CacheChangeReceiver {
    fn try_process_message(&mut self, source_guid_prefix: GuidPrefix, submessage: &mut Option<RtpsSubmessage>, reader_cache: &mut HistoryCache);
}

pub trait AcknowldegmentSender {
    fn produce_messages(&self, writer_cache: &Mutex<HistoryCache>) -> Vec<DestinedMessages>;
}

pub trait AcknowldegmentReceiver {
    fn try_process_message(&mut self, source_guid_prefix: GuidPrefix, submessage: &mut Option<RtpsSubmessage>, reader_cache: &mut HistoryCache);
}