use serde::ser::SerializeStruct;

use rust_rtps_pim::{behavior::types::{DurationPIM, ParticipantMessageDataPIM}, messages::{submessages::{DataSubmessage, DataSubmessagePIM}, types::{
        CountPIM, FragmentNumberPIM, GroupDigestPIM, ParameterIdPIM, ProtocolIdPIM,
        SubmessageFlagPIM, SubmessageKindPIM, TimePIM,
    }}, structure::types::{
        DataPIM, EntityIdPIM, GuidPrefixPIM, InstanceHandlePIM, LocatorPIM, ParameterListPIM,
        ProtocolVersionPIM, SequenceNumberPIM, VendorIdPIM, GUIDPIM,
    }};

pub mod submessages;

pub struct RtpsUdpPsm;

impl GuidPrefixPIM for RtpsUdpPsm {
    type GuidPrefixType = GuidPrefix;
    const GUIDPREFIX_UNKNOWN: Self::GuidPrefixType = GuidPrefix([0; 12]);
}

impl EntityIdPIM for RtpsUdpPsm {
    type EntityIdType = EntityId;
    const ENTITYID_UNKNOWN: Self::EntityIdType = EntityId {
        entity_key: [0; 3],
        entity_kind: 0,
    };

    const ENTITYID_PARTICIPANT: Self::EntityIdType = EntityId {
        entity_key: [0, 0, 0x01],
        entity_kind: 0xc1,
    };
}

impl GUIDPIM<RtpsUdpPsm> for RtpsUdpPsm {
    type GUIDType = GUID;
    const GUID_UNKNOWN: Self::GUIDType = GUID {
        prefix: RtpsUdpPsm::GUIDPREFIX_UNKNOWN,
        entity_id: RtpsUdpPsm::ENTITYID_UNKNOWN,
    };
}

#[derive(Clone, Copy, PartialEq)]
pub struct GUID {
    pub prefix: GuidPrefix,
    pub entity_id: EntityId,
}

impl rust_rtps_pim::structure::types::GUID<RtpsUdpPsm> for GUID {
    fn new(prefix: GuidPrefix, entity_id: EntityId) -> Self {
        Self { prefix, entity_id }
    }

    fn prefix(&self) -> &GuidPrefix {
        todo!()
    }

    fn entity_id(&self) -> &EntityId {
        todo!()
    }
}

impl SequenceNumberPIM for RtpsUdpPsm {
    type SequenceNumberType = SequenceNumber;
    const SEQUENCE_NUMBER_UNKNOWN: Self::SequenceNumberType = SequenceNumber {
        high: core::i32::MIN,
        low: core::u32::MAX,
    };
}

impl LocatorPIM for RtpsUdpPsm {
    type LocatorType = Locator;
}

impl InstanceHandlePIM for RtpsUdpPsm {
    type InstanceHandleType = InstanceHandle;
}

impl ProtocolVersionPIM for RtpsUdpPsm {
    type ProtocolVersionType = ProtocolVersion;
    const PROTOCOLVERSION: Self::ProtocolVersionType = Self::PROTOCOLVERSION_2_4;
    const PROTOCOLVERSION_1_0: Self::ProtocolVersionType = ProtocolVersion { major: 1, minor: 0 };
    const PROTOCOLVERSION_1_1: Self::ProtocolVersionType = ProtocolVersion { major: 1, minor: 1 };
    const PROTOCOLVERSION_2_0: Self::ProtocolVersionType = ProtocolVersion { major: 2, minor: 0 };
    const PROTOCOLVERSION_2_1: Self::ProtocolVersionType = ProtocolVersion { major: 2, minor: 1 };
    const PROTOCOLVERSION_2_2: Self::ProtocolVersionType = ProtocolVersion { major: 2, minor: 2 };
    const PROTOCOLVERSION_2_3: Self::ProtocolVersionType = ProtocolVersion { major: 2, minor: 3 };
    const PROTOCOLVERSION_2_4: Self::ProtocolVersionType = ProtocolVersion { major: 2, minor: 4 };
}

