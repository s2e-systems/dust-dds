use rust_rtps_pim::{
    behavior::types::ParticipantMessageDataPIM,
    messages::{
        submessage_elements::{
            CountSubmessageElementPIM, EntityIdSubmessageElementPIM,
            FragmentNumberSetSubmessageElementPIM, FragmentNumberSubmessageElementPIM,
            GuidPrefixSubmessageElementPIM, LocatorListSubmessageElementPIM,
            LongSubmessageElementPIM, ParameterListSubmessageElementPIM,
            ProtocolVersionSubmessageElementPIM, SequenceNumberSetSubmessageElementPIM,
            SequenceNumberSubmessageElementPIM, SerializedDataFragmentSubmessageElementPIM,
            TimestampSubmessageElementPIM,
            ULongSubmessageElementPIM, UShortSubmessageElementPIM, VendorIdSubmessageElementPIM,
        },
        submessages::{
            AckNackSubmessagePIM, DataFragSubmessagePIM, DataSubmessagePIM, GapSubmessagePIM,
            HeartbeatFragSubmessagePIM, HeartbeatSubmessagePIM, InfoDestinationSubmessagePIM,
            InfoReplySubmessagePIM, InfoSourceSubmessagePIM, InfoTimestampSubmessagePIM,
            NackFragSubmessagePIM, PadSubmessagePIM,
        },
        types::{CountPIM, GroupDigestPIM, SubmessageKindPIM, TimePIM},
        RTPSMessagePIM, RtpsMessageHeaderPIM,
    },
};

use crate::{
    message::RTPSMessageC,
    message_header::{ProtocolId, RTPSMessageHeader},
    submessage_elements::{
        Count, EntityId, FragmentNumber, FragmentNumberSet, GroupDigest, GuidPrefix, LocatorList,
        Long, ParameterList, ProtocolVersionC, SequenceNumber, SequenceNumberSet, SerializedData,
        Time, ULong, UShort, VendorId,
    },
    submessages,
};

#[derive(Debug, PartialEq)]
pub struct RtpsUdpPsm<'a>(&'a ());

impl<'a> ParameterListSubmessageElementPIM for RtpsUdpPsm<'a>{
    type ParameterListSubmessageElementType = ParameterList;
}

pub(crate) type SubmessageKind = u8;

impl<'a> SubmessageKindPIM for RtpsUdpPsm<'a>{
    type SubmessageKindType = SubmessageKind;
    const DATA: Self::SubmessageKindType = 0x15;
    const GAP: Self::SubmessageKindType = 0x08;
    const HEARTBEAT: Self::SubmessageKindType = 0x07;
    const ACKNACK: Self::SubmessageKindType = 0x06;
    const PAD: Self::SubmessageKindType = 0x01;
    const INFO_TS: Self::SubmessageKindType = 0x09;
    const INFO_REPLY: Self::SubmessageKindType = 0x0f;
    const INFO_DST: Self::SubmessageKindType = 0x0e;
    const INFO_SRC: Self::SubmessageKindType = 0x0c;
    const DATA_FRAG: Self::SubmessageKindType = 0x16;
    const NACK_FRAG: Self::SubmessageKindType = 0x12;
    const HEARTBEAT_FRAG: Self::SubmessageKindType = 0x13;
}

impl<'a> TimePIM for RtpsUdpPsm<'a>{
    type TimeType = Time;
    const TIME_ZERO: Self::TimeType = Time {
        seconds: 0,
        fraction: 0,
    };
    const TIME_INVALID: Self::TimeType = Time {
        seconds: 0xffffffff,
        fraction: 0xffffffff,
    };
    const TIME_INFINITE: Self::TimeType = Time {
        seconds: 0xffffffff,
        fraction: 0xfffffffe,
    };
}

impl<'a> CountPIM for RtpsUdpPsm<'a>{
    type CountType = Count;
}

impl<'a> GroupDigestPIM for RtpsUdpPsm<'a>{
    type GroupDigestType = GroupDigest;
}

impl<'a> ParticipantMessageDataPIM for RtpsUdpPsm<'a>{
    type ParticipantMessageDataType = ();
}

impl<'a> UShortSubmessageElementPIM for RtpsUdpPsm<'a>{
    type UShortSubmessageElementType = UShort;
}

impl<'a> ULongSubmessageElementPIM for RtpsUdpPsm<'a>{
    type ULongSubmessageElementType = ULong;
}

impl<'a> LongSubmessageElementPIM for RtpsUdpPsm<'a>{
    type LongSubmessageElementType = Long;
}

