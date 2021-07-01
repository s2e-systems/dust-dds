use rust_rtps_pim::{
    behavior::types::{DurationPIM, ParticipantMessageDataPIM},
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
        types::{CountPIM, GroupDigestPIM, ProtocolIdPIM, SubmessageKindPIM, TimePIM},
        RTPSMessagePIM, RtpsMessageHeaderPIM, RtpsSubmessageHeaderPIM,
    },
};

use crate::{message::RTPSMessageC, message_header::{ProtocolId, RTPSMessageHeader}, submessage_elements::{
        Count, Duration, EntityId, FragmentNumber, FragmentNumberSet, GroupDigest, GuidPrefix,
        LocatorList, Long, ParameterList, ProtocolVersion, SequenceNumber, SequenceNumberSet,
        SerializedData, Time, ULong, UShort, VendorId,
    }, submessage_header::SubmessageHeader, submessages};

#[derive(Debug, PartialEq)]
pub struct RtpsUdpPsm;

impl ProtocolIdPIM for RtpsUdpPsm {
    type ProtocolIdType = ProtocolId;
    const PROTOCOL_RTPS: Self::ProtocolIdType = [b'R', b'T', b'P', b'S'];
}

impl ParameterListSubmessageElementPIM for RtpsUdpPsm {
    type ParameterListSubmessageElementType = ParameterList;
}

pub(crate) type SubmessageKind = u8;

impl SubmessageKindPIM for RtpsUdpPsm {
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

impl TimePIM for RtpsUdpPsm {
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

impl CountPIM for RtpsUdpPsm {
    type CountType = Count;
}

impl GroupDigestPIM for RtpsUdpPsm {
    type GroupDigestType = GroupDigest;
}

impl DurationPIM for RtpsUdpPsm {
    type DurationType = Duration;
}

impl ParticipantMessageDataPIM for RtpsUdpPsm {
    type ParticipantMessageDataType = ();
}

impl UShortSubmessageElementPIM for RtpsUdpPsm {
    type UShortSubmessageElementType = UShort;
}

impl ULongSubmessageElementPIM for RtpsUdpPsm {
    type ULongSubmessageElementType = ULong;
}

impl LongSubmessageElementPIM for RtpsUdpPsm {
    type LongSubmessageElementType = Long;
}

impl EntityIdSubmessageElementPIM for RtpsUdpPsm {
    type EntityIdSubmessageElementType = EntityId;
}

impl GuidPrefixSubmessageElementPIM for RtpsUdpPsm {
    type GuidPrefixSubmessageElementType = GuidPrefix;
}

impl SequenceNumberSubmessageElementPIM for RtpsUdpPsm {
    type SequenceNumberSubmessageElementType = SequenceNumber;
}

impl SequenceNumberSetSubmessageElementPIM for RtpsUdpPsm {
    type SequenceNumberSetSubmessageElementType = SequenceNumberSet;
}

impl FragmentNumberSubmessageElementPIM for RtpsUdpPsm {
    type FragmentNumberSubmessageElementType = FragmentNumber;
}

impl FragmentNumberSetSubmessageElementPIM for RtpsUdpPsm {
    type FragmentNumberSetSubmessageElementType = FragmentNumberSet;
}

impl VendorIdSubmessageElementPIM for RtpsUdpPsm {
    type VendorIdSubmessageElementType = VendorId;
}

impl LocatorListSubmessageElementPIM for RtpsUdpPsm {
    type LocatorListSubmessageElementType = LocatorList;
}

impl ProtocolVersionSubmessageElementPIM for RtpsUdpPsm {
    type ProtocolVersionSubmessageElementType = ProtocolVersion;
}

impl TimestampSubmessageElementPIM for RtpsUdpPsm {
    type TimestampSubmessageElementType = Time;
}

impl<'a> SerializedDataSubmessageElementPIM<'a> for RtpsUdpPsm {
    type SerializedDataSubmessageElementType = SerializedData<'a>;
}

impl<'a> SerializedDataFragmentSubmessageElementPIM<'a> for RtpsUdpPsm {
    type SerializedDataFragmentSubmessageElementType = SerializedData<'a>;
}

impl CountSubmessageElementPIM for RtpsUdpPsm {
    type CountSubmessageElementType = Count;
}

impl<'a> RTPSMessagePIM<'a, Self> for RtpsUdpPsm {
    type RTPSMessageType = RTPSMessageC<'a>;
}

impl RtpsMessageHeaderPIM for RtpsUdpPsm {
    type RtpsMessageHeaderType = RTPSMessageHeader;
}

impl RtpsSubmessageHeaderPIM for RtpsUdpPsm {
    type RtpsSubmessageHeaderType = SubmessageHeader;
}

impl AckNackSubmessagePIM for RtpsUdpPsm {
    type AckNackSubmessageType = submessages::ack_nack::AckNack;
}

impl<'a> DataSubmessagePIM<'a, Self> for RtpsUdpPsm {
    type DataSubmessageType = submessages::data::DataSubmesage<'a>;
}

impl<'a> DataFragSubmessagePIM<'a> for RtpsUdpPsm {
    type DataFragSubmessageType = submessages::data_frag::DataFrag<'a>;
}

impl GapSubmessagePIM for RtpsUdpPsm {
    type GapSubmessageType = submessages::gap::GapSubmessage;
}

impl HeartbeatSubmessagePIM for RtpsUdpPsm {
    type HeartbeatSubmessageType = submessages::heartbeat::HeartbeatSubmessage;
}

impl HeartbeatFragSubmessagePIM for RtpsUdpPsm {
    type HeartbeatFragSubmessageType = submessages::heartbeat_frag::HeartbeatFrag;
}

impl InfoDestinationSubmessagePIM for RtpsUdpPsm {
    type InfoDestinationSubmessageType = submessages::info_destination::InfoDestination;
}

impl InfoReplySubmessagePIM for RtpsUdpPsm {
    type InfoReplySubmessageType = submessages::info_reply::InfoReply;
}

impl InfoSourceSubmessagePIM for RtpsUdpPsm {
    type InfoSourceSubmessageType = submessages::info_source::InfoSource;
}

impl InfoTimestampSubmessagePIM for RtpsUdpPsm {
    type InfoTimestampSubmessageType = submessages::info_timestamp::InfoTimestamp;
}

impl NackFragSubmessagePIM for RtpsUdpPsm {
    type NackFragSubmessageType = submessages::nack_frag::NackFrag;
}

impl PadSubmessagePIM for RtpsUdpPsm {
    type PadSubmessageType = submessages::pad::Pad;
}
