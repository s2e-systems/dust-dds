use serde::ser::SerializeStruct;

use rust_rtps_pim::{
    behavior::types::{DurationPIM, ParticipantMessageDataPIM},
    messages::{
        submessages::{DataSubmessagePIM, GapSubmessagePIM},
        types::{
            CountPIM, FragmentNumberPIM, GroupDigestPIM, ParameterIdPIM, ProtocolIdPIM,
            SubmessageFlagPIM, SubmessageKindPIM, TimePIM,
        },
        RTPSMessagePIM, SubmessageHeaderPIM,
    },
    structure::types::{
        DataPIM, EntityIdPIM, GuidPrefixPIM, InstanceHandlePIM, LocatorPIM, ParameterListPIM,
        ProtocolVersionPIM, SequenceNumberPIM, VendorIdPIM, GUIDPIM,
    },
};

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

impl<'a> RTPSMessagePIM<'a, Self> for RtpsUdpPsm {
    type RTPSMessageType = RTPSMessage<'a>;
}

impl SubmessageHeaderPIM<Self> for RtpsUdpPsm {
    type SubmessageHeaderType = submessages::SubmessageHeader;
}

impl<'a> DataSubmessagePIM<'a, Self> for RtpsUdpPsm {
    type DataSubmessageType = submessages::data::DataSubmesage<'a>;
}

impl GapSubmessagePIM<Self> for RtpsUdpPsm {
    type GapSubmessageType = submessages::gap::GapSubmessage;
}

#[derive(Clone, Copy, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
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

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
    type SequenceNumberVector = Vec<SequenceNumber>;

    fn new(_base: SequenceNumber, _set: Self::SequenceNumberVector) -> Self {
        todo!()
    }

    fn base(&self) -> &SequenceNumber {
        &self.bitmap_base
    }

    fn set(&self) -> Self::SequenceNumberVector {
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




#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct SerializedData<'a>(&'a [u8]);

impl<'a> SerializedData<'a> {
    pub fn len(&self) -> u16 {
        self.0.len() as u16
    }
}

impl<'a> rust_rtps_pim::messages::submessage_elements::SerializedData<'a> for SerializedData<'a> {
    fn new(value: &'a [u8]) -> Self {
        Self(value.into())
    }

    fn value(&self) -> &[u8] {
        self.0
    }
}

impl<'a> serde::Serialize for SerializedData<'a> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.0)
    }
}

impl<'a> rust_rtps_pim::messages::submessage_elements::SerializedDataFragment<'a>
    for SerializedData<'a>
{
    fn new(value: &'a [u8]) -> Self {
        Self(value.into())
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

    fn set(&self) -> Self::FragmentNumberVector {
        todo!()
        // self
    }
}

pub type GroupDigest = [u8; 4];

#[derive(Clone, Copy)]
pub struct Duration {
    pub seconds: i32,
    pub fraction: u32,
}


#[derive(Debug, PartialEq)]
pub struct Vector<T>(Vec<T>);
impl<T: serde::Serialize> serde::Serialize for Vector<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(VectorIter(self.0.iter()))
    }
}
impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for Vector<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
            const MAX_BYTES: usize = 2^16;
            deserializer.deserialize_tuple(MAX_BYTES, VectorVisitor(std::marker::PhantomData))
    }
}
impl<T: serde::Serialize> From<Vec<T>> for Vector<T> {
    fn from(value: Vec<T>) -> Self {
        Self(value)
    }
}
struct VectorVisitor<T>(std::marker::PhantomData<T>);

impl<'de, T: serde::Deserialize<'de>> serde::de::Visitor<'de> for VectorVisitor<T> {
    type Value = Vector<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("2 byte length + length bytes")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where A: serde::de::SeqAccess<'de>,
    {
        let error_message_len = seq.size_hint().unwrap_or(0);
        let data_length: u16 = seq.next_element()?.ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
        let mut data = vec![];
        for _ in 0..data_length {
            data.push(seq.next_element()?.ok_or_else(|| serde::de::Error::invalid_length(error_message_len, &self))?);
        }
        // Ok(Vector())
        todo!()
    }
}
/// Note: size_hint() is not implemented since the default returns (0, None) which
/// in turn istructs the collect_seq of the serializer to put a None to the len argument
/// of the serialize_seq() function. And this in turn should cause the serilization of a
/// sequence without serializing its length as prefix
pub struct VectorIter<'a, T: serde::Serialize>(std::slice::Iter<'a, T>);
impl<'a, T: serde::Serialize> Iterator for VectorIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct Parameter {
    pub parameter_id: ParameterId,
    pub length: i16,
    pub value: Vector<u8>,
}

