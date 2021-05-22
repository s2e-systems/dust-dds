#![no_std]

pub mod behavior;
pub mod messages;
pub mod structure;
// pub mod discovery;

pub trait PIM: structure::Types + behavior::Types + messages::Types + Sized + 'static {
    type AckNackSubmessage: messages::submessages::AckNack<Self>;
    type DataSubmesage: messages::submessages::Data<Self>;
    type DataFrag: messages::submessages::DataFrag<Self>;
    type GapSubmessage: messages::submessages::Gap<Self>;
    type HeartbeatSubmessage: messages::submessages::Heartbeat<Self>;
    type HeartbeatFragSubmessage: messages::submessages::HeartbeatFrag<Self>;
    type InfoDestinationSubmessage: messages::submessages::InfoDestination<Self>;
    type InfoReplySubmessage: messages::submessages::InfoReply<Self>;
    type InfoSourceSubmessage: messages::submessages::InfoSource<Self>;
    type InfoTimestampSubmessage: messages::submessages::InfoTimestamp<Self>;
    type NackFragSubmessage: messages::submessages::NackFrag<Self>;
    type PadSubmessage: messages::submessages::Pad<Self>;
}

#[cfg(test)]
#[macro_use]
extern crate std;
#[cfg(test)]
#[macro_use]
extern crate alloc;
