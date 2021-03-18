use rust_rtps::{
    messages::submessages::Submessage,
    types::{GuidPrefix, Locator},
};

pub enum DestinedMessages<'a> {
    SingleDestination {
        locator: Locator,
        messages: Vec<&'a dyn Submessage>,
    },
    MultiDestination {
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        messages: Vec<&'a dyn Submessage>,
    },
}

// pub trait CacheChangeSender {
//     fn produce_messages(&mut self) -> Vec<DestinedMessages>;
// }

// pub trait CacheChangeReceiver {
//     fn try_process_message(
//         &mut self,
//         source_guid_prefix: GuidPrefix,
//         submessage: &mut Option<RtpsSubmessage>,
//     );
// }

// pub trait AcknowldegmentSender {
//     fn produce_messages(&mut self) -> Vec<DestinedMessages>;
// }

// pub trait AcknowldegmentReceiver {
//     fn try_process_message(
//         &mut self,
//         source_guid_prefix: GuidPrefix,
//         submessage: &mut Option<RtpsSubmessage>,
//     );
// }