impl Parameter {
    pub fn new(parameter_id: ParameterId, value: Vector<u8>) -> Self {
        Self {
            parameter_id, length: value.0.len() as i16, value
        }
    }

    pub fn len(&self) -> u16 {
        4 + self.value.0.len() as u16
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
        &self.value.0
    }
}

struct ParameterVisitor;

impl<'de> serde::de::Visitor<'de> for ParameterVisitor {
    type Value = Parameter;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Parameter of the ParameterList Submessage Element")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where A: serde::de::SeqAccess<'de>,
    {
        let paramter_id: ParameterId = seq.next_element()?.ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
        let data_length: u16 = seq.next_element()?.ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
        let mut data = vec![];
        for _ in 0..data_length {
            data.push(seq.next_element()?.ok_or_else(|| serde::de::Error::invalid_length(3, &self))?);
        }
        Ok(Parameter::new(paramter_id, data.into()))
    }
}

impl<'de> serde::Deserialize<'de> for Parameter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
            const MAX_BYTES: usize = 2^16;
            deserializer.deserialize_tuple( MAX_BYTES, ParameterVisitor {})
    }
}
const PID_SENTINEL: ParameterId = 1;
static SENTINEL: Parameter = Parameter{parameter_id: PID_SENTINEL, length: 0, value: Vector(vec![])};

#[derive(Debug, PartialEq)]
pub struct ParameterList {
    pub parameter: Vector<Parameter>,
}
impl serde::Serialize for ParameterList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        let mut state = serializer.serialize_struct("ParameterList", 2)?;
        state.serialize_field("parameter", &self.parameter)?;
        state.serialize_field("sentinel", &SENTINEL)?;
        state.end()
    }
}

struct ParameterListVisitor;

impl<'de> serde::de::Visitor<'de> for ParameterListVisitor {
    type Value = ParameterList;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("ParameterList Submessage Element")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where A: serde::de::SeqAccess<'de>,
    {
        let mut parameters = vec![];
        for _ in 0..seq.size_hint().unwrap() {
            let parameter: Parameter = seq.next_element()?.ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
            if parameter == SENTINEL {
                return Ok(ParameterList{parameter: parameters.into()});
            }
            else {
                parameters.push(parameter);
            }
        }
        todo!()
    }
}

impl<'de, 'a> serde::Deserialize<'de> for ParameterList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        const MAX_PARAMETERS: usize = 2^16;
        deserializer.deserialize_tuple(MAX_PARAMETERS, ParameterListVisitor{})
    }
}
static EMPTY_PARAMETER: ParameterList = ParameterList{parameter: Vector(vec![])};
impl<'de, 'a> serde::Deserialize<'de> for &'a ParameterList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
            Ok(&EMPTY_PARAMETER)
    }
}

impl ParameterList {
    pub fn len(&self) -> u16 {
        self.parameter.0.iter().map(|p| p.len()).sum()
    }
}

impl rust_rtps_pim::messages::submessage_elements::ParameterList<RtpsUdpPsm> for ParameterList {
    type Parameter = Parameter;
    type ParameterList = Vec<Self::Parameter>;

    fn new(parameter: Self::ParameterList) -> Self {
        Self { parameter: parameter.into() }
    }

    fn parameter(&self) -> &Self::ParameterList {
        &self.parameter.0
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

#[derive(Clone, Copy)]
pub struct RTPSMessageHeader {
    protocol: ProtocolId,
    version: ProtocolVersion,
    vendor_id: VendorId,
    guid_prefix: GuidPrefix,
}

impl rust_rtps_pim::messages::Header<RtpsUdpPsm> for RTPSMessageHeader {
    fn protocol(&self) -> ProtocolId {
        self.protocol
    }

    fn version(&self) -> ProtocolVersion {
        self.version
    }

    fn vendor_id(&self) -> VendorId {
        self.vendor_id
    }

    fn guid_prefix(&self) -> GuidPrefix {
        self.guid_prefix
    }
}

pub struct RTPSMessage<'a> {
    header: RTPSMessageHeader,
    submessages: Vec<&'a dyn rust_rtps_pim::messages::Submessage<RtpsUdpPsm>>,
}

impl<'a> rust_rtps_pim::messages::RTPSMessage<'a, RtpsUdpPsm> for RTPSMessage<'a> {
    type RTPSMessageHeaderType = RTPSMessageHeader;
    type RTPSSubmessageVectorType = Vec<&'a dyn rust_rtps_pim::messages::Submessage<RtpsUdpPsm>>;