impl VendorIdPIM for RtpsUdpPsm {
    type VendorIdType = VendorId;
    const VENDOR_ID_UNKNOWN: Self::VendorIdType = VendorId([0; 2]);
}

impl DataPIM for RtpsUdpPsm {
    type DataType = Data;
}

impl ProtocolIdPIM for RtpsUdpPsm {
    type ProtocolIdType = ProtocolId;
    const PROTOCOL_RTPS: Self::ProtocolIdType = [b'R', b'T', b'P', b'S'];
}

impl ParameterListPIM<RtpsUdpPsm> for RtpsUdpPsm {
    type ParameterListType = ParameterList;
}

impl SubmessageFlagPIM for RtpsUdpPsm {
    type SubmessageFlagType = SubmessageFlag;
}

type SubmessageKind = u8;

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

impl ParameterIdPIM for RtpsUdpPsm {
    type ParameterIdType = ParameterId;
}

impl FragmentNumberPIM for RtpsUdpPsm {
    type FragmentNumberType = FragmentNumber;
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

impl<'a> DataSubmessagePIM<'a, Self> for RtpsUdpPsm {
    type DataSubmessageType = submessages::data::DataSubmesage<'a>;
}

#[derive(Clone, Copy, PartialEq, Debug, serde::Serialize)]
pub struct Octet(u8);

impl Octet {
    pub fn is_bit_set(&self, index: usize) -> bool {
        self.0 & (0b_0000_0001 << index) != 0
    }
}

impl<const N: usize> From<[SubmessageFlag; N]> for Octet {
    fn from(value: [SubmessageFlag; N]) -> Self {
        let mut flags = 0b_0000_0000;
        for (i, &item) in value.iter().enumerate() {
            if item {
                flags |= 0b_0000_0001 << i
            }
        }
        Self(flags)
    }
}
impl<const N: usize> From<Octet> for [SubmessageFlag; N] {
    fn from(_value: Octet) -> Self {
        todo!()
    }
}
impl From<Octet> for u8 {
    fn from(value: Octet) -> Self {
        value.0
    }
}
impl From<u8> for Octet {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UShort(u16);

impl rust_rtps_pim::messages::submessage_elements::UShort for UShort {
    fn new(value: u16) -> Self {
        Self(value)
    }

