use serde::ser::SerializeStruct;

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
        submessages::{DataSubmessagePIM, GapSubmessagePIM},
        types::{
            CountPIM, FragmentNumberPIM, GroupDigestPIM, ParameterIdPIM, ProtocolIdPIM,
            SubmessageFlag, SubmessageKindPIM, TimePIM,
        },
        RTPSMessagePIM, RtpsMessageHeaderPIM, RtpsSubmessageHeaderPIM,
    },
    structure::types::{
        DataPIM, EntityIdPIM, GuidPrefixPIM, InstanceHandlePIM, LocatorPIM, ProtocolVersionPIM,
        SequenceNumberPIM, VendorIdPIM, GUIDPIM,
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

impl GUIDPIM<Self> for RtpsUdpPsm {
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

impl rust_rtps_pim::structure::types::GUIDType<RtpsUdpPsm> for GUID {
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

impl From<[u8; 16]> for GUID {
    fn from(_: [u8; 16]) -> Self {
        todo!()
    }
}

impl Into<[u8; 16]> for GUID {
    fn into(self) -> [u8; 16] {
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

    const LOCATOR_INVALID: Self::LocatorType = Locator {
        kind: Self::LOCATOR_KIND_INVALID,
        port: Self::LOCATOR_PORT_INVALID,
        address: Self::LOCATOR_ADDRESS_INVALID,
    };
    const LOCATOR_KIND_INVALID:
        <Self::LocatorType as rust_rtps_pim::structure::types::LocatorType>::LocatorKind = Long(-1);
    const LOCATOR_KIND_RESERVED:
        <Self::LocatorType as rust_rtps_pim::structure::types::LocatorType>::LocatorKind = Long(0);
    #[allow(non_upper_case_globals)]
    const LOCATOR_KIND_UDPv4:
        <Self::LocatorType as rust_rtps_pim::structure::types::LocatorType>::LocatorKind = Long(1);
    #[allow(non_upper_case_globals)]
    const LOCATOR_KIND_UDPv6:
        <Self::LocatorType as rust_rtps_pim::structure::types::LocatorType>::LocatorKind = Long(2);
    const LOCATOR_ADDRESS_INVALID:
        <Self::LocatorType as rust_rtps_pim::structure::types::LocatorType>::LocatorAddress =
        [0; 16];
    const LOCATOR_PORT_INVALID:
        <Self::LocatorType as rust_rtps_pim::structure::types::LocatorType>::LocatorPort = ULong(0);
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

impl ParameterListSubmessageElementPIM<RtpsUdpPsm> for RtpsUdpPsm {
    type ParameterListSubmessageElementType = ParameterList;
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

impl UShortSubmessageElementPIM for RtpsUdpPsm {
    type UShortSubmessageElementType = UShort;
}

impl ULongSubmessageElementPIM for RtpsUdpPsm {
    type ULongSubmessageElementType = ULong;
}

impl LongSubmessageElementPIM for RtpsUdpPsm {
    type LongSubmessageElementType = Long;
}

impl EntityIdSubmessageElementPIM<Self> for RtpsUdpPsm {
    type EntityIdSubmessageElementType = EntityId;
}

impl GuidPrefixSubmessageElementPIM<Self> for RtpsUdpPsm {
    type GuidPrefixSubmessageElementType = GuidPrefix;
}

impl SequenceNumberSubmessageElementPIM<Self> for RtpsUdpPsm {
    type SequenceNumberSubmessageElementType = SequenceNumber;
}

impl SequenceNumberSetSubmessageElementPIM<Self> for RtpsUdpPsm {
    type SequenceNumberSetSubmessageElementType = SequenceNumberSet;
}

impl FragmentNumberSubmessageElementPIM<Self> for RtpsUdpPsm {
    type FragmentNumberSubmessageElementType = FragmentNumber;
}

impl FragmentNumberSetSubmessageElementPIM<Self> for RtpsUdpPsm {
    type FragmentNumberSetSubmessageElementType = FragmentNumberSet;
}

impl VendorIdSubmessageElementPIM<Self> for RtpsUdpPsm {
    type VendorIdSubmessageElementType = VendorId;
}

impl LocatorListSubmessageElementPIM<Self> for RtpsUdpPsm {
    type LocatorListSubmessageElementType = LocatorList;
}

impl ProtocolVersionSubmessageElementPIM<Self> for RtpsUdpPsm {
    type ProtocolVersionSubmessageElementType = ProtocolVersion;
}

impl TimestampSubmessageElementPIM<Self> for RtpsUdpPsm {
    type TimestampSubmessageElementType = Time;
}

impl<'a> SerializedDataSubmessageElementPIM<'a> for RtpsUdpPsm {
    type SerializedDataSubmessageElementType = SerializedData<'a>;
}

impl<'a> SerializedDataFragmentSubmessageElementPIM<'a> for RtpsUdpPsm {
    type SerializedDataFragmentSubmessageElementType = SerializedData<'a>;
}

impl CountSubmessageElementPIM<Self> for RtpsUdpPsm {
    type CountSubmessageElementType = Count;
}

impl<'a> RTPSMessagePIM<'a, Self> for RtpsUdpPsm {
    type RTPSMessageType = RTPSMessageC<'a>;
}

impl<'a> RtpsMessageHeaderPIM<'a, Self> for RtpsUdpPsm {
    type RtpsMessageHeaderType = RTPSMessageHeader;
}

impl RtpsSubmessageHeaderPIM<Self> for RtpsUdpPsm {
    type RtpsSubmessageHeaderType = submessages::SubmessageHeader;
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

impl rust_rtps_pim::messages::submessage_elements::UShortSubmessageElementType for UShort {
    fn new(value: u16) -> Self {
        Self(value)
    }

    fn value(&self) -> &u16 {
        &self.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub struct Long(i32);

impl rust_rtps_pim::messages::submessage_elements::LongSubmessageElementType for Long {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub struct ULong(u32);

impl rust_rtps_pim::messages::submessage_elements::ULongSubmessageElementType for ULong {
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

impl rust_rtps_pim::messages::submessage_elements::GuidPrefixSubmessageElementType<RtpsUdpPsm>
    for GuidPrefix
{
    fn new(value: &GuidPrefix) -> Self {
        value.clone()
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

impl rust_rtps_pim::messages::submessage_elements::EntityIdSubmessageElementType<RtpsUdpPsm>
    for EntityId
{
    fn new(value: &EntityId) -> Self {
        value.clone()
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

impl rust_rtps_pim::messages::submessage_elements::SequenceNumberSubmessageElementType<RtpsUdpPsm>
    for SequenceNumber
{
    fn new(value: &SequenceNumber) -> Self {
        value.clone()
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

impl rust_rtps_pim::structure::types::LocatorType for Locator {
    type LocatorKind = Long;
    type LocatorPort = ULong;
    type LocatorAddress = [u8; 16];

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

#[derive(Debug, PartialEq, Clone)]
pub struct SequenceNumberSet {
    base: SequenceNumber,
    set: Vec<SequenceNumber>,
    num_bits: ULong,
    bitmap: Vec<i32>,
}

impl SequenceNumberSet {
    pub fn len(&self) -> u16 {
        12 /*bitmapBase + numBits */ + 4 * self.bitmap.len() /* bitmap[0] .. bitmap[M-1] */ as u16
    }
}

impl serde::Serialize for SequenceNumberSet {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let len = 2 + self.bitmap.len();

        let mut state = serializer.serialize_struct("SequenceNumberSet", len)?;
        state.serialize_field("bitmapBase", &self.base)?;
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

impl<'de> serde::Deserialize<'de> for SequenceNumberSet {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!()
    }
}

impl
    rust_rtps_pim::messages::submessage_elements::SequenceNumberSetSubmessageElementType<RtpsUdpPsm>
    for SequenceNumberSet
{
    fn new(base: &SequenceNumber, set: &[SequenceNumber]) -> Self {
        let max = set.iter().max();
        let num_bits = match max {
            Some(max) => Into::<i64>::into(*max) - Into::<i64>::into(*base),
            None => 0,
        };
        let number_of_bitmap_elements = ((num_bits + 31) / 32) as usize; // aka "M"
        let mut bitmap = vec![0; number_of_bitmap_elements];
        for sequence_number in set.iter() {
            let delta_n = Into::<i64>::into(*sequence_number) - Into::<i64>::into(*base);
            let bitmap_num = delta_n / 32;
            let bit_position = delta_n - bitmap_num * 32;
            bitmap[bitmap_num as usize] |= 1 << bit_position;
        }
        Self {
            base: base.clone(),
            set: set.into_iter().map(|x| x.clone()).collect(),
            num_bits: ULong(num_bits as u32),
            bitmap,
        }
    }

    fn base(&self) -> &SequenceNumber {
        &self.base
    }

    fn set(&self) -> &[SequenceNumber] {
        &self.set
    }
}

pub type InstanceHandle = i32;

#[derive(Clone, Copy)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
}

impl rust_rtps_pim::messages::submessage_elements::ProtocolVersionSubmessageElementType<RtpsUdpPsm>
    for ProtocolVersion
{
    fn new(value: &ProtocolVersion) -> Self {
        value.clone()
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

impl<'a> rust_rtps_pim::messages::submessage_elements::SerializedDataSubmessageElementType<'a>
    for SerializedData<'a>
{
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

impl<'a>
    rust_rtps_pim::messages::submessage_elements::SerializedDataFragmentSubmessageElementType<'a>
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

impl rust_rtps_pim::messages::submessage_elements::VendorIdSubmessageElementType<RtpsUdpPsm>
    for VendorId
{
    fn new(value: &VendorId) -> Self {
        value.clone()
    }

    fn value(&self) -> &VendorId {
        self
    }
}

pub type ProtocolId = [u8; 4];

#[derive(Clone, Copy)]
pub struct Time {
    pub seconds: u32,
    pub fraction: u32,
}

impl rust_rtps_pim::messages::submessage_elements::TimestampSubmessageElementType<RtpsUdpPsm>
    for Time
{
    fn new(value: &Time) -> Self {
        value.clone()
    }

    fn value(&self) -> &Time {
        self
    }
}

#[derive(Clone, Copy, serde::Serialize)]
pub struct Count(i32);

impl rust_rtps_pim::messages::submessage_elements::CountSubmessageElementType<RtpsUdpPsm>
    for Count
{
    fn new(value: &Count) -> Self {
        value.clone()
    }

    fn value(&self) -> &Count {
        self
    }
}

pub type ParameterId = i16;
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct FragmentNumber(u32);

impl rust_rtps_pim::messages::submessage_elements::FragmentNumberSubmessageElementType<RtpsUdpPsm>
    for FragmentNumber
{
    fn new(value: &FragmentNumber) -> Self {
        value.clone()
    }

    fn value(&self) -> &FragmentNumber {
        self
    }
}

impl From<u32> for FragmentNumber {
    fn from(_: u32) -> Self {
        todo!()
    }
}

impl Into<u32> for FragmentNumber {
    fn into(self) -> u32 {
        todo!()
    }
}

pub struct FragmentNumberSet(Vec<FragmentNumber>);

impl
    rust_rtps_pim::messages::submessage_elements::FragmentNumberSetSubmessageElementType<RtpsUdpPsm>
    for FragmentNumberSet
{
    fn new(_base: &FragmentNumber, _set: &[FragmentNumber]) -> Self {
        todo!()
    }

    fn base(&self) -> &FragmentNumber {
        &FragmentNumber(0)
    }

    fn set(&self) -> &[FragmentNumber] {
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

#[derive(Debug, PartialEq, Clone)]
pub struct Vector(Vec<u8>);
impl serde::Serialize for Vector {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.0.as_slice())
    }
}

impl From<Vec<u8>> for Vector {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Parameter {
    pub parameter_id: ParameterId,
    pub length: i16,
    pub value: Vector,
}

impl Parameter {
    pub fn new(parameter_id: ParameterId, value: Vector) -> Self {
        Self {
            parameter_id,
            length: value.0.len() as i16,
            value,
        }
    }

    pub fn len(&self) -> u16 {
        4 + self.value.0.len() as u16
    }
}

impl rust_rtps_pim::messages::submessage_elements::ParameterType<RtpsUdpPsm> for Parameter {
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

impl serde::Serialize for Parameter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Parameter", 3)?;
        state.serialize_field("ParameterId", &self.parameter_id)?;
        state.serialize_field("length", &self.length)?;
        state.serialize_field("value", &self.value)?;
        state.end()
    }
}

struct ParameterVisitor;

impl<'de> serde::de::Visitor<'de> for ParameterVisitor {
    type Value = Parameter;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Parameter of the ParameterList Submessage Element")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let paramter_id: ParameterId = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
        let data_length: u16 = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
        let mut data = vec![];
        for _ in 0..data_length {
            data.push(
                seq.next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(3, &self))?,
            );
        }
        Ok(Parameter::new(paramter_id, data.into()))
    }
}

impl<'de> serde::Deserialize<'de> for Parameter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const MAX_BYTES: usize = 2 ^ 16;
        deserializer.deserialize_tuple(MAX_BYTES, ParameterVisitor {})
    }
}
const PID_SENTINEL: ParameterId = 1;
static SENTINEL: Parameter = Parameter {
    parameter_id: PID_SENTINEL,
    length: 0,
    value: Vector(vec![]),
};

#[derive(Debug, PartialEq, Clone)]
pub struct ParameterList {
    pub parameter: Vec<Parameter>,
}
impl serde::Serialize for ParameterList {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let len = self.parameter.len();
        let mut state = serializer.serialize_struct("ParameterList", len)?;
        for parameter in &self.parameter {
            state.serialize_field("parameter", &parameter)?;
        }
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
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut parameters = vec![];
        for _ in 0..seq.size_hint().unwrap() {
            let parameter: Parameter = seq
                .next_element()?
                .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
            if parameter == SENTINEL {
                return Ok(ParameterList {
                    parameter: parameters.into(),
                });
            } else {
                parameters.push(parameter);
            }
        }
        todo!()
    }
}

impl<'de, 'a> serde::Deserialize<'de> for ParameterList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const MAX_PARAMETERS: usize = 2 ^ 16;
        deserializer.deserialize_tuple(MAX_PARAMETERS, ParameterListVisitor {})
    }
}

impl ParameterList {
    pub fn len(&self) -> u16 {
        self.parameter.iter().map(|p| p.len()).sum()
    }
}

impl rust_rtps_pim::messages::submessage_elements::ParameterListSubmessageElementType<RtpsUdpPsm>
    for ParameterList
{
    type Parameter = Parameter;

    fn new(_parameter: &[Self::Parameter]) -> Self {
        //let vec: Vec<Parameter> = parameter.iter().map(|x| x.clone()).collect();
        todo!()
    }

    fn parameter(&self) -> &[Self::Parameter] {
        &self.parameter
    }
}

pub struct LocatorList(Vec<Locator>);

impl rust_rtps_pim::messages::submessage_elements::LocatorListSubmessageElementType<RtpsUdpPsm>
    for LocatorList
{
    fn new(_value: &[Locator]) -> Self {
        // Self(value)
        todo!()
    }

    fn value(&self) -> &[Locator] {
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

impl<'a> rust_rtps_pim::messages::RtpsMessageHeaderType<'a, RtpsUdpPsm> for RTPSMessageHeader {
    fn protocol(&self) -> &ProtocolId {
        &self.protocol
    }

    fn version(&self) -> &ProtocolVersion {
        &self.version
    }

    fn vendor_id(&self) -> &VendorId {
        &self.vendor_id
    }

    fn guid_prefix(&self) -> &GuidPrefix {
        &self.guid_prefix
    }
}

pub struct RTPSMessageC<'a> {
    header: RTPSMessageHeader,
    submessages: &'a [&'a dyn rust_rtps_pim::messages::Submessage<RtpsUdpPsm>],
}

impl<'a> rust_rtps_pim::messages::RTPSMessageConstructor<'a, RtpsUdpPsm> for RTPSMessageC<'_> {
    fn new(
        protocol: ProtocolId,
        version: ProtocolVersion,
        vendor_id: VendorId,
        guid_prefix: GuidPrefix,
        submessages: &'a [&'a dyn rust_rtps_pim::messages::Submessage<RtpsUdpPsm>],
    ) -> RTPSMessageC<'a> {
        RTPSMessageC {
            header: RTPSMessageHeader {
                protocol: protocol.clone(),
                version: version.clone(),
                vendor_id: vendor_id.clone(),
                guid_prefix: guid_prefix.clone(),
            },
            submessages,
        }
    }
}

impl<'a> rust_rtps_pim::messages::RTPSMessage<'a, RtpsUdpPsm> for RTPSMessageC<'a> {
    fn header(&self) -> RTPSMessageHeader {
        self.header
    }

    fn submessages(&self) -> &[&dyn rust_rtps_pim::messages::Submessage<RtpsUdpPsm>] {
        &self.submessages
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_serde_cdr::{
        deserializer::RtpsMessageDeserializer, serializer::RtpsMessageSerializer,
    };

    fn serialize<T: serde::Serialize>(value: T) -> Vec<u8> {
        let mut serializer = RtpsMessageSerializer {
            writer: Vec::<u8>::new(),
        };
        value.serialize(&mut serializer).unwrap();
        serializer.writer
    }

    fn deserialize<'de, T: serde::Deserialize<'de>>(buffer: &'de [u8]) -> T {
        let mut de = RtpsMessageDeserializer { reader: buffer };
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
        let parameter = ParameterList {
            parameter: vec![
                Parameter::new(2, vec![51, 61, 71, 81].into()),
                Parameter::new(3, vec![52, 62, 72, 82].into()),
            ]
            .into(),
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
        let expected = ParameterList {
            parameter: vec![
                Parameter::new(0x02, vec![15, 16, 17, 18].into()),
                Parameter::new(0x03, vec![25, 26, 27, 28].into()),
            ]
            .into(),
        };
        #[rustfmt::skip]
        let result: ParameterList = deserialize(&[
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

    #[test]
    fn serialize_sequence_number_set() {
        use rust_rtps_pim::messages::submessage_elements::SequenceNumberSetSubmessageElementType;

        let sequence_number_set = SequenceNumberSet::new(&2.into(), &[4.into(), 6.into()]);
        #[rustfmt::skip]
        assert_eq!(serialize(sequence_number_set), vec![
            0, 0, 0, 0, // SequenceNumberSet: bitmapBase: high
            2, 0, 0, 0, // SequenceNumberSet: bitmapBase: low
            4, 0, 0, 0, // SequenceNumberSet: numBits (ULong)
           0b_001_0100, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // SequenceNumberSet: bitmap[0] long
        ]);
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