    fn new<T: IntoIterator<Item = &'a dyn rust_rtps_pim::messages::Submessage<RtpsUdpPsm>>>(
        protocol: ProtocolId,
        version: ProtocolVersion,
        vendor_id: VendorId,
        guid_prefix: GuidPrefix,
        submessages: T,
    ) -> Self {
        Self {
            header: RTPSMessageHeader {
                protocol,
                version,
                vendor_id,
                guid_prefix,
            },
            submessages: submessages.into_iter().collect(),
        }
    }

    fn header(&self) -> Self::RTPSMessageHeaderType {
        self.header
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_serde_cdr::{deserializer::RtpsMessageDeserializer, serializer::RtpsMessageSerializer};

    fn serialize<T: serde::Serialize>(value: T) -> Vec<u8> {
        let mut serializer = RtpsMessageSerializer {writer: Vec::<u8>::new()};
        value.serialize(&mut serializer).unwrap();
        serializer.writer
    }

    fn deserialize<'de, T: serde::Deserialize<'de>>(buffer: &'de [u8]) -> T {
        let mut de = RtpsMessageDeserializer {
            reader: buffer,
        };
        serde::de::Deserialize::deserialize(&mut de).unwrap()
    }

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
    #[test]
    fn serialize_octet() {
        assert_eq!(serialize(Octet(5)), vec![5]);
    }
    #[test]
    fn deserialize_octet() {
        let result: Octet = deserialize(&[5]);
        assert_eq!(result, Octet(5));
    }

    #[test]
    fn serialize_parameter() {
        let parameter = Parameter::new(2, vec![5, 6, 7, 8].into());
        #[rustfmt::skip]
        assert_eq!(serialize(parameter), vec![
            0x02, 0x00, 4, 0, // Parameter | length
            5, 6, 7, 8,       // value
        ]);
    }

    #[test]
    fn serialize_parameter_list() {
        let parameter = ParameterList{
            parameter: vec![
                Parameter::new(2, vec![51, 61, 71, 81].into()),
                Parameter::new(3, vec![52, 62, 72, 82].into())
            ].into()
        };
        #[rustfmt::skip]
        assert_eq!(serialize(parameter), vec![
            0x02, 0x00, 4, 0, // Parameter ID | length
            51, 61, 71, 81,   // value
            0x03, 0x00, 4, 0, // Parameter ID | length
            52, 62, 72, 82,   // value
            0x01, 0x00, 0, 0, // Sentinel: PID_SENTINEL | PID_PAD
        ]);
    }

    #[test]
    fn deserialize_parameter() {
        let expected = Parameter::new(0x02, vec![5, 6, 7, 8].into());
        #[rustfmt::skip]
        let result = deserialize(&[
            0x02, 0x00, 4, 0, // Parameter | length
            5, 6, 7, 8,       // value
        ]);
        assert_eq!(expected, result);
    }

    #[test]
    fn deserialize_parameter_list() {
        let expected = ParameterList{parameter: vec![
            Parameter::new(0x02, vec![15, 16, 17, 18].into()),
            Parameter::new(0x03, vec![25, 26, 27, 28].into())
        ].into()};
        #[rustfmt::skip]
        let result = deserialize(&[
            0x02, 0x00, 4, 0, // Parameter ID | length
            15, 16, 17, 18,        // value
            0x03, 0x00, 4, 0, // Parameter ID | length
            25, 26, 27, 28,        // value
            0x01, 0x00, 0, 0, // Sentinel: Parameter ID | length
            9, 9, 9,    // Following data
        ]);
        assert_eq!(expected, result);
    }

    #[test]
    fn serialize_serialized_data() {
        let data = SerializedData(&[1, 2]);
        assert_eq!(serialize(data), vec![1, 2]);
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
