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
            SerializedDataSubmessageElementPIM, TimestampSubmessageElementPIM,
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

use crate::{message::RTPSMessageC, message_header::{ProtocolId, RTPSMessageHeader}, submessage_elements::{Count, EntityId, FragmentNumber, FragmentNumberSet, GroupDigest, GuidPrefix, LocatorList, Long, ParameterList, ProtocolVersionC, SequenceNumber, SequenceNumberSet, SerializedData, Time, ULong, UShort, VendorId}, submessages};

#[derive(Debug, PartialEq)]
pub struct RtpsUdpPsm;

impl<'a> ParameterListSubmessageElementPIM for RtpsUdpPsm {
    type ParameterListSubmessageElementType = ParameterList;
}

pub(crate) type SubmessageKind = u8;

impl<'a> SubmessageKindPIM for RtpsUdpPsm {
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

impl<'a> TimePIM for RtpsUdpPsm {
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

impl<'a> CountPIM for RtpsUdpPsm {
    type CountType = Count;
}

impl<'a> GroupDigestPIM for RtpsUdpPsm {
    type GroupDigestType = GroupDigest;
}

impl<'a> ParticipantMessageDataPIM for RtpsUdpPsm {
    type ParticipantMessageDataType = ();
}

impl<'a> UShortSubmessageElementPIM for RtpsUdpPsm {
    type UShortSubmessageElementType = UShort;
}

impl<'a> ULongSubmessageElementPIM for RtpsUdpPsm {
    type ULongSubmessageElementType = ULong;
}

impl<'a> LongSubmessageElementPIM for RtpsUdpPsm {
    type LongSubmessageElementType = Long;
}

impl<'a> EntityIdSubmessageElementPIM for RtpsUdpPsm {
    type EntityIdSubmessageElementType = EntityId;
}

impl<'a> GuidPrefixSubmessageElementPIM for RtpsUdpPsm {
    type GuidPrefixSubmessageElementType = GuidPrefix;
}

impl<'a> SequenceNumberSubmessageElementPIM for RtpsUdpPsm {
    type SequenceNumberSubmessageElementType = SequenceNumber;
}

impl<'a> SequenceNumberSetSubmessageElementPIM for RtpsUdpPsm {
    type SequenceNumberSetSubmessageElementType = SequenceNumberSet;
}

impl<'a> FragmentNumberSubmessageElementPIM for RtpsUdpPsm {
    type FragmentNumberSubmessageElementType = FragmentNumber;
}

impl<'a> FragmentNumberSetSubmessageElementPIM for RtpsUdpPsm {
    type FragmentNumberSetSubmessageElementType = FragmentNumberSet;
}

impl<'a> VendorIdSubmessageElementPIM for RtpsUdpPsm {
    type VendorIdSubmessageElementType = VendorId;
}

impl<'a> LocatorListSubmessageElementPIM for RtpsUdpPsm {
    type LocatorListSubmessageElementType = LocatorList;
}

impl<'a> ProtocolVersionSubmessageElementPIM for RtpsUdpPsm {
    type ProtocolVersionSubmessageElementType = ProtocolVersionC;
}

impl<'a> TimestampSubmessageElementPIM for RtpsUdpPsm {
    type TimestampSubmessageElementType = Time;
}

impl<'a> SerializedDataSubmessageElementPIM<'a> for RtpsUdpPsm {
    type SerializedDataSubmessageElementType = SerializedData<'a>;
}

impl<'a> SerializedDataFragmentSubmessageElementPIM<'a> for RtpsUdpPsm {
    type SerializedDataFragmentSubmessageElementType = SerializedData<'a>;
}

impl<'a> CountSubmessageElementPIM for RtpsUdpPsm {
    type CountSubmessageElementType = Count;
}

impl<'a> RTPSMessagePIM<'a> for RtpsUdpPsm {
    type RTPSMessageType = RTPSMessageC<'a>;
}

impl<'a> RtpsMessageHeaderPIM for RtpsUdpPsm {
    type RtpsMessageHeaderType = RTPSMessageHeader;
}

impl<'a> AckNackSubmessagePIM for RtpsUdpPsm {
    type AckNackSubmessageType = submessages::ack_nack::AckNack;
}

impl<'a> DataSubmessagePIM<'a> for RtpsUdpPsm {
    type DataSubmessageType = submessages::data::DataSubmesage<'a>;
}

impl<'a> DataFragSubmessagePIM<'a> for RtpsUdpPsm {
    type DataFragSubmessageType = submessages::data_frag::DataFrag<'a>;
}

impl<'a> GapSubmessagePIM for RtpsUdpPsm {
    type GapSubmessageType = submessages::gap::GapSubmessage;
}

impl<'a> HeartbeatSubmessagePIM for RtpsUdpPsm {
    type HeartbeatSubmessageType = submessages::heartbeat::HeartbeatSubmessage;
}

impl<'a> HeartbeatFragSubmessagePIM for RtpsUdpPsm {
    type HeartbeatFragSubmessageType = submessages::heartbeat_frag::HeartbeatFrag;
}

impl<'a> InfoDestinationSubmessagePIM for RtpsUdpPsm {
    type InfoDestinationSubmessageType = submessages::info_destination::InfoDestination;
}

impl<'a> InfoReplySubmessagePIM for RtpsUdpPsm {
    type InfoReplySubmessageType = submessages::info_reply::InfoReply;
}

impl<'a> InfoSourceSubmessagePIM for RtpsUdpPsm {
    type InfoSourceSubmessageType = submessages::info_source::InfoSource;
}

impl<'a> InfoTimestampSubmessagePIM for RtpsUdpPsm {
    type InfoTimestampSubmessageType = submessages::info_timestamp::InfoTimestamp;
}

impl<'a> NackFragSubmessagePIM for RtpsUdpPsm {
    type NackFragSubmessageType = submessages::nack_frag::NackFrag;
}

impl<'a> PadSubmessagePIM for RtpsUdpPsm {
    type PadSubmessageType = submessages::pad::Pad;
}
