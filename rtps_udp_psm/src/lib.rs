use rust_rtps_pim::RtpsPim;
use types::{
    ChangeForReaderStatusKind, ChangeFromWriterStatusKind, ChangeKind, Count, Duration, EntityId,
    FragmentNumber, GroupDigest, Guid, GuidPrefix, InstanceHandle, Locator, Parameter, ParameterId,
    ProtocolId, ProtocolVersion, ReliabilityKind, SequenceNumber, SequenceNumberSet,
    SubmessageFlag, Time, TopicKind, VendorId,
};

pub mod submessages;
pub mod types;

pub struct RtpsUdpPsm;

impl RtpsPim for RtpsUdpPsm {
    type Data = Vec<u8>;
    type SequenceNumberVector = SequenceNumberSet;
    type LocatorVector = Vec<Locator>;
    type FragmentNumberVector = Vec<FragmentNumber>;
    type Parameter = Parameter;
    type ParameterVector = Vec<Parameter>;
}

impl rust_rtps_pim::structure::Types for RtpsUdpPsm {
    type Guid = Guid;
    const GUID_UNKNOWN: Self::Guid = Guid {
        prefix: Self::GUIDPREFIX_UNKNOWN,
        entity_id: Self::ENTITYID_UNKNOWN,
    };

    type GuidPrefix = GuidPrefix;
    const GUIDPREFIX_UNKNOWN: Self::GuidPrefix = [0; 12];

    type EntityId = EntityId;
    const ENTITYID_UNKNOWN: Self::EntityId = EntityId {
        entity_key: [0; 3],
        entity_kind: 0,
    };

    type SequenceNumber = SequenceNumber;
    const SEQUENCE_NUMBER_UNKNOWN: Self::SequenceNumber = SequenceNumber {
        high: core::i32::MIN,
        low: core::u32::MAX,
    };

    type Locator = Locator;
    const LOCATOR_INVALID: Self::Locator = Locator {
        kind: <Locator as rust_rtps_pim::structure::types::Locator>::LOCATOR_KIND_INVALID,
        port: <Locator as rust_rtps_pim::structure::types::Locator>::LOCATOR_PORT_INVALID,
        address: <Locator as rust_rtps_pim::structure::types::Locator>::LOCATOR_ADDRESS_INVALID,
    };

    type TopicKind = TopicKind;
    const NO_KEY: Self::TopicKind = TopicKind::NoKey;
    const WITH_KEY: Self::TopicKind = TopicKind::WithKey;

    type ChangeKind = ChangeKind;
    const ALIVE: Self::ChangeKind = ChangeKind::Alive;
    const ALIVE_FILTERED: Self::ChangeKind = ChangeKind::AliveFiltered;
    const NOT_ALIVE_DISPOSED: Self::ChangeKind = ChangeKind::NotAliveDisposed;
    const NOT_ALIVE_UNREGISTERED: Self::ChangeKind = ChangeKind::NotAliveUnregistered;

    type ReliabilityKind = ReliabilityKind;
    const BEST_EFFORT: Self::ReliabilityKind = 1;
    const RELIABLE: Self::ReliabilityKind = 2;

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
}

impl rust_rtps_pim::behavior::Types for RtpsUdpPsm {
    type Duration = Duration;

    type ChangeForReaderStatusKind = ChangeForReaderStatusKind;
    const UNSENT: Self::ChangeForReaderStatusKind = ChangeForReaderStatusKind::Unsent;
    const UNACKNOWLEDGED: Self::ChangeForReaderStatusKind =
        ChangeForReaderStatusKind::Unacknowledged;
    const REQUESTED: Self::ChangeForReaderStatusKind = ChangeForReaderStatusKind::Requested;
    const ACKNOWLEDGED: Self::ChangeForReaderStatusKind = ChangeForReaderStatusKind::Acknowledged;
    const UNDERWAY: Self::ChangeForReaderStatusKind = ChangeForReaderStatusKind::Underway;

    type ChangeFromWriterStatusKind = ChangeFromWriterStatusKind;
    const LOST: Self::ChangeFromWriterStatusKind = ChangeFromWriterStatusKind::Lost;
    const MISSING: Self::ChangeFromWriterStatusKind = ChangeFromWriterStatusKind::Missing;
    const RECEIVED: Self::ChangeFromWriterStatusKind = ChangeFromWriterStatusKind::Received;
    const UNKNOWN: Self::ChangeFromWriterStatusKind = ChangeFromWriterStatusKind::Unknown;

    type ParticipantMessageData = ();
}
