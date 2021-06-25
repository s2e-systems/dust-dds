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
        submessages::{
            AckNackSubmessagePIM, DataFragSubmessagePIM, DataSubmessagePIM, GapSubmessagePIM,
            HeartbeatFragSubmessagePIM, HeartbeatSubmessagePIM, InfoDestinationSubmessagePIM,
            InfoReplySubmessagePIM, InfoSourceSubmessagePIM, InfoTimestampSubmessagePIM,
            NackFragSubmessagePIM, PadSubmessagePIM, RtpsSubmessageType,
        },
        types::{
            CountPIM, GroupDigestPIM, ProtocolIdPIM, SubmessageFlag, SubmessageKindPIM, TimePIM,
        },
        RTPSMessagePIM, RtpsMessageHeaderPIM, RtpsSubmessageHeaderPIM,
    },
    structure::types::{DataPIM, InstanceHandlePIM},
};

pub mod submessages;

#[derive(Debug, PartialEq)]
pub struct RtpsUdpPsm;

impl InstanceHandlePIM for RtpsUdpPsm {
    type InstanceHandleType = InstanceHandle;
}

impl DataPIM for RtpsUdpPsm {
    type DataType = Data;
}

impl ProtocolIdPIM for RtpsUdpPsm {
    type ProtocolIdType = ProtocolId;
    const PROTOCOL_RTPS: Self::ProtocolIdType = [b'R', b'T', b'P', b'S'];
}

impl ParameterListSubmessageElementPIM for RtpsUdpPsm {
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
    type RtpsSubmessageHeaderType = submessages::SubmessageHeader;
}

impl AckNackSubmessagePIM for RtpsUdpPsm {
    type AckNackSubmessageType = submessages::ack_nack::AckNack;
}

impl<'a> DataSubmessagePIM<'a, Self> for RtpsUdpPsm {
    type DataSubmessageType = submessages::data::DataSubmesage<'a>;
}

impl<'a> DataFragSubmessagePIM<'a, Self> for RtpsUdpPsm {
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

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
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

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
pub struct GuidPrefix(pub [u8; 12]);

impl rust_rtps_pim::messages::submessage_elements::GuidPrefixSubmessageElementType for GuidPrefix {
    fn new(value: &rust_rtps_pim::structure::types::GuidPrefix) -> Self {
        Self(value.clone())
    }

