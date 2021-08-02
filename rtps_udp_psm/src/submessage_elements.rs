use std::io::{Read, Write};

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use rust_rtps_pim::{
    messages::{
        submessage_elements::GuidPrefixSubmessageElementType,
        types::{Count, FragmentNumber, SubmessageFlag, Time},
    },
    structure::types::{EntityId, EntityKind, GuidPrefix, Locator, ProtocolVersion},
};

use crate::serialize::NumberofBytes;

pub fn is_bit_set(value: u8, index: usize) -> bool {
    value & (0b_0000_0001 << index) != 0
}
pub fn flags_to_byte<const N: usize>(value: [SubmessageFlag; N]) -> u8 {
    let mut flags = 0b_0000_0000_u8;
    for (i, &item) in value.iter().enumerate() {
        if item {
            flags |= 0b_0000_0001 << i
        }
    }
    flags
}

#[derive(Debug, PartialEq)]
pub struct UShortUdp(pub(crate) u16);

impl rust_rtps_pim::messages::submessage_elements::UShortSubmessageElementType for UShortUdp {
    fn new(value: &u16) -> Self {
        Self(*value)
    }

    fn value(&self) -> u16 {
        self.0
    }
}

#[derive(Debug, PartialEq)]
pub struct LongUdp(pub(crate) i32);

impl rust_rtps_pim::messages::submessage_elements::LongSubmessageElementType for LongUdp {
    fn new(value: &i32) -> Self {
        Self(*value)
    }

    fn value(&self) -> i32 {
        self.0
    }
}

#[derive(Debug, PartialEq)]
pub struct ULongUdp(pub(crate) u32);

impl rust_rtps_pim::messages::submessage_elements::ULongSubmessageElementType for ULongUdp {
    fn new(value: &u32) -> Self {
        Self(*value)
    }

    fn value(&self) -> u32 {
        self.0
    }
}

#[derive(Debug, PartialEq)]
pub struct GuidPrefixUdp(pub(crate) [u8; 12]);
impl From<&GuidPrefixUdp> for GuidPrefix {
    fn from(v: &GuidPrefixUdp) -> Self {
        v.0
    }
}
impl From<GuidPrefix> for GuidPrefixUdp {
    fn from(v: GuidPrefix) -> Self {
        Self(v)
    }
}
impl GuidPrefixSubmessageElementType for GuidPrefixUdp {
    fn new(value: &GuidPrefix) -> Self {
        Self((*value).into())
    }

    fn value(&self) -> GuidPrefix {
        self.into()
    }
}

