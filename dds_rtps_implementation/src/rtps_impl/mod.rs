pub mod rtps_participant_impl;
pub mod rtps_reader_group_impl;
pub mod rtps_writer_group_impl;
// pub mod topic_impl;
pub mod rtps_writer_impl;

pub mod rtps_history_cache_impl;

pub mod rtps_message_sender;

pub trait PIM:
    rust_rtps_pim::structure::Types
    + rust_rtps_pim::behavior::Types
    + rust_rtps_pim::messages::Types
    + Sized
    + 'static
{
    type AckNackSubmessage: rust_rtps_pim::messages::submessages::AckNack<Self>;
    type DataSubmesage: rust_rtps_pim::messages::submessages::Data<Self>;
    type DataFrag: rust_rtps_pim::messages::submessages::DataFrag<Self>;
    type GapSubmessage: rust_rtps_pim::messages::submessages::Gap<Self>;
    type HeartbeatSubmessage: rust_rtps_pim::messages::submessages::Heartbeat<Self>;
    type HeartbeatFragSubmessage: rust_rtps_pim::messages::submessages::HeartbeatFrag<Self>;
    type InfoDestinationSubmessage: rust_rtps_pim::messages::submessages::InfoDestination<Self>;
    type InfoReplySubmessage: rust_rtps_pim::messages::submessages::InfoReply<Self>;
    type InfoSourceSubmessage: rust_rtps_pim::messages::submessages::InfoSource<Self>;
    type InfoTimestampSubmessage: rust_rtps_pim::messages::submessages::InfoTimestamp<Self>;
    type NackFragSubmessage: rust_rtps_pim::messages::submessages::NackFrag<Self>;
    type PadSubmessage: rust_rtps_pim::messages::submessages::Pad<Self>;
}