    fn value(&self) -> &rust_rtps_pim::structure::types::GuidPrefix {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EntityId(rust_rtps_pim::structure::types::EntityId);

impl rust_rtps_pim::messages::submessage_elements::EntityIdSubmessageElementType for EntityId {
    fn new(value: &rust_rtps_pim::structure::types::EntityId) -> Self {
        Self(value.clone())
    }

    fn value(&self) -> &rust_rtps_pim::structure::types::EntityId {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SequenceNumber(i64);

impl rust_rtps_pim::messages::submessage_elements::SequenceNumberSubmessageElementType
    for SequenceNumber
{
    fn new(value: &rust_rtps_pim::structure::types::SequenceNumber) -> Self {
        Self(value.clone())
    }

    fn value(&self) -> &rust_rtps_pim::structure::types::SequenceNumber {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct SequenceNumberSet {
    base: rust_rtps_pim::structure::types::SequenceNumber,
    set: Vec<rust_rtps_pim::structure::types::SequenceNumber>,
    num_bits: ULong,
    bitmap: Vec<i32>,
}
impl PartialEq for SequenceNumberSet {
    fn eq(&self, other: &Self) -> bool {
        use rust_rtps_pim::messages::submessage_elements::SequenceNumberSetSubmessageElementType;
        self.base() == other.base() && self.set() == other.set()
    }
}

impl SequenceNumberSet {
    pub fn len(&self) -> u16 {
        12 /*bitmapBase + numBits */ + 4 * self.bitmap.len() /* bitmap[0] .. bitmap[M-1] */ as u16
    }
    pub fn from_bitmap(
        bitmap_base: rust_rtps_pim::structure::types::SequenceNumber,
        bitmap: Vec<i32>,
    ) -> Self {
        let mut set = vec![];
        let num_bits = 32 * bitmap.len();
        for delta_n in 0..num_bits {
            if (bitmap[delta_n / 32] & (1 << (31 - delta_n % 32))) == (1 << (31 - delta_n % 32)) {
                let seq_num = bitmap_base + delta_n as i64;
                set.push(seq_num.into());
            }
        }
        Self {
            base: bitmap_base,
            set,
            num_bits: ULong(num_bits as u32),
            bitmap,
        }
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

struct SequenceNumberSetVisitor;

impl<'de> serde::de::Visitor<'de> for SequenceNumberSetVisitor {
    type Value = SequenceNumberSet;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("SequenceNumberSet Submessage Element")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let base: rust_rtps_pim::structure::types::SequenceNumber = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
        let num_bits: u32 = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
        let num_bitmaps = (num_bits + 31) / 32; //In standard refered to as "M"
        let mut set = vec![];
        for i in 0..num_bitmaps as usize {
            let bitmap = seq
                .next_element()?
                .ok_or_else(|| serde::de::Error::invalid_length(i + 2, &self))?;
            set.push(bitmap);
        }
        Ok(SequenceNumberSet::from_bitmap(base, set))
    }
}

impl<'de> serde::Deserialize<'de> for SequenceNumberSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const MAX_BITMAPS: usize = 8;
        const OTHER_FIELDS: usize = 2; /* base + num_bits */
        const MAX_FIELDS: usize = MAX_BITMAPS + OTHER_FIELDS;
        deserializer.deserialize_tuple(MAX_FIELDS, SequenceNumberSetVisitor)
    }
}

impl rust_rtps_pim::messages::submessage_elements::SequenceNumberSetSubmessageElementType
    for SequenceNumberSet
{
    fn new(
        base: &rust_rtps_pim::structure::types::SequenceNumber,
        set: &[rust_rtps_pim::structure::types::SequenceNumber],
    ) -> Self {
        let max = set.iter().max();
        let num_bits = match max {
            Some(max) => Into::<i64>::into(*max) - Into::<i64>::into(*base) + 1,
            None => 0,
        };
        let number_of_bitmap_elements = ((num_bits + 31) / 32) as usize; // aka "M"
        let mut bitmap = vec![0; number_of_bitmap_elements];
        for sequence_number in set.iter() {
            let delta_n = Into::<i64>::into(*sequence_number) - Into::<i64>::into(*base);
            let bitmap_num = delta_n / 32;
            bitmap[bitmap_num as usize] |= 1 << (31 - delta_n % 32);
        }
        Self {
            base: base.clone(),
            set: set.into_iter().map(|x| x.clone()).collect(),
            num_bits: ULong(num_bits as u32),
            bitmap,
        }
    }

    fn base(&self) -> &rust_rtps_pim::structure::types::SequenceNumber {
        &self.base
    }

    fn set(&self) -> &[rust_rtps_pim::structure::types::SequenceNumber] {
        &self.set
    }
}

pub type InstanceHandle = i32;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
}

impl rust_rtps_pim::messages::submessage_elements::ProtocolVersionSubmessageElementType
    for ProtocolVersion
{
    fn new(value: &rust_rtps_pim::structure::types::ProtocolVersion) -> Self {
        todo!()
    }

    fn value(&self) -> &rust_rtps_pim::structure::types::ProtocolVersion {
        todo!()
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

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct VendorId([u8; 2]);

impl rust_rtps_pim::messages::submessage_elements::VendorIdSubmessageElementType
    for VendorId
{
    fn new(value: &rust_rtps_pim::structure::types::VendorId) -> Self {
        Self(value.clone())
    }

    fn value(&self) -> &rust_rtps_pim::structure::types::VendorId {
        &self.0
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

#[derive(Debug, PartialEq, Clone, Copy, serde::Serialize)]
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

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct FragmentNumber(u32);

impl rust_rtps_pim::messages::submessage_elements::FragmentNumberSubmessageElementType
    for FragmentNumber
{
    fn new(value: &rust_rtps_pim::messages::types::FragmentNumber) -> Self {
        Self(value.clone())
    }

    fn value(&self) -> &rust_rtps_pim::messages::types::FragmentNumber {
        &self.0
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

impl rust_rtps_pim::messages::submessage_elements::FragmentNumberSetSubmessageElementType
    for FragmentNumberSet
{
    fn new(
        _base: &rust_rtps_pim::messages::types::FragmentNumber,
        _set: &[rust_rtps_pim::messages::types::FragmentNumber],
    ) -> Self {
        todo!()
    }

    fn base(&self) -> &rust_rtps_pim::messages::types::FragmentNumber {
        &0
    }

    fn set(&self) -> &[rust_rtps_pim::messages::types::FragmentNumber] {
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
    pub parameter_id: rust_rtps_pim::messages::types::ParameterId,
    pub length: i16,
    pub value: Vector,
}

impl Parameter {
    pub fn new(parameter_id: rust_rtps_pim::messages::types::ParameterId, value: Vector) -> Self {
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

impl rust_rtps_pim::messages::submessage_elements::ParameterType for Parameter {
    fn parameter_id(&self) -> rust_rtps_pim::messages::types::ParameterId {
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
        let paramter_id: rust_rtps_pim::messages::types::ParameterId = seq
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
const PID_SENTINEL: rust_rtps_pim::messages::types::ParameterId = 1;
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

pub struct LocatorList(Vec<rust_rtps_pim::structure::types::Locator>);

impl rust_rtps_pim::messages::submessage_elements::LocatorListSubmessageElementType
    for LocatorList
{
    fn new(_value: &[rust_rtps_pim::structure::types::Locator]) -> Self {
        // Self(value)
        todo!()
    }

    fn value(&self) -> &[rust_rtps_pim::structure::types::Locator] {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RTPSMessageHeader {
    protocol: ProtocolId,
    version: ProtocolVersion,
    vendor_id: VendorId,
    guid_prefix: rust_rtps_pim::structure::types::GuidPrefix,
}

impl rust_rtps_pim::messages::RtpsMessageHeaderType<RtpsUdpPsm> for RTPSMessageHeader {
    fn protocol(&self) -> &ProtocolId {
        &self.protocol
    }

    fn version(&self) -> &rust_rtps_pim::structure::types::ProtocolVersion {
        // &self.version
        todo!()
    }

    fn vendor_id(&self) -> &rust_rtps_pim::structure::types::VendorId {
        // &self.vendor_id
        todo!()
    }

    fn guid_prefix(&self) -> &rust_rtps_pim::structure::types::GuidPrefix {
        &self.guid_prefix
    }
}

#[derive(Debug, PartialEq)]
pub struct RTPSMessageC<'a> {
    header: RTPSMessageHeader,
    submessages: Vec<RtpsSubmessageType<'a, RtpsUdpPsm>>,
}

impl<'a> rust_rtps_pim::messages::RTPSMessage<'a, RtpsUdpPsm> for RTPSMessageC<'a> {
    fn new<T: IntoIterator<Item = RtpsSubmessageType<'a, RtpsUdpPsm>>>(
        protocol: ProtocolId,
        version: rust_rtps_pim::structure::types::ProtocolVersion,
        vendor_id: rust_rtps_pim::structure::types::VendorId,
        guid_prefix: rust_rtps_pim::structure::types::GuidPrefix,
        submessages: T,
    ) -> Self {
        todo!()
        // RTPSMessageC {
        //     header: RTPSMessageHeader {
        //         protocol: protocol.clone(),
        //         version: version.clone(),
        //         vendor_id: vendor_id.clone(),
        //         guid_prefix: guid_prefix.clone(),
        //     },
        //     submessages: submessages.into_iter().collect(),
        // }
    }

    fn header(&self) -> RTPSMessageHeader {
        self.header
    }

    fn submessages(&self) -> &[RtpsSubmessageType<'a, RtpsUdpPsm>] {
        &self.submessages
    }
}

impl<'a> serde::Serialize for RTPSMessageC<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let len = self.submessages.len();
        let mut state = serializer.serialize_struct("RTPSMessage", len)?;
        state.serialize_field("header", &self.header)?;
        for submessage in &self.submessages {
            match submessage {
                //RtpsSubmessageType::AckNack(submessage) => state.serialize_field("submessage", submessage)?,
                RtpsSubmessageType::Data(submessage) => {
                    state.serialize_field("submessage", submessage)?
                }
                //RtpsSubmessageType::DataFrag(submessage) => state.serialize_field("submessage", submessage)?,
                RtpsSubmessageType::Gap(submessage) => {
                    state.serialize_field("submessage", submessage)?
                }
                // RtpsSubmessageType::Heartbeat(submessage) => state.serialize_field("submessage", submessage)?,
                // RtpsSubmessageType::HeartbeatFrag(submessage) => state.serialize_field("submessage", submessage)?,
                // RtpsSubmessageType::InfoDestination(submessage) => state.serialize_field("submessage", submessage)?,
                // RtpsSubmessageType::InfoReply(submessage) => state.serialize_field("submessage", submessage)?,
                // RtpsSubmessageType::InfoSource(submessage) => state.serialize_field("submessage", submessage)?,
                // RtpsSubmessageType::InfoTimestamp(submessage) => state.serialize_field("submessage", submessage)?,
                // RtpsSubmessageType::NackFrag(submessage) => state.serialize_field("submessage", submessage)?,
                //RtpsSubmessageType::Pad(submessage) => state.serialize_field("submessage", submessage)?,
                _ => todo!(),
            }
        }
        state.end()
    }
}
struct RTPSMessageVisitor<'a>(std::marker::PhantomData<&'a ()>);

impl<'a, 'de: 'a> serde::de::Visitor<'de> for RTPSMessageVisitor<'a> {
    type Value = RTPSMessageC<'a>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("RTPSMessage")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let header: RTPSMessageHeader = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
        let mut submessages = vec![];

        for _ in 0..seq.size_hint().unwrap() {
            let submessage_id_result: Result<Option<Octet>, _> = seq.next_element();
            let submessage_id: u8 = match submessage_id_result {
                Ok(submessage_id) => submessage_id
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?
                    .into(),
                Err(_) => break,
            };
            let typed_submessage = match submessage_id {
                0x08 => RtpsSubmessageType::Gap(
                    seq.next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?,
                ),
                0x15 => RtpsSubmessageType::Data(
                    seq.next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?,
                ),
                _ => todo!("Submessage type unhandled"),
            };
            submessages.push(typed_submessage);
        }
        Ok(RTPSMessageC {
            header,
            submessages,
        })
    }
}
impl<'a, 'de: 'a> serde::Deserialize<'de> for RTPSMessageC<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const MAX_SUBMESSAGES: usize = 2 ^ 16;
        const OTHER_FIELDS: usize = 1;
        const MAX_FIELDS: usize = MAX_SUBMESSAGES + OTHER_FIELDS;
        deserializer.deserialize_tuple(MAX_FIELDS, RTPSMessageVisitor(std::marker::PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use crate::submessages::data::DataSubmesage;

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

    use rust_rtps_pim::messages::{
        submessage_elements::SequenceNumberSetSubmessageElementType,
        submessages::{DataSubmessage, GapSubmessage},
    };

    #[test]
    fn serialize_sequence_number_max_gap() {
        let sequence_number_set = SequenceNumberSet::new(&2.into(), &[2.into(), 257.into()]);
        #[rustfmt::skip]
        assert_eq!(serialize(sequence_number_set), vec![
            0, 0, 0, 0, // bitmapBase: high (long)
            2, 0, 0, 0, // bitmapBase: low (unsigned long)
            0, 1, 0, 0, // numBits (ULong)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_1000_0000, // bitmap[0] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[1] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[2] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[3] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[4] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[5] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[6] (long)
            0b_000_0001, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[7] (long)
        ]);
    }

    #[test]
    fn serialize_sequence_number_set_empty() {
        let sequence_number_set = SequenceNumberSet::new(&2.into(), &[]);
        #[rustfmt::skip]
        assert_eq!(serialize(sequence_number_set), vec![
            0, 0, 0, 0, // bitmapBase: high (long)
            2, 0, 0, 0, // bitmapBase: low (unsigned long)
            0, 0, 0, 0, // numBits (ULong)
        ]);
    }

    #[test]
    fn deserialize_sequence_number_set_empty() {
        let expected = SequenceNumberSet::new(&2.into(), &[]);
        #[rustfmt::skip]
        let result = deserialize(&[
            0, 0, 0, 0, // bitmapBase: high (long)
            2, 0, 0, 0, // bitmapBase: low (unsigned long)
            0, 0, 0, 0, // numBits (ULong)
        ]);
        assert_eq!(expected, result);
    }

    #[test]
    fn deserialize_sequence_number_set_max_gap() {
        let expected = SequenceNumberSet::new(&2.into(), &[2.into(), 257.into()]);
        #[rustfmt::skip]
        let result = deserialize(&[
            0, 0, 0, 0, // bitmapBase: high (long)
            2, 0, 0, 0, // bitmapBase: low (unsigned long)
            0, 1, 0, 0, // numBits (ULong)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_1000_0000, // bitmap[0] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[1] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[2] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[3] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[4] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[5] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[6] (long)
            0b_000_0001, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[7] (long)

        ]);
        assert_eq!(expected, result);
    }

    #[test]
    fn serialize_rtps_message_header() {
        let value = RTPSMessageHeader {
            protocol: b"RTPS".to_owned(),
            version: ProtocolVersion { major: 2, minor: 3 },
            vendor_id: VendorId([9, 8]),
            guid_prefix: [3; 12],
        };
        #[rustfmt::skip]
        assert_eq!(serialize(value), vec![
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
        ]);
    }

    #[test]
    fn deserialize_rtps_message_header() {
        let expected = RTPSMessageHeader {
            protocol: b"RTPS".to_owned(),
            version: ProtocolVersion { major: 2, minor: 3 },
            vendor_id: VendorId([9, 8]),
            guid_prefix: [3; 12],
        };
        #[rustfmt::skip]
        let result = deserialize(&[
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
        ]);
        assert_eq!(expected, result);
    }

    #[test]
    fn serialize_rtps_message() {
        let header = RTPSMessageHeader {
            protocol: b"RTPS".to_owned(),
            version: ProtocolVersion { major: 2, minor: 3 },
            vendor_id: VendorId([9, 8]),
            guid_prefix: [3; 12],
        };
        let endianness_flag = true;
        let reader_id = EntityId([1, 2, 3, 4]);
        let writer_id = EntityId([6, 7, 8, 9]);
        let gap_start = SequenceNumber(5);
        let gap_list = SequenceNumberSet::new(&10.into(), &[]);
        let gap_submessage = RtpsSubmessageType::Gap(GapSubmessage::new(
            endianness_flag,
            reader_id,
            writer_id,
            gap_start,
            gap_list,
        ));

        let inline_qos_flag = false;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let writer_sn = SequenceNumber(5);
        let inline_qos = ParameterList {
            parameter: vec![].into(),
        };
        let data = [];
        let serialized_payload = SerializedData(&data[..]);
        let data_submessage = RtpsSubmessageType::Data(DataSubmesage::new(
            endianness_flag,
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        ));
        let value = RTPSMessageC {
            header,
            submessages: vec![gap_submessage, data_submessage],
        };
        #[rustfmt::skip]
        assert_eq!(serialize(value), vec![
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            0x08, 0b_0000_0001, 28, 0, // Submessage header
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // gapStart: SequenceNumber: high
            5, 0, 0, 0, // gapStart: SequenceNumber: low
            0, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: high
           10, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: low
            0, 0, 0, 0, // gapList: SequenceNumberSet: numBits (ULong)
            0x15, 0b_0000_0001, 20, 0, // Submessage header
            0, 0, 16, 0, // extraFlags, octetsToInlineQos
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: high
            5, 0, 0, 0, // writerSN: low
        ]);
    }

    #[test]
    fn deserialize_rtps_message_no_submessage() {
        let header = RTPSMessageHeader {
            protocol: b"RTPS".to_owned(),
            version: ProtocolVersion { major: 2, minor: 3 },
            vendor_id: VendorId([9, 8]),
            guid_prefix: [3; 12],
        };

        let expected = RTPSMessageC {
            header,
            submessages: vec![],
        };
        #[rustfmt::skip]
        let result: RTPSMessageC = deserialize(&[
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn deserialize_rtps_message() {
        let header = RTPSMessageHeader {
            protocol: b"RTPS".to_owned(),
            version: ProtocolVersion { major: 2, minor: 3 },
            vendor_id: VendorId([9, 8]),
            guid_prefix: [3; 12],
        };

        let endianness_flag = true;
        let reader_id = EntityId([1, 2, 3, 4]);
        let writer_id = EntityId([6, 7, 8, 9]);
        let gap_start = SequenceNumber(5);
        let gap_list = SequenceNumberSet::new(&10.into(), &[]);
        let gap_submessage = RtpsSubmessageType::Gap(GapSubmessage::new(
            endianness_flag,
            reader_id,
            writer_id,
            gap_start,
            gap_list,
        ));

        let inline_qos_flag = false;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let writer_sn = SequenceNumber(5);
        let inline_qos = ParameterList {
            parameter: vec![].into(),
        };
        let data = [];
        let serialized_payload = SerializedData(&data[..]);
        let data_submessage = RtpsSubmessageType::Data(DataSubmesage::new(
            endianness_flag,
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        ));
        let expected = RTPSMessageC {
            header,
            submessages: vec![gap_submessage, data_submessage],
        };
        #[rustfmt::skip]
        let result: RTPSMessageC = deserialize(&[
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            0x08_u8, 0b_0000_0001, 28, 0, // Submessage header
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // gapStart: SequenceNumber: high
            5, 0, 0, 0, // gapStart: SequenceNumber: low
            0, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: high
           10, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: low
            0, 0, 0, 0, // gapList: SequenceNumberSet: numBits (ULong)
            0x15, 0b_0000_0001, 20, 0, // Submessage header
            0, 0, 16, 0, // extraFlags, octetsToInlineQos
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: high
            5, 0, 0, 0, // writerSN: low
        ]);
        assert_eq!(result, expected);
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
