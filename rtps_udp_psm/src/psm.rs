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
            TimestampSubmessageElementPIM, ULongSubmessageElementPIM, UShortSubmessageElementPIM,
            VendorIdSubmessageElementPIM,
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
    message::RTPSMessageUdp,
    message_header::RTPSMessageHeaderUdp,
    submessage_elements::{
        CountUdp, EntityIdUdp, FragmentNumberSetUdp, FragmentNumberUdp, GroupDigestUdp, GuidPrefixUdp,
        LocatorListUdp, LongUdp, ParameterListUdp, ProtocolVersionUdp, SequenceNumberUdp,
        SequenceNumberSetUdp, SerializedDataUdp, TimeUdp, ULongUdp, UShortUdp, VendorIdUdp,
    },
    submessages,
};

#[derive(Debug, PartialEq)]
pub struct RtpsUdpPsm<'a>(&'a ());

impl<'a> ParameterListSubmessageElementPIM for RtpsUdpPsm<'a> {
    type ParameterListSubmessageElementType = ParameterListUdp;
}

pub(crate) type SubmessageKind = u8;

impl<'a> SubmessageKindPIM for RtpsUdpPsm<'a> {
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

impl<'a> TimePIM for RtpsUdpPsm<'a> {
    type TimeType = TimeUdp;
    const TIME_ZERO: Self::TimeType = TimeUdp {
        seconds: 0,
        fraction: 0,
    };
    const TIME_INVALID: Self::TimeType = TimeUdp {
        seconds: 0xffffffff,
        fraction: 0xffffffff,
    };
    const TIME_INFINITE: Self::TimeType = TimeUdp {
        seconds: 0xffffffff,
        fraction: 0xfffffffe,
    };
}

impl<'a> CountPIM for RtpsUdpPsm<'a> {
    type CountType = CountUdp;
}

impl<'a> GroupDigestPIM for RtpsUdpPsm<'a> {
    type GroupDigestType = GroupDigestUdp;
}

impl<'a> ParticipantMessageDataPIM for RtpsUdpPsm<'a> {
    type ParticipantMessageDataType = ();
}

impl<'a> UShortSubmessageElementPIM for RtpsUdpPsm<'a> {
    type UShortSubmessageElementType = UShortUdp;
}

impl<'a> ULongSubmessageElementPIM for RtpsUdpPsm<'a> {
    type ULongSubmessageElementType = ULongUdp;
}

impl<'a> LongSubmessageElementPIM for RtpsUdpPsm<'a> {
    type LongSubmessageElementType = LongUdp;
}

impl<'a> EntityIdSubmessageElementPIM for RtpsUdpPsm<'a> {
    type EntityIdSubmessageElementType = EntityIdUdp;
}

impl<'a> GuidPrefixSubmessageElementPIM for RtpsUdpPsm<'a> {
    type GuidPrefixSubmessageElementType = GuidPrefixUdp;
}

impl<'a> SequenceNumberSubmessageElementPIM for RtpsUdpPsm<'a> {
    type SequenceNumberSubmessageElementType = SequenceNumberUdp;
}

impl<'a> SequenceNumberSetSubmessageElementPIM for RtpsUdpPsm<'a> {
    type SequenceNumberSetSubmessageElementType = SequenceNumberSetUdp;
}

impl<'a> FragmentNumberSubmessageElementPIM for RtpsUdpPsm<'a> {
    type FragmentNumberSubmessageElementType = FragmentNumberUdp;
}

impl<'a> FragmentNumberSetSubmessageElementPIM for RtpsUdpPsm<'a> {
    type FragmentNumberSetSubmessageElementType = FragmentNumberSetUdp;
}

impl<'a> VendorIdSubmessageElementPIM for RtpsUdpPsm<'a> {
    type VendorIdSubmessageElementType = VendorIdUdp;
}

impl<'a> LocatorListSubmessageElementPIM for RtpsUdpPsm<'a> {
    type LocatorListSubmessageElementType = LocatorListUdp;
}

impl<'a> ProtocolVersionSubmessageElementPIM for RtpsUdpPsm<'a> {
    type ProtocolVersionSubmessageElementType = ProtocolVersionUdp;
}

impl<'a> TimestampSubmessageElementPIM for RtpsUdpPsm<'a> {
    type TimestampSubmessageElementType = TimeUdp;
}

// impl<'a> SerializedDataSubmessageElementPIM for RtpsUdpPsm<'a>{
//     type SerializedDataSubmessageElementType = SerializedData<'a>;
// }

impl<'a> SerializedDataFragmentSubmessageElementPIM for RtpsUdpPsm<'a> {
    type SerializedDataFragmentSubmessageElementType = SerializedDataUdp<'a>;
}

impl<'a> CountSubmessageElementPIM for RtpsUdpPsm<'a> {
    type CountSubmessageElementType = CountUdp;
}

impl<'a> RTPSMessagePIM<'a> for RtpsUdpPsm<'a> {
    type RTPSMessageType = RTPSMessageUdp<'a>;
}

impl<'a> RtpsMessageHeaderPIM for RtpsUdpPsm<'a> {
    type RtpsMessageHeaderType = RTPSMessageHeaderUdp;
}

impl<'a> AckNackSubmessagePIM for RtpsUdpPsm<'a> {
    type AckNackSubmessageType = submessages::ack_nack::AckNackUdp;
}

impl<'a> DataSubmessagePIM<'a> for RtpsUdpPsm<'a> {
    type DataSubmessageType = submessages::data::DataSubmesageUdp<'a>;
}

impl<'a> DataFragSubmessagePIM for RtpsUdpPsm<'a> {
    type DataFragSubmessageType = submessages::data_frag::DataFragUdp<'a>;
}

impl<'a> GapSubmessagePIM for RtpsUdpPsm<'a> {
    type GapSubmessageType = submessages::gap::GapSubmessageUdp;
}

impl<'a> HeartbeatSubmessagePIM for RtpsUdpPsm<'a> {
    type HeartbeatSubmessageType = submessages::heartbeat::HeartbeatSubmessageUdp;
}

impl<'a> HeartbeatFragSubmessagePIM for RtpsUdpPsm<'a> {
    type HeartbeatFragSubmessageType = submessages::heartbeat_frag::HeartbeatFragUdp;
}

impl<'a> InfoDestinationSubmessagePIM for RtpsUdpPsm<'a> {
    type InfoDestinationSubmessageType = submessages::info_destination::InfoDestinationUdp;
}

impl<'a> InfoReplySubmessagePIM for RtpsUdpPsm<'a> {
    type InfoReplySubmessageType = submessages::info_reply::InfoReplyUdp;
}

impl<'a> InfoSourceSubmessagePIM for RtpsUdpPsm<'a> {
    type InfoSourceSubmessageType = submessages::info_source::InfoSourceUdp;
}

impl<'a> InfoTimestampSubmessagePIM for RtpsUdpPsm<'a> {
    type InfoTimestampSubmessageType = submessages::info_timestamp::InfoTimestampUdp;
}

impl<'a> NackFragSubmessagePIM for RtpsUdpPsm<'a> {
    type NackFragSubmessageType = submessages::nack_frag::NackFragUdp;
}

impl<'a> PadSubmessagePIM for RtpsUdpPsm<'a> {
    type PadSubmessageType = submessages::pad::PadUdp;
}