impl<'a> EntityIdSubmessageElementPIM for RtpsUdpPsm<'a>{
    type EntityIdSubmessageElementType = EntityId;
}

impl<'a> GuidPrefixSubmessageElementPIM for RtpsUdpPsm<'a>{
    type GuidPrefixSubmessageElementType = GuidPrefix;
}

impl<'a> SequenceNumberSubmessageElementPIM for RtpsUdpPsm<'a>{
    type SequenceNumberSubmessageElementType = SequenceNumber;
}

impl<'a> SequenceNumberSetSubmessageElementPIM for RtpsUdpPsm<'a>{
    type SequenceNumberSetSubmessageElementType = SequenceNumberSet;
}

impl<'a> FragmentNumberSubmessageElementPIM for RtpsUdpPsm<'a>{
    type FragmentNumberSubmessageElementType = FragmentNumber;
}

impl<'a> FragmentNumberSetSubmessageElementPIM for RtpsUdpPsm<'a>{
    type FragmentNumberSetSubmessageElementType = FragmentNumberSet;
}

impl<'a> VendorIdSubmessageElementPIM for RtpsUdpPsm<'a>{
    type VendorIdSubmessageElementType = VendorId;
}

impl<'a> LocatorListSubmessageElementPIM for RtpsUdpPsm<'a>{
    type LocatorListSubmessageElementType = LocatorList;
}

impl<'a> ProtocolVersionSubmessageElementPIM for RtpsUdpPsm<'a>{
    type ProtocolVersionSubmessageElementType = ProtocolVersionC;
}

impl<'a> TimestampSubmessageElementPIM for RtpsUdpPsm<'a>{
    type TimestampSubmessageElementType = Time;
}

// impl<'a> SerializedDataSubmessageElementPIM for RtpsUdpPsm<'a>{
//     type SerializedDataSubmessageElementType = SerializedData<'a>;
// }

impl<'a> SerializedDataFragmentSubmessageElementPIM for RtpsUdpPsm<'a>{
    type SerializedDataFragmentSubmessageElementType = SerializedData<'a>;
}

impl<'a> CountSubmessageElementPIM for RtpsUdpPsm<'a>{
    type CountSubmessageElementType = Count;
}

impl<'a> RTPSMessagePIM<'a> for RtpsUdpPsm<'a> {
    type RTPSMessageType = RTPSMessageC<'a>;
}

impl<'a> RtpsMessageHeaderPIM for RtpsUdpPsm<'a>{
    type RtpsMessageHeaderType = RTPSMessageHeader;
}

impl<'a> AckNackSubmessagePIM for RtpsUdpPsm<'a>{
    type AckNackSubmessageType = submessages::ack_nack::AckNack;
}

impl<'a> DataSubmessagePIM<'a> for RtpsUdpPsm<'a> {
    type DataSubmessageType = submessages::data::DataSubmesage<'a>;
}

impl<'a> DataFragSubmessagePIM for RtpsUdpPsm<'a> {
    type DataFragSubmessageType = submessages::data_frag::DataFrag<'a>;
}

impl<'a> GapSubmessagePIM for RtpsUdpPsm<'a>{
    type GapSubmessageType = submessages::gap::GapSubmessage;
}

impl<'a> HeartbeatSubmessagePIM for RtpsUdpPsm<'a>{
    type HeartbeatSubmessageType = submessages::heartbeat::HeartbeatSubmessage;
}

impl<'a> HeartbeatFragSubmessagePIM for RtpsUdpPsm<'a>{
    type HeartbeatFragSubmessageType = submessages::heartbeat_frag::HeartbeatFrag;
}

impl<'a> InfoDestinationSubmessagePIM for RtpsUdpPsm<'a>{
    type InfoDestinationSubmessageType = submessages::info_destination::InfoDestination;
}

impl<'a> InfoReplySubmessagePIM for RtpsUdpPsm<'a>{
    type InfoReplySubmessageType = submessages::info_reply::InfoReply;
}

impl<'a> InfoSourceSubmessagePIM for RtpsUdpPsm<'a>{
    type InfoSourceSubmessageType = submessages::info_source::InfoSource;
}

impl<'a> InfoTimestampSubmessagePIM for RtpsUdpPsm<'a>{
    type InfoTimestampSubmessageType = submessages::info_timestamp::InfoTimestamp;
}

impl<'a> NackFragSubmessagePIM for RtpsUdpPsm<'a>{
    type NackFragSubmessageType = submessages::nack_frag::NackFrag;
}

impl<'a> PadSubmessagePIM for RtpsUdpPsm<'a>{
    type PadSubmessageType = submessages::pad::Pad;
}
