use crate::{
    behavior,
    messages::{self, submessage_elements::Parameter},
    structure::{self, types::Locator},
    RtpsPsm,
};
use std::vec::Vec;

pub struct MockPsm;

impl RtpsPsm for MockPsm {
    type SequenceNumberSet = Vec<<Self as structure::Types>::SequenceNumber>;

    type LocatorList = Vec<<Self as structure::Types>::Locator>;

    type FragmentNumberSet = Vec<<Self as messages::Types>::FragmentNumber>;

    type Parameter = MockParameter;

    type ParameterList = Vec<Self::Parameter>;
}

impl structure::Types for MockPsm {
    type Guid = [u8; 16];
    const GUID_UNKNOWN: Self::Guid = [0; 16];

    type GuidPrefix = [u8; 12];
    const GUIDPREFIX_UNKNOWN: Self::GuidPrefix = [0; 12];

    type EntityId = [u8; 4];
    const ENTITYID_UNKNOWN: Self::EntityId = [0; 4];

    type SequenceNumber = i64;
    const SEQUENCE_NUMBER_UNKNOWN: Self::SequenceNumber = i64::MIN;

    type Locator = MockLocator;

    const LOCATOR_INVALID: Self::Locator = MockLocator;

    type TopicKind = u8;
    const NO_KEY: Self::TopicKind = 0;
    const WITH_KEY: Self::TopicKind = 1;

    type ChangeKind = u8;

    const ALIVE: Self::ChangeKind = 0;
    const ALIVE_FILTERED: Self::ChangeKind = 1;
    const NOT_ALIVE_DISPOSED: Self::ChangeKind = 2;
    const NOT_ALIVE_UNREGISTERED: Self::ChangeKind = 3;

    type ReliabilityKind = u8;
    const BEST_EFFORT: Self::ReliabilityKind = 0;
    const RELIABLE: Self::ReliabilityKind = 1;

    type InstanceHandle = u32;

    type ProtocolVersion = [u8; 2];

    const PROTOCOLVERSION: Self::ProtocolVersion = [2, 4];
    const PROTOCOLVERSION_1_0: Self::ProtocolVersion = [1, 0];
    const PROTOCOLVERSION_1_1: Self::ProtocolVersion = [1, 1];
    const PROTOCOLVERSION_2_0: Self::ProtocolVersion = [2, 0];
    const PROTOCOLVERSION_2_1: Self::ProtocolVersion = [2, 1];
    const PROTOCOLVERSION_2_2: Self::ProtocolVersion = [2, 2];
    const PROTOCOLVERSION_2_3: Self::ProtocolVersion = [2, 3];
    const PROTOCOLVERSION_2_4: Self::ProtocolVersion = [2, 4];

    type VendorId = i8;
    const VENDOR_ID_UNKNOWN: Self::VendorId = -1;
}

impl messages::Types for MockPsm {
    type ProtocolId = [u8; 4];

    const PROTOCOL_RTPS: Self::ProtocolId = [b'R', b'T', b'P', b'S'];

    type SubmessageFlag = bool;

    type SubmessageKind = u8;
    const DATA: Self::SubmessageKind = 0;
    const GAP: Self::SubmessageKind = 1;
    const HEARTBEAT: Self::SubmessageKind = 2;
    const ACKNACK: Self::SubmessageKind = 3;
    const PAD: Self::SubmessageKind = 4;
    const INFO_TS: Self::SubmessageKind = 5;
    const INFO_REPLY: Self::SubmessageKind = 6;
    const INFO_DST: Self::SubmessageKind = 7;
    const INFO_SRC: Self::SubmessageKind = 8;
    const DATA_FRAG: Self::SubmessageKind = 9;
    const NACK_FRAG: Self::SubmessageKind = 10;
    const HEARTBEAT_FRAG: Self::SubmessageKind = 11;

    type Time = u64;
    const TIME_ZERO: Self::Time = 0;
    const TIME_INVALID: Self::Time = u64::MIN;
    const TIME_INFINITE: Self::Time = u64::MAX;

    type Count = u32;
    type ParameterId = u8;
    type FragmentNumber = i32;
    type GroupDigest = i32;
}

impl behavior::Types for MockPsm {
    type Duration = i64;

    type ChangeForReaderStatusKind = u8;
    const UNSENT: Self::ChangeForReaderStatusKind = 0;
    const UNACKNOWLEDGED: Self::ChangeForReaderStatusKind = 1;
    const REQUESTED: Self::ChangeForReaderStatusKind = 2;
    const ACKNOWLEDGED: Self::ChangeForReaderStatusKind = 3;
    const UNDERWAY: Self::ChangeForReaderStatusKind = 4;

    type ChangeFromWriterStatusKind = u8;
    const LOST: Self::ChangeFromWriterStatusKind = 0;
    const MISSING: Self::ChangeFromWriterStatusKind = 1;
    const RECEIVED: Self::ChangeFromWriterStatusKind = 2;
    const UNKNOWN: Self::ChangeFromWriterStatusKind = 3;

    type ParticipantMessageData = u8;
}

pub struct MockLocator;

impl Locator for MockLocator {
    type Kind = i32;
    type Port = u32;
    type Address = [u8; 16];

    const LOCATOR_KIND_INVALID: Self::Kind = -1;
    const LOCATOR_KIND_RESERVED: Self::Kind = 0;
    #[allow(non_upper_case_globals)]
    const LOCATOR_KIND_UDPv4: Self::Kind = 1;
    #[allow(non_upper_case_globals)]
    const LOCATOR_KIND_UDPv6: Self::Kind = 2;
    const LOCATOR_ADDRESS_INVALID: Self::Address = [0; 16];
    const LOCATOR_PORT_INVALID: Self::Port = 0;
}

pub struct MockParameter;

impl Parameter for MockParameter {
    type PSM = MockPsm;

    fn parameter_id(&self) -> <Self::PSM as messages::Types>::ParameterId {
        todo!()
    }

    fn length(&self) -> i16 {
        todo!()
    }

    fn value(&self) -> &[u8] {
        todo!()
    }
}