    fn value(&self) -> &u16 {
        &self.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub struct Long(i32);

impl rust_rtps_pim::messages::submessage_elements::Long for Long {
    fn new(value: i32) -> Self {
        Self(value)
    }

    fn value(&self) -> &i32 {
        &self.0
    }
}

impl From<[u8; 4]> for Long {
    fn from(value: [u8; 4]) -> Self {
        Self(i32::from_le_bytes(value))
    }
}

impl Into<[u8; 4]> for Long {
    fn into(self) -> [u8; 4] {
        self.0.to_le_bytes()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub struct ULong(u32);

impl rust_rtps_pim::messages::submessage_elements::ULong for ULong {
    fn new(value: u32) -> Self {
        Self(value)
    }

    fn value(&self) -> &u32 {
        &self.0
    }
}

impl From<[u8; 4]> for ULong {
    fn from(value: [u8; 4]) -> Self {
        Self(u32::from_le_bytes(value))
    }
}

impl Into<[u8; 4]> for ULong {
    fn into(self) -> [u8; 4] {
        self.0.to_le_bytes()
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct GuidPrefix(pub [u8; 12]);

impl From<[u8; 12]> for GuidPrefix {
    fn from(value: [u8; 12]) -> Self {
        Self(value)
    }
}

impl Into<[u8; 12]> for GuidPrefix {
    fn into(self) -> [u8; 12] {
        self.0
    }
}

impl rust_rtps_pim::messages::submessage_elements::GuidPrefix<RtpsUdpPsm> for GuidPrefix {
    fn new(value: GuidPrefix) -> Self {
        value
    }

    fn value(&self) -> &GuidPrefix {
        self
    }
}

#[derive(Clone, Copy, PartialEq, serde::Serialize)]
pub struct EntityId {
    pub entity_key: [u8; 3],
    pub entity_kind: u8,
}

impl Into<[u8; 4]> for EntityId {
    fn into(self) -> [u8; 4] {
        [
            self.entity_key[0],
            self.entity_key[1],
            self.entity_key[2],
            self.entity_kind,
        ]
    }
}

impl From<[u8; 4]> for EntityId {
    fn from(value: [u8; 4]) -> Self {
        Self {
            entity_key: [value[0], value[1], value[2]],
            entity_kind: value[3],
        }
    }
}

impl rust_rtps_pim::messages::submessage_elements::EntityId<RtpsUdpPsm> for EntityId {
    fn new(value: EntityId) -> Self {
        value
    }

    fn value(&self) -> &EntityId {
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct SequenceNumber {
    pub high: i32,
    pub low: u32,
}
impl PartialOrd for SequenceNumber {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Into::<i64>::into(*self).partial_cmp(&(*other).into())
    }
}
impl Ord for SequenceNumber {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        Into::<i64>::into(*self).cmp(&(*other).into())
    }
}

impl From<SequenceNumber> for i64 {
    fn from(value: SequenceNumber) -> Self {
        ((value.high as i64) << 32) + value.low as i64
    }
}

impl From<i64> for SequenceNumber {
    fn from(value: i64) -> Self {
        Self {
            high: (value >> 32) as i32,
            low: value as u32,
        }
    }
}

impl rust_rtps_pim::messages::submessage_elements::SequenceNumber<RtpsUdpPsm> for SequenceNumber {
    fn new(value: SequenceNumber) -> Self {
        value
    }

    fn value(&self) -> &SequenceNumber {
        self
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Locator {
    pub kind: Long,
    pub port: ULong,
    pub address: [u8; 16],
}

impl rust_rtps_pim::structure::types::Locator for Locator {
    type LocatorKind = Long;
    type LocatorPort = ULong;
    type LocatorAddress = [u8; 16];

    const LOCATOR_KIND_INVALID: Self::LocatorKind = Long(-1);
    const LOCATOR_KIND_RESERVED: Self::LocatorKind = Long(0);
    #[allow(non_upper_case_globals)]
    const LOCATOR_KIND_UDPv4: Self::LocatorKind = Long(1);
    #[allow(non_upper_case_globals)]
    const LOCATOR_KIND_UDPv6: Self::LocatorKind = Long(2);
    const LOCATOR_ADDRESS_INVALID: Self::LocatorAddress = [0; 16];
    const LOCATOR_PORT_INVALID: Self::LocatorPort = ULong(0);

    const LOCATOR_INVALID: Self = Locator {
        kind: Self::LOCATOR_KIND_INVALID,
        port: Self::LOCATOR_PORT_INVALID,
        address: Self::LOCATOR_ADDRESS_INVALID,
    };

    fn kind(&self) -> &Self::LocatorKind {
        &self.kind
    }

    fn port(&self) -> &Self::LocatorPort {
        &self.port
    }

    fn address(&self) -> &Self::LocatorAddress {
        &self.address
    }
}

#[derive(Clone)]
pub struct SequenceNumberSet {
    bitmap_base: SequenceNumber,
    num_bits: ULong,
    bitmap: Vec<i32>,
}
#[test]
fn integer_division() {
    let m = (0_i32 + 31) / 32;
    assert_eq!(0_i32, m);
    assert_eq!(0, vec![0; m as usize].len());
}
impl SequenceNumberSet {
    pub fn new(bitmap_base: SequenceNumber, set: Vec<SequenceNumber>) -> Self {
        let base = Into::<i64>::into(bitmap_base) as i32;
        let max = set.iter().max();
        let num_bits = match max {
            Some(max) => Into::<i64>::into(*max) as i32 - base,
            None => 0,
        };
        let number_of_bitmap_elements = ((num_bits + 31) / 32) as usize; // aka "M"
        let mut bitmap = vec![0; number_of_bitmap_elements];
        for sequence_number in set.iter() {
            let delta_n = Into::<i64>::into(*sequence_number) as i32 - base;
            let bitmap_num = delta_n / 32;
            let bit_position = delta_n - bitmap_num * 32;
            bitmap[bitmap_num as usize] |= 1 << bit_position;
        }
        Self {
            bitmap_base,
            num_bits: ULong(num_bits as u32),
            bitmap,
        }
    }

    pub fn len(&self) -> u16 {
        12 + 4 * self.bitmap.len() as u16
    }
}

impl serde::Serialize for SequenceNumberSet {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let len = 2 + self.bitmap.len();
        let mut state = serializer.serialize_struct("SequenceNumberSet", len)?;
        state.serialize_field("bitmapBase", &self.bitmap_base)?;
        state.serialize_field("numBits", &self.num_bits)?;
        const BITMAP_NAMES: [&str; 8] = [
            "bitmap[0]",
            "bitmap[1]",
            "bitmap[2]",
            "bitmap[3]",
            "bitmap[4]",
            "bitmap[5]",
            "bitmap[6]",
            "bitmap[7]",
        ];
        for e in self.bitmap.iter().enumerate() {
            state.serialize_field(BITMAP_NAMES[e.0], e.1)?;
        }
        state.end()
    }
}

impl rust_rtps_pim::messages::submessage_elements::SequenceNumberSet<RtpsUdpPsm>
    for SequenceNumberSet
{
    type SequenceNumberVector = ();

    fn new(_base: SequenceNumber, _set: Self::SequenceNumberVector) -> Self {
        todo!()
    }

    fn base(&self) -> &SequenceNumber {
        &self.bitmap_base
    }

    fn set(&self) -> &Self::SequenceNumberVector {
        // &self.bitmap
        todo!()
    }
}

pub type InstanceHandle = i32;

#[derive(Clone, Copy)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
}

impl rust_rtps_pim::messages::submessage_elements::ProtocolVersion<RtpsUdpPsm> for ProtocolVersion {
    fn new(value: ProtocolVersion) -> Self {
        value
    }

    fn value(&self) -> &ProtocolVersion {
        self
    }
}

pub type Data = Vec<u8>;
#[derive(serde::Serialize)]
pub struct SerializedData<'a>(&'a [u8]);

impl<'a> SerializedData<'a> {
    pub fn len(&self) -> u16 {
        self.0.len() as u16
    }
}

impl<'a> rust_rtps_pim::messages::submessage_elements::SerializedData<'a> for SerializedData<'a> {
    fn new(value: &'a [u8]) -> Self {
        Self(value)
    }

    fn value(&self) -> &[u8] {
        self.0
    }
}

impl<'a> rust_rtps_pim::messages::submessage_elements::SerializedDataFragment<'a>
    for SerializedData<'a>
{
    fn new(value: &'a [u8]) -> Self {
        Self(value)
    }

    fn value(&self) -> &[u8] {
        self.0
    }
}

#[derive(Clone, Copy)]
pub struct VendorId([u8; 2]);

impl rust_rtps_pim::messages::submessage_elements::VendorId<RtpsUdpPsm> for VendorId {
    fn new(value: VendorId) -> Self {
        value
    }

    fn value(&self) -> &VendorId {
        self
    }
}

pub type ProtocolId = [u8; 4];
pub type SubmessageFlag = bool;

#[derive(Clone, Copy)]
pub struct Time {
    pub seconds: u32,
    pub fraction: u32,
}

impl rust_rtps_pim::messages::submessage_elements::Timestamp<RtpsUdpPsm> for Time {
    fn new(value: Time) -> Self {
        value
    }

    fn value(&self) -> &Time {
        self
    }
}

#[derive(Clone, Copy, serde::Serialize)]
pub struct Count(i32);

impl rust_rtps_pim::messages::submessage_elements::Count<RtpsUdpPsm> for Count {
    fn new(value: Count) -> Self {
        value
    }

    fn value(&self) -> &Count {
        self
    }
}

pub type ParameterId = i16;
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct FragmentNumber(u32);

impl rust_rtps_pim::messages::submessage_elements::FragmentNumber<RtpsUdpPsm> for FragmentNumber {
    fn new(value: FragmentNumber) -> Self {
        value
    }

    fn value(&self) -> &FragmentNumber {
        self
    }
}

pub struct FragmentNumberSet(Vec<FragmentNumber>);

impl rust_rtps_pim::messages::submessage_elements::FragmentNumberSet<RtpsUdpPsm>
    for FragmentNumberSet
{
    type FragmentNumberVector = Self;

    fn new(_base: FragmentNumber, _set: Self::FragmentNumberVector) -> Self {
        todo!()
    }

    fn base(&self) -> &FragmentNumber {
        &FragmentNumber(0)
    }

    fn set(&self) -> &Self::FragmentNumberVector {
        self
    }
}

pub type GroupDigest = [u8; 4];

#[derive(Clone, Copy)]
pub struct Duration {
    pub seconds: i32,
    pub fraction: u32,
}

#[derive(Clone)]
pub struct Parameter {
    pub parameter_id: ParameterId,
    pub length: i16,
    pub value: Vec<u8>,
}

impl Parameter {
    pub fn len(&self) -> u16 {
        4 + self.value.len() as u16
    }
}

impl rust_rtps_pim::messages::submessage_elements::Parameter<RtpsUdpPsm> for Parameter {
    fn parameter_id(&self) -> ParameterId {
        self.parameter_id
    }

    fn length(&self) -> i16 {
        self.length
    }

    fn value(&self) -> &[u8] {
        &self.value
    }
}
impl serde::Serialize for Parameter {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("ParameterList", 1)?;
        state.serialize_field("parameter_id", &self.parameter_id)?;
        state.serialize_field("length", &self.length)?;
        state.serialize_field("value", &self.value)?;
        state.end()
    }
}

pub struct ParameterList {
    pub parameter: Vec<Parameter>,
}

impl ParameterList {
    pub fn len(&self) -> u16 {
        self.parameter.iter().map(|p| p.len()).sum()
    }
}

impl rust_rtps_pim::messages::submessage_elements::ParameterList<RtpsUdpPsm> for ParameterList {
    type Parameter = Parameter;
    type ParameterList = Vec<Self::Parameter>;

    fn new(parameter: Self::ParameterList) -> Self {
        Self { parameter }
    }

    fn parameter(&self) -> &Self::ParameterList {
        &self.parameter
    }
}
impl serde::Serialize for ParameterList {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("ParameterList", 1)?;
        for parameter in &self.parameter {
            state.serialize_field("parameter", parameter)?;
        }
        state.end()
    }
}

pub struct LocatorList(Vec<Locator>);

impl rust_rtps_pim::messages::submessage_elements::LocatorList<RtpsUdpPsm> for LocatorList {
    type LocatorList = Vec<Locator>;

    fn new(value: Self::LocatorList) -> Self {
        Self(value)
    }

    fn value(&self) -> &Self::LocatorList {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn octet_from_submessage_flags() {
        let result: Octet = [true, false, true].into();
        assert_eq!(result, Octet(0b_0000_0101));
    }

    #[test]
    fn octet_from_submessage_flags_empty() {
        let result: Octet = [].into();
        assert_eq!(result, Octet(0b_0000_0000));
    }
    #[test]
    #[should_panic]
    fn octet_from_submessage_flags_overflow() {
        let _: Octet = [true; 9].into();
    }

    #[test]
    fn octet_is_set_bit() {
        let flags = Octet(0b_0000_0001);
        assert_eq!(flags.is_bit_set(0), true);

        let flags = Octet(0b_0000_0000);
        assert_eq!(flags.is_bit_set(0), false);

        let flags = Octet(0b_0000_0010);
        assert_eq!(flags.is_bit_set(1), true);

        let flags = Octet(0b_1000_0011);
        assert_eq!(flags.is_bit_set(7), true);
    }
}

// impl EntityId {
//     pub const ENTITY_KIND_USER_DEFINED_UNKNOWN: u8 = 0x00;
//     pub const ENTITY_KIND_USER_DEFINED_WRITER_WITH_KEY: u8 = 0x02;
//     pub const ENTITY_KIND_USER_DEFINED_WRITER_NO_KEY: u8 = 0x03;
//     pub const ENTITY_KIND_USER_DEFINED_READER_WITH_KEY: u8 = 0x04;
//     pub const ENTITY_KIND_USER_DEFINED_READER_NO_KEY: u8 = 0x07;
//     pub const ENTITY_KIND_USER_DEFINED_WRITER_GROUP: u8 = 0x08;
//     pub const ENTITY_KIND_USER_DEFINED_READER_GROUP: u8 = 0x09;
//     pub const ENTITY_KIND_BUILT_IN_UNKNOWN: u8 = 0xc0;
//     pub const ENTITY_KIND_BUILT_IN_PARTICIPANT: u8 = 0xc1;
//     pub const ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY: u8 = 0xc2;
//     pub const ENTITY_KIND_BUILT_IN_WRITER_NO_KEY: u8 = 0xc3;
//     pub const ENTITY_KIND_BUILT_IN_READER_WITH_KEY: u8 = 0xc4;
//     pub const ENTITY_KIND_BUILT_IN_READER_NO_KEY: u8 = 0xc7;
//     pub const ENTITY_KIND_BUILT_IN_WRITER_GROUP: u8 = 0xc8;
//     pub const ENTITY_KIND_BUILT_IN_READER_GROUP: u8 = 0xc9;

//     pub const ENTITYID_PARTICIPANT: EntityId = EntityId {
//         entity_key: [0, 0, 0x01],
//         entity_kind: 0xc1,
//     };

//     pub const ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER: EntityId = EntityId {
//         entity_key: [0, 0, 0x02],
//         entity_kind: 0xc2,
//     };
//     pub const ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR: EntityId = EntityId {
//         entity_key: [0, 0, 0x02],
//         entity_kind: 0xc7,
//     };

//     pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER: EntityId = EntityId {
//         entity_key: [0, 0, 0x03],
//         entity_kind: 0xc2,
//     };
//     pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR: EntityId = EntityId {
//         entity_key: [0, 0, 0x03],
//         entity_kind: 0xc7,
//     };

//     pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER: EntityId = EntityId {
//         entity_key: [0, 0, 0x04],
//         entity_kind: 0xc2,
//     };
//     pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR: EntityId = EntityId {
//         entity_key: [0, 0, 0x04],
//         entity_kind: 0xc7,
//     };

//     pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER: EntityId = EntityId {
//         entity_key: [0, 0x01, 0x00],
//         entity_kind: 0xc2,
//     };

//     pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR: EntityId = EntityId {
//         entity_key: [0, 0x01, 0x00],
//         entity_kind: 0xc7,
//     };

//     pub const ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER: EntityId = EntityId {
//         entity_key: [0, 0x02, 0x00],
//         entity_kind: 0xc2,
//     };
//     pub const ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER: EntityId = EntityId {
//         entity_key: [0, 0x02, 0x00],
//         entity_kind: 0xc7,
//     };
// }

// impl rust_rtps_pim::types::EntityId for EntityId {
//     const ENTITYID_UNKNOWN: Self = Self {
//         entity_key: [0; 3],
//         entity_kind: 0,
//     };
// }