impl crate::serialize::Serialize for GuidPrefixUdp {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> crate::serialize::Result {
        self.0.serialize::<_, B>(&mut writer)
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for GuidPrefixUdp {
    fn deserialize<B>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        Ok(Self(crate::deserialize::Deserialize::deserialize::<B>(
            buf,
        )?))
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct EntityIdUdp {
    pub entity_key: [u8; 3],
    pub entity_kind: u8,
}
impl From<EntityIdUdp> for EntityId {
    fn from(v: EntityIdUdp) -> Self {
        EntityId::new(
            [v.entity_key[0], v.entity_key[1], v.entity_key[2]],
            u8_into_entity_kind(v.entity_kind),
        )
    }
}
impl From<EntityId> for EntityIdUdp {
    fn from(v: EntityId) -> Self {
        Self {
            entity_key: [v.entity_key()[0], v.entity_key()[1], v.entity_key()[2]],
            entity_kind: entity_kind_into_u8(v.entity_kind()),
        }
    }
}

impl crate::serialize::Serialize for EntityIdUdp {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> crate::serialize::Result {
        self.entity_key.serialize::<_, B>(&mut writer)?;
        self.entity_kind.serialize::<_, B>(&mut writer)
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for EntityIdUdp {
    fn deserialize<B>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        let entity_key = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        let entity_kind = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        Ok(Self {
            entity_key,
            entity_kind,
        })
    }
}

impl rust_rtps_pim::messages::submessage_elements::EntityIdSubmessageElementType for EntityIdUdp {
    fn new(value: &EntityId) -> Self {
        (*value).into()
    }

    fn value(&self) -> EntityId {
        (*self).into()
    }
}

pub fn entity_kind_into_u8(value: &EntityKind) -> u8 {
    match value {
        EntityKind::UserDefinedUnknown => 0x00,
        EntityKind::BuiltInUnknown => 0xc0,
        EntityKind::BuiltInParticipant => 0xc1,
        EntityKind::UserDefinedWriterWithKey => 0x02,
        EntityKind::BuiltInWriterWithKey => 0xc2,
        EntityKind::UserDefinedWriterNoKey => 0x03,
        EntityKind::BuiltInWriterNoKey => 0xc3,
        EntityKind::UserDefinedReaderWithKey => 0x07,
        EntityKind::BuiltInReaderWithKey => 0xc7,
        EntityKind::UserDefinedReaderNoKey => 0x04,
        EntityKind::BuiltInReaderNoKey => 0xc4,
        EntityKind::UserDefinedWriterGroup => 0x08,
        EntityKind::BuiltInWriterGroup => 0xc8,
        EntityKind::UserDefinedReaderGroup => 0x09,
        EntityKind::BuiltInReaderGroup => 0xc9,
    }
}
pub fn u8_into_entity_kind(_value: u8) -> EntityKind {
    todo!()
}

#[derive(Debug, PartialEq, Eq)]
pub struct SequenceNumberUdp {
    pub(crate) high: i32,
    pub(crate) low: u32,
}
impl crate::serialize::Serialize for SequenceNumberUdp {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> crate::serialize::Result {
        self.high.serialize::<_, B>(&mut writer)?;
        self.low.serialize::<_, B>(&mut writer)
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for SequenceNumberUdp {
    fn deserialize<B>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        let high = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        let low = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        Ok(Self { high, low })
    }
}
impl From<&SequenceNumberUdp> for i64 {
    fn from(value: &SequenceNumberUdp) -> Self {
        ((value.high as i64) << 32) + value.low as i64
    }
}
impl From<&i64> for SequenceNumberUdp {
    fn from(value: &i64) -> Self {
        Self {
            high: (value >> 32) as i32,
            low: *value as u32,
        }
    }
}

impl rust_rtps_pim::messages::submessage_elements::SequenceNumberSubmessageElementType
    for SequenceNumberUdp
{
    fn new(value: &rust_rtps_pim::structure::types::SequenceNumber) -> Self {
        value.into()
    }

    fn value(&self) -> rust_rtps_pim::structure::types::SequenceNumber {
        self.into()
    }
}

#[derive(Debug, PartialEq)]
pub struct SequenceNumberSetUdp {
    base: SequenceNumberUdp,
    num_bits: u32,
    bitmap: [i32; 8],
}

impl SequenceNumberSetUdp {
    pub fn len(&self) -> u16 {
        let number_of_bitmap_elements = ((self.num_bits + 31) / 32) as usize; // aka "M"
        12 /*bitmapBase + numBits */ + 4 * number_of_bitmap_elements /* bitmap[0] .. bitmap[M-1] */ as u16
    }
}

impl crate::serialize::Serialize for SequenceNumberSetUdp {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> crate::serialize::Result {
        self.base.serialize::<_, B>(&mut writer)?;
        self.num_bits.serialize::<_, B>(&mut writer)?;
        let number_of_bitmap_elements = ((self.num_bits + 31) / 32) as usize; //In standard refered to as "M"
        for i in 0..number_of_bitmap_elements {
            self.bitmap[i].serialize::<_, B>(&mut writer)?;
        }
        Ok(())
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for SequenceNumberSetUdp {
    fn deserialize<B>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        let base = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        let num_bits = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        let num_bitmaps = (num_bits + 31) / 32; //In standard refered to as "M"
        let mut bitmap = [0; 8];
        for i in 0..num_bitmaps as usize {
            bitmap[i] = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        }
        Ok(Self {
            base,
            num_bits,
            bitmap,
        })
    }
}

impl rust_rtps_pim::messages::submessage_elements::SequenceNumberSetSubmessageElementType
    for SequenceNumberSetUdp
{
    type IntoIter = std::vec::IntoIter<rust_rtps_pim::structure::types::SequenceNumber>;

    fn new(
        base: &rust_rtps_pim::structure::types::SequenceNumber,
        set: &[rust_rtps_pim::structure::types::SequenceNumber],
    ) -> Self {
        let mut bitmap = [0; 8];
        let mut num_bits = 0;
        for sequence_number in set.iter() {
            let delta_n = (sequence_number - base) as u32;
            let bitmap_num = delta_n / 32;
            bitmap[bitmap_num as usize] |= 1 << (31 - delta_n % 32);
            if delta_n + 1 > num_bits {
                num_bits = delta_n + 1;
            }
        }
        Self {
            base: base.into(),
            num_bits,
            bitmap,
        }
    }

    fn base(&self) -> rust_rtps_pim::structure::types::SequenceNumber {
        (&self.base).into()
    }

    fn set(&self) -> Self::IntoIter {
        let mut set = vec![];
        for delta_n in 0..self.num_bits as usize {
            if (self.bitmap[delta_n / 32] & (1 << (31 - delta_n % 32)))
                == (1 << (31 - delta_n % 32))
            {
                let seq_num =
                    Into::<rust_rtps_pim::structure::types::SequenceNumber>::into(&self.base)
                        + delta_n as rust_rtps_pim::structure::types::SequenceNumber;
                set.push(seq_num);
            }
        }
        set.into_iter()
    }
}

pub type InstanceHandleUdp = i32;

#[derive(Debug, PartialEq)]
pub struct ProtocolVersionUdp {
    pub major: u8,
    pub minor: u8,
}

impl crate::serialize::Serialize for ProtocolVersionUdp {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> crate::serialize::Result {
        self.major.serialize::<_, B>(&mut writer)?;
        self.minor.serialize::<_, B>(&mut writer)
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for ProtocolVersionUdp {
    fn deserialize<B>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        let major = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        let minor = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        Ok(Self { major, minor })
    }
}
impl NumberofBytes for ProtocolVersionUdp {
    fn number_of_bytes(&self) -> usize {
        2
    }
}

impl rust_rtps_pim::messages::submessage_elements::ProtocolVersionSubmessageElementType
    for ProtocolVersionUdp
{
    fn new(value: &ProtocolVersion) -> Self {
        Self {
            major: value.major,
            minor: value.minor,
        }
    }

    fn value(&self) -> ProtocolVersion {
        ProtocolVersion {
            major: self.major,
            minor: self.minor,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SerializedDataUdp<'a>(pub &'a [u8]);

impl<'a> SerializedDataUdp<'a> {
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> rust_rtps_pim::messages::submessage_elements::SerializedDataSubmessageElementType<'a>
    for SerializedDataUdp<'a>
{
    fn new(value: &'a [u8]) -> Self {
        SerializedDataUdp(value)
    }

    fn value(&self) -> &'a [u8] {
        self.0
    }
}

impl<'a> crate::serialize::Serialize for SerializedDataUdp<'a> {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> crate::serialize::Result {
        self.0.serialize::<_, B>(&mut writer)
    }
}

impl<'a>
    rust_rtps_pim::messages::submessage_elements::SerializedDataFragmentSubmessageElementType<'a>
    for SerializedDataUdp<'a>
{
    fn new(value: &'a [u8]) -> Self {
        Self(value)
    }

    fn value(&self) -> &'a [u8] {
        self.0
    }
}

#[derive(Debug, PartialEq)]
pub struct VendorIdUdp(pub(crate) [u8; 2]);

impl crate::serialize::Serialize for VendorIdUdp {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> crate::serialize::Result {
        self.0.serialize::<_, B>(&mut writer)
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for VendorIdUdp {
    fn deserialize<B>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        Ok(Self(crate::deserialize::Deserialize::deserialize::<B>(
            buf,
        )?))
    }
}
impl NumberofBytes for VendorIdUdp {
    fn number_of_bytes(&self) -> usize {
        2
    }
}

impl rust_rtps_pim::messages::submessage_elements::VendorIdSubmessageElementType for VendorIdUdp {
    fn new(value: &rust_rtps_pim::structure::types::VendorId) -> Self {
        Self(value.clone())
    }

    fn value(&self) -> rust_rtps_pim::structure::types::VendorId {
        self.0
    }
}

#[derive(Debug, PartialEq)]
pub struct TimeUdp {
    pub seconds: u32,
    pub fraction: u32,
}
impl crate::serialize::Serialize for TimeUdp {
    fn serialize<W: Write, B: ByteOrder>(&self, mut _writer: W) -> crate::serialize::Result {
        todo!()
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for TimeUdp {
    fn deserialize<B>(_buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        todo!()
    }
}
impl<'a> rust_rtps_pim::messages::submessage_elements::TimestampSubmessageElementType for TimeUdp {
    fn new(_value: &Time) -> Self {
        todo!()
    }

    fn value(&self) -> Time {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct CountUdp(pub(crate) i32);

impl crate::serialize::Serialize for CountUdp {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> crate::serialize::Result {
        self.0.serialize::<_, B>(&mut writer)
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for CountUdp {
    fn deserialize<B>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        Ok(Self(crate::deserialize::Deserialize::deserialize::<B>(
            buf,
        )?))
    }
}
impl NumberofBytes for CountUdp {
    fn number_of_bytes(&self) -> usize {
        4
    }
}

impl<'a> rust_rtps_pim::messages::submessage_elements::CountSubmessageElementType for CountUdp {
    fn new(value: &Count) -> Self {
        Self(value.0)
    }

    fn value(&self) -> Count {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct FragmentNumberUdp(pub(crate) u32);

impl crate::serialize::Serialize for FragmentNumberUdp {
    fn serialize<W: Write, B: ByteOrder>(&self, mut _writer: W) -> crate::serialize::Result {
        todo!()
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for FragmentNumberUdp {
    fn deserialize<B>(_buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        todo!()
    }
}

impl rust_rtps_pim::messages::submessage_elements::FragmentNumberSubmessageElementType
    for FragmentNumberUdp
{
    fn new(value: &FragmentNumber) -> Self {
        Self(value.0)
    }

    fn value(&self) -> FragmentNumber {
        FragmentNumber(self.0)
    }
}

impl From<u32> for FragmentNumberUdp {
    fn from(_: u32) -> Self {
        todo!()
    }
}

impl Into<u32> for FragmentNumberUdp {
    fn into(self) -> u32 {
        todo!()
    }
}

pub struct FragmentNumberSetUdp(Vec<FragmentNumberUdp>);

impl crate::serialize::Serialize for FragmentNumberSetUdp {
    fn serialize<W: Write, B: ByteOrder>(&self, mut _writer: W) -> crate::serialize::Result {
        todo!()
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for FragmentNumberSetUdp {
    fn deserialize<B>(_buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        todo!()
    }
}

impl rust_rtps_pim::messages::submessage_elements::FragmentNumberSetSubmessageElementType
    for FragmentNumberSetUdp
{
    type IntoIter = Vec<FragmentNumber>;
    fn new(_base: &FragmentNumber, _set: &[FragmentNumber]) -> Self {
        todo!()
    }

    fn base(&self) -> FragmentNumber {
        todo!()
    }

    fn set(&self) -> Self::IntoIter {
        todo!()
    }
}

pub type GroupDigestUdp = [u8; 4];

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct LocatorUdp {
    kind: i32,
    port: u32,
    address: [u8; 16],
}

impl LocatorUdp {
    pub fn new(locator: &Locator) -> Self {
        Self {
            kind: *locator.kind(),
            port: *locator.port(),
            address: *locator.address(),
        }
    }

    pub fn value(&self) -> Locator {
        Locator::new(self.kind, self.port, self.address)
    }
}
impl NumberofBytes for LocatorUdp {
    fn number_of_bytes(&self) -> usize {
        24
    }
}
impl<const N: usize> crate::serialize::Serialize for [u8; N] {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> crate::serialize::Result {
        writer.write(self.as_ref()).map(|_| ())
    }
}
impl<'de, const N: usize> crate::deserialize::Deserialize<'de> for [u8; N] {
    fn deserialize<T>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        T: ByteOrder,
    {
        let mut this = [0; N];
        buf.read(&mut this)?;
        Ok(this)
    }
}
impl<const N: usize> NumberofBytes for [u8; N] {
    fn number_of_bytes(&self) -> usize {
        N
    }
}

impl crate::serialize::Serialize for u8 {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> crate::serialize::Result {
        writer.write_u8(*self)
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for u8 {
    fn deserialize<B>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        buf.read_u8()
    }
}
impl crate::serialize::Serialize for i32 {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> crate::serialize::Result {
        writer.write_i32::<B>(*self)
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for i32 {
    fn deserialize<B>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        buf.read_i32::<B>()
    }
}

impl crate::serialize::Serialize for u32 {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> crate::serialize::Result {
        writer.write_u32::<B>(*self)
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for u32 {
    fn deserialize<B>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        buf.read_u32::<B>()
    }
}
impl NumberofBytes for u32 {
    fn number_of_bytes(&self) -> usize {
        4
    }
}

impl crate::serialize::Serialize for LocatorUdp {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> super::serialize::Result {
        self.kind.serialize::<_, B>(&mut writer)?;
        self.port.serialize::<_, B>(&mut writer)?;
        self.address.serialize::<_, B>(&mut writer)
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for LocatorUdp {
    fn deserialize<B>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        let kind = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        let port = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        let address = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        Ok(Self {
            kind,
            port,
            address,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct LocatorListUdp(pub Vec<LocatorUdp>);

impl rust_rtps_pim::messages::submessage_elements::LocatorListSubmessageElementType
    for LocatorListUdp
{
    type IntoIter = Vec<rust_rtps_pim::structure::types::Locator>;

    fn new(value: &[rust_rtps_pim::structure::types::Locator]) -> Self {
        let mut locator_list = Vec::new();
        for locator in value {
            locator_list.push(LocatorUdp::new(locator));
        }
        Self(locator_list)
    }

    fn value(&self) -> Self::IntoIter {
        let mut locator_list = Vec::new();
        for locator_udp in &self.0 {
            let locator = rust_rtps_pim::structure::types::Locator::new(
                locator_udp.kind,
                locator_udp.port,
                locator_udp.address,
            );
            locator_list.push(locator);
        }
        locator_list
    }
}

impl crate::serialize::Serialize for LocatorListUdp {
    fn serialize<W: Write, B: ByteOrder>(&self, mut _writer: W) -> crate::serialize::Result {
        todo!()
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for LocatorListUdp {
    fn deserialize<B>(_buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deserialize::from_bytes_le;
    use crate::serialize::to_bytes_le;

    use rust_rtps_pim::messages::submessage_elements::{
        SequenceNumberSetSubmessageElementType, SequenceNumberSubmessageElementType,
    };

    #[test]
    fn octet_from_submessage_flags() {
        let result: u8 = flags_to_byte([true, false, true]);
        assert_eq!(result, 0b_0000_0101);
    }

    #[test]
    fn octet_from_submessage_flags_empty() {
        let result: u8 = flags_to_byte([]);
        assert_eq!(result, 0b_0000_0000);
    }
    #[test]
    #[should_panic]
    fn octet_from_submessage_flags_overflow() {
        let _: u8 = flags_to_byte([true; 9]);
    }

    #[test]
    fn octet_is_set_bit() {
        let flags = 0b_0000_0001;
        assert_eq!(is_bit_set(flags, 0), true);

        let flags = 0b_0000_0000;
        assert_eq!(is_bit_set(flags, 0), false);

        let flags = 0b_0000_0010;
        assert_eq!(is_bit_set(flags, 1), true);

        let flags = 0b_1000_0011;
        assert_eq!(is_bit_set(flags, 7), true);
    }

    #[test]
    fn serialize_octet() {
        assert_eq!(to_bytes_le(&5_u8).unwrap(), vec![5]);
    }

    #[test]
    fn deserialize_octet() {
        let result: u8 = from_bytes_le(&[5]).unwrap();
        assert_eq!(result, 5);
    }

    #[test]
    fn serialize_serialized_data() {
        let data = SerializedDataUdp(&[1, 2]);
        assert_eq!(to_bytes_le(&data).unwrap(), vec![1, 2]);
    }

    #[test]
    fn serialize_guid_prefix() {
        let data = GuidPrefixUdp([1; 12]);
        #[rustfmt::skip]
        assert_eq!(to_bytes_le(&data).unwrap(), vec![
            1, 1, 1, 1,
            1, 1, 1, 1,
            1, 1, 1, 1,
        ]);
    }

    #[test]
    fn deserialize_guid_prefix() {
        let expected = GuidPrefixUdp([1; 12]);
        #[rustfmt::skip]
        assert_eq!(expected, from_bytes_le(&[
            1, 1, 1, 1,
            1, 1, 1, 1,
            1, 1, 1, 1,
        ]).unwrap());
    }

    #[test]
    fn sequence_number_set_submessage_element_type_constructor() {
        let expected = SequenceNumberSetUdp {
            base: SequenceNumberUdp::new(&2),
            num_bits: 0,
            bitmap: [0; 8],
        };
        assert_eq!(SequenceNumberSetUdp::new(&2, &[]), expected);

        let expected = SequenceNumberSetUdp {
            base: SequenceNumberUdp::new(&2),
            num_bits: 1,
            bitmap: [
                0b_10000000_00000000_00000000_00000000_u32 as i32,
                0b_00000000_00000000_00000000_00000000,
                0b_00000000_00000000_00000000_00000000,
                0b_00000000_00000000_00000000_00000000,
                0b_00000000_00000000_00000000_00000000,
                0b_00000000_00000000_00000000_00000000,
                0b_00000000_00000000_00000000_00000000,
                0b_00000000_00000000_00000000_00000000,
            ],
        };
        assert_eq!(SequenceNumberSetUdp::new(&2, &[2]), expected);

        let expected = SequenceNumberSetUdp {
            base: SequenceNumberUdp::new(&2),
            num_bits: 256,
            bitmap: [
                0b_10000000_00000000_00000000_00000000_u32 as i32,
                0b_00000000_00000000_00000000_00000000,
                0b_00000000_00000000_00000000_00000000,
                0b_00000000_00000000_00000000_00000000,
                0b_00000000_00000000_00000000_00000000,
                0b_00000000_00000000_00000000_00000000,
                0b_00000000_00000000_00000000_00000000,
                0b_00000000_00000000_00000000_00000001,
            ],
        };
        assert_eq!(SequenceNumberSetUdp::new(&2, &[2, 257]), expected);
    }

    #[test]
    fn sequence_number_set_submessage_element_type_getters() {
        let sequence_number_set = SequenceNumberSetUdp {
            base: SequenceNumberUdp::new(&2),
            num_bits: 0,
            bitmap: [0; 8],
        };
        assert_eq!(sequence_number_set.base(), 2);
        assert!(sequence_number_set.set().eq(Vec::<i64>::new()));

        let sequence_number_set = SequenceNumberSetUdp {
            base: SequenceNumberUdp::new(&2),
            num_bits: 100,
            bitmap: [
                0b_10000000_00000000_00000000_00000000_u32 as i32,
                0b_00000000_00000000_00000000_00000000,
                0b_00000000_00000000_00000000_00000000,
                0b_00000000_00000000_00000000_00000000,
                0b_00000000_00000000_00000000_00000000,
                0b_00000000_00000000_00000000_00000000,
                0b_00000000_00000000_00000000_00000000,
                0b_00000000_00000000_00000000_00000000,
            ],
        };
        assert_eq!(sequence_number_set.base(), 2);
        assert!(sequence_number_set.set().eq(vec![2]));

        let sequence_number_set = SequenceNumberSetUdp {
            base: SequenceNumberUdp::new(&2),
            num_bits: 256,
            bitmap: [
                0b_10000000_00000000_00000000_00000000_u32 as i32,
                0b_00000000_00000000_00000000_00000000,
                0b_00000000_00000000_00000000_00000000,
                0b_00000000_00000000_00000000_00000000,
                0b_00000000_00000000_00000000_00000000,
                0b_00000000_00000000_00000000_00000000,
                0b_00000000_00000000_00000000_00000000,
                0b_00000000_00000000_00000000_00000001,
            ],
        };
        assert_eq!(sequence_number_set.base(), 2);
        assert!(sequence_number_set.set().eq(vec![2, 257]));
    }

    #[test]
    fn serialize_sequence_number_max_gap() {
        let sequence_number_set = SequenceNumberSetUdp::new(&2, &[2, 257]);
        #[rustfmt::skip]
        assert_eq!(to_bytes_le(&sequence_number_set).unwrap(), vec![
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
        let sequence_number_set = SequenceNumberSetUdp::new(&2, &[]);
        #[rustfmt::skip]
        assert_eq!(to_bytes_le(&sequence_number_set).unwrap(), vec![
            0, 0, 0, 0, // bitmapBase: high (long)
            2, 0, 0, 0, // bitmapBase: low (unsigned long)
            0, 0, 0, 0, // numBits (ULong)
        ]);
    }

    #[test]
    fn deserialize_sequence_number_set_empty() {
        let expected = SequenceNumberSetUdp::new(&2, &[]);
        #[rustfmt::skip]
        let result = from_bytes_le(&[
            0, 0, 0, 0, // bitmapBase: high (long)
            2, 0, 0, 0, // bitmapBase: low (unsigned long)
            0, 0, 0, 0, // numBits (ULong)
        ]).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn deserialize_sequence_number_set_max_gap() {
        let expected = SequenceNumberSetUdp::new(&2, &[2, 257]);
        #[rustfmt::skip]
        let result = from_bytes_le(&[
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

        ]).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn serialize_locator() {
        let locator = LocatorUdp {
            kind: 1,
            port: 2,
            address: [3; 16],
        };
        #[rustfmt::skip]
        assert_eq!(to_bytes_le(&locator).unwrap(), vec![
            1, 0, 0, 0, // kind (long)
            2, 0, 0, 0, // port (unsigned long)
            3, 3, 3, 3, // address (octet[16])
            3, 3, 3, 3, // address (octet[16])
            3, 3, 3, 3, // address (octet[16])
            3, 3, 3, 3, // address (octet[16])
        ]);
    }

    #[test]
    fn deserialize_locator() {
        let expected = LocatorUdp {
            kind: 1,
            port: 2,
            address: [3; 16],
        };
        #[rustfmt::skip]
        let result: LocatorUdp = from_bytes_le(&[
            1, 0, 0, 0, // kind (long)
            2, 0, 0, 0, // port (unsigned long)
            3, 3, 3, 3, // address (octet[16])
            3, 3, 3, 3, // address (octet[16])
            3, 3, 3, 3, // address (octet[16])
            3, 3, 3, 3, // address (octet[16])
        ]).unwrap();
        assert_eq!(expected, result);
    }
}
