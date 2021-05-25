use types::{
    Count, Duration, EntityId, FragmentNumber, GroupDigest, GuidPrefix, InstanceHandle, Parameter,
    ParameterId, ProtocolId, ProtocolVersion, SequenceNumber, SequenceNumberSet, SubmessageFlag,
    Time, VendorId,
};

pub mod submessages;
pub mod types;

pub struct RtpsUdpPsm;

// impl rust_rtps_pim::PIM for RtpsUdpPsm {
//     type AckNackSubmessage = submessages::ack_nack::AckNack;
//     type DataSubmesage = submessages::data::Data;
//     type DataFrag = submessages::data_frag::DataFrag;
//     type GapSubmessage = submessages::gap::Gap;
//     type HeartbeatSubmessage = submessages::heartbeat::Heartbeat;
//     type HeartbeatFragSubmessage = submessages::heartbeat_frag::HeartbeatFrag;
//     type InfoDestinationSubmessage = submessages::info_destination::InfoDestination;
//     type InfoReplySubmessage = submessages::info_reply::InfoReply;
//     type InfoSourceSubmessage = submessages::info_source::InfoSource;
//     type InfoTimestampSubmessage = submessages::info_timestamp::InfoTimestamp;
//     type NackFragSubmessage = submessages::nack_frag::NackFrag;
//     type PadSubmessage = submessages::pad::Pad;
// }

impl rust_rtps_pim::structure::Types for RtpsUdpPsm {
    type GuidPrefix = GuidPrefix;
    const GUIDPREFIX_UNKNOWN: Self::GuidPrefix = [0; 12];

    type EntityId = EntityId;
    const ENTITYID_UNKNOWN: Self::EntityId = EntityId {
        entity_key: [0; 3],
        entity_kind: 0,
    };

    const ENTITYID_PARTICIPANT: Self::EntityId = EntityId {
        entity_key: [0, 0, 0x01],
        entity_kind: 0xc1,
    };

    type SequenceNumber = SequenceNumber;
    const SEQUENCE_NUMBER_UNKNOWN: Self::SequenceNumber = SequenceNumber {
        high: core::i32::MIN,
        low: core::u32::MAX,
    };

    type LocatorKind = i32;
    type LocatorPort = u32;
    type LocatorAddress = [u8; 16];

    const LOCATOR_KIND_INVALID: Self::LocatorKind = -1;
    const LOCATOR_KIND_RESERVED: Self::LocatorKind = 0;
    #[allow(non_upper_case_globals)]
    const LOCATOR_KIND_UDPv4: Self::LocatorKind = 1;
    #[allow(non_upper_case_globals)]
    const LOCATOR_KIND_UDPv6: Self::LocatorKind = 2;
    const LOCATOR_ADDRESS_INVALID: Self::LocatorAddress = [0; 16];
    const LOCATOR_PORT_INVALID: Self::LocatorPort = 0;

    type InstanceHandle = InstanceHandle;

    type ProtocolVersion = ProtocolVersion;
    const PROTOCOLVERSION: Self::ProtocolVersion = Self::PROTOCOLVERSION_2_4;
    const PROTOCOLVERSION_1_0: Self::ProtocolVersion = ProtocolVersion { major: 1, minor: 0 };
    const PROTOCOLVERSION_1_1: Self::ProtocolVersion = ProtocolVersion { major: 1, minor: 1 };
    const PROTOCOLVERSION_2_0: Self::ProtocolVersion = ProtocolVersion { major: 2, minor: 0 };
    const PROTOCOLVERSION_2_1: Self::ProtocolVersion = ProtocolVersion { major: 2, minor: 1 };
    const PROTOCOLVERSION_2_2: Self::ProtocolVersion = ProtocolVersion { major: 2, minor: 2 };
    const PROTOCOLVERSION_2_3: Self::ProtocolVersion = ProtocolVersion { major: 2, minor: 3 };
    const PROTOCOLVERSION_2_4: Self::ProtocolVersion = ProtocolVersion { major: 2, minor: 4 };

    type VendorId = VendorId;
    const VENDOR_ID_UNKNOWN: Self::VendorId = [0; 2];

    type Data = Vec<u8>;
    type Locator = rust_rtps_pim::structure::types::Locator<RtpsUdpPsm>;
    type LocatorVector = Vec<Self::Locator>;
    type SequenceNumberVector = SequenceNumberSet;
    type Parameter = Parameter;
    type ParameterVector = Vec<Parameter>;
}

impl rust_rtps_pim::messages::Types for RtpsUdpPsm {
    type ProtocolId = ProtocolId;
    const PROTOCOL_RTPS: Self::ProtocolId = [b'R', b'T', b'P', b'S'];

    type SubmessageFlag = SubmessageFlag;

    type SubmessageKind = u8;
    const DATA: Self::SubmessageKind = 0x15;
    const GAP: Self::SubmessageKind = 0x08;
    const HEARTBEAT: Self::SubmessageKind = 0x07;
    const ACKNACK: Self::SubmessageKind = 0x06;
    const PAD: Self::SubmessageKind = 0x01;
    const INFO_TS: Self::SubmessageKind = 0x09;
    const INFO_REPLY: Self::SubmessageKind = 0x0f;
    const INFO_DST: Self::SubmessageKind = 0x0e;
    const INFO_SRC: Self::SubmessageKind = 0x0c;
    const DATA_FRAG: Self::SubmessageKind = 0x16;
    const NACK_FRAG: Self::SubmessageKind = 0x12;
    const HEARTBEAT_FRAG: Self::SubmessageKind = 0x13;

    type Time = Time;

    const TIME_ZERO: Self::Time = Time {
        seconds: 0,
        fraction: 0,
    };
    const TIME_INVALID: Self::Time = Time {
        seconds: 0xffffffff,
        fraction: 0xffffffff,
    };
    const TIME_INFINITE: Self::Time = Time {
        seconds: 0xffffffff,
        fraction: 0xfffffffe,
    };

    type Count = Count;

    type ParameterId = ParameterId;

    type FragmentNumber = FragmentNumber;

    type GroupDigest = GroupDigest;

    type FragmentNumberVector = Vec<FragmentNumber>;
}

impl rust_rtps_pim::behavior::Types for RtpsUdpPsm {
    type Duration = Duration;

    type ParticipantMessageData = ();
}
