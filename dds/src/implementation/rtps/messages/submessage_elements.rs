use super::{
    overall_structure::{FromBytes, WriteBytes},
    types::{ParameterId, Time},
};
use crate::implementation::{
    data_representation_builtin_endpoints::parameter_id_values::PID_SENTINEL,
    rtps::{
        messages::types::{Count, FragmentNumber},
        types::{
            EntityId, GuidPrefix, Locator, LocatorAddress, ProtocolVersion, SequenceNumber,
            VendorId,
        },
    },
};
use std::io::BufRead;
///
/// This files shall only contain the types as listed in the DDS-RTPS Version 2.3
/// 8.3.5 RTPS SubmessageElements
///

#[derive(Debug, PartialEq, Eq)]
pub enum SubmessageElement<'a> {
    Count(Count),
    EntityId(EntityId),
    FragmentNumber(FragmentNumber),
    FragmentNumberSet(FragmentNumberSet),
    GuidPrefix(GuidPrefix),
    LocatorList(LocatorList),
    Long(i32),
    ParameterList(&'a ParameterList),
    ProtocolVersion(ProtocolVersion),
    SequenceNumber(SequenceNumber),
    SequenceNumberSet(SequenceNumberSet),
    SerializedData(&'a Data),
    Timestamp(Time),
    ULong(u32),
    UShort(u16),
    VendorId(VendorId),
}

impl WriteBytes for SubmessageElement<'_> {
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
        match self {
            SubmessageElement::Count(e) => e.write_bytes(buf),
            SubmessageElement::EntityId(e) => e.write_bytes(buf),
            SubmessageElement::FragmentNumber(e) => e.write_bytes(buf),
            SubmessageElement::FragmentNumberSet(e) => e.write_bytes(buf),
            SubmessageElement::GuidPrefix(e) => e.write_bytes(buf),
            SubmessageElement::LocatorList(e) => e.write_bytes(buf),
            SubmessageElement::Long(e) => e.write_bytes(buf),
            SubmessageElement::ParameterList(e) => e.write_bytes(buf),
            SubmessageElement::ProtocolVersion(e) => e.write_bytes(buf),
            SubmessageElement::SequenceNumber(e) => e.write_bytes(buf),
            SubmessageElement::SequenceNumberSet(e) => e.write_bytes(buf),
            SubmessageElement::SerializedData(e) => e.write_bytes(buf),
            SubmessageElement::Timestamp(e) => e.write_bytes(buf),
            SubmessageElement::ULong(e) => e.write_bytes(buf),
            SubmessageElement::UShort(e) => e.write_bytes(buf),
            SubmessageElement::VendorId(e) => e.write_bytes(buf),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SequenceNumberSet {
    base: SequenceNumber,
    set: Vec<SequenceNumber>,
}

impl SequenceNumberSet {
    pub fn new(base: SequenceNumber, set: Vec<SequenceNumber>) -> Self {
        if let Some(&min) = set.iter().min() {
            let max = *set.iter().max().unwrap();
            if !(max - min < SequenceNumber::from(256) && min >= SequenceNumber::from(1)) {
                panic!("SequenceNumber set max - min < 256 && min >= 1 must hold")
            }
        }
        Self { base, set }
    }

    pub fn base(&self) -> SequenceNumber {
        self.base
    }

    pub fn set(&self) -> &[SequenceNumber] {
        self.set.as_ref()
    }
}

impl WriteBytes for SequenceNumberSet {
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
        let mut bitmap = [0; 8];
        let mut num_bits = 0;
        for sequence_number in &self.set {
            let delta_n = <i64>::from(*sequence_number - self.base) as u32;
            let bitmap_num = delta_n / 32;
            bitmap[bitmap_num as usize] |= 1 << (31 - delta_n % 32);
            if delta_n + 1 > num_bits {
                num_bits = delta_n + 1;
            }
        }
        let number_of_bitmap_elements = ((num_bits + 31) / 32) as usize; //In standard referred to as "M"

        self.base.write_bytes(&mut buf[0..]);
        num_bits.write_bytes(&mut buf[8..]);
        let mut len = 12;
        for bitmap_element in &bitmap[..number_of_bitmap_elements] {
            bitmap_element.write_bytes(&mut buf[len..]);
            len += 4;
        }
        len
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FragmentNumberSet {
    base: FragmentNumber,
    set: Vec<FragmentNumber>,
}

impl FragmentNumberSet {
    pub fn new(base: FragmentNumber, set: Vec<FragmentNumber>) -> Self {
        if let Some(&min) = set.iter().min() {
            let max = *set.iter().max().unwrap();
            if !(max - min < FragmentNumber::new(256) && min >= FragmentNumber::new(1)) {
                panic!("FragmentNumberSet set max - min < 256 && min >= 1 must hold")
            }
        }
        Self { base, set }
    }
}

impl WriteBytes for FragmentNumberSet {
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
        let mut bitmap = [0; 8];
        let mut num_bits = 0;
        for fragment_number in &self.set {
            let delta_n = <u32>::from(*fragment_number - self.base);
            let bitmap_num = delta_n / 32;
            bitmap[bitmap_num as usize] |= 1 << (31 - delta_n % 32);
            if delta_n + 1 > num_bits {
                num_bits = delta_n + 1;
            }
        }
        let number_of_bitmap_elements = ((num_bits + 31) / 32) as usize; //In standard referred to as "M"

        let mut len = 0;
        len += self.base.write_bytes(&mut buf[len..]);
        len += num_bits.write_bytes(&mut buf[len..]);

        for bitmap_element in &bitmap[..number_of_bitmap_elements] {
            len += bitmap_element.write_bytes(&mut buf[len..]);
        }
        len
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocatorList {
    value: Vec<Locator>,
}

impl LocatorList {
    pub fn new(value: Vec<Locator>) -> Self {
        Self { value }
    }

    pub fn value(&self) -> &[Locator] {
        self.value.as_ref()
    }
}

impl WriteBytes for LocatorList {
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
        let num_locators = self.value().len() as u32;
        num_locators.write_bytes(&mut buf[0..]);
        let mut len = 4;
        for locator in self.value().iter() {
            len += locator.write_bytes(&mut buf[len..]);
        }
        len
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Parameter {
    parameter_id: ParameterId,
    value: Vec<u8>,
}

impl Parameter {
    pub fn new(parameter_id: ParameterId, value: Vec<u8>) -> Self {
        Self {
            parameter_id,
            value,
        }
    }

    pub fn parameter_id(&self) -> ParameterId {
        self.parameter_id
    }

    pub fn value(&self) -> &[u8] {
        self.value.as_ref()
    }

    pub fn length(&self) -> i16 {
        self.value.len() as i16
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParameterList {
    parameter: Vec<Parameter>,
}

impl ParameterList {
    pub fn new(parameter: Vec<Parameter>) -> Self {
        Self { parameter }
    }

    pub fn empty() -> Self {
        Self { parameter: vec![] }
    }

    pub fn parameter(&self) -> &[Parameter] {
        self.parameter.as_ref()
    }
}

impl FromBytes for Parameter {
    fn from_bytes<E: byteorder::ByteOrder>(v: &[u8]) -> Self {
        let parameter_id = E::read_u16(&v[0..]);
        let length = E::read_i16(&v[2..]);
        let value = if parameter_id == PID_SENTINEL {
            &[]
        } else {
            &v[4..length as usize + 4]
        };

        Self::new(ParameterId(parameter_id), value.to_vec())
    }
}

impl FromBytes for ParameterList {
    fn from_bytes<E: byteorder::ByteOrder>(mut v: &[u8]) -> Self {
        const MAX_PARAMETERS: usize = 2_usize.pow(16);

        let mut parameter = vec![];
        for _ in 0..MAX_PARAMETERS {
            let parameter_i = Parameter::from_bytes::<E>(v);
            if parameter_i.parameter_id() == ParameterId(PID_SENTINEL) {
                break;
            } else {
                v.consume(parameter_i.length() as usize + 4);
                parameter.push(parameter_i);
            }
        }
        Self::new(parameter)
    }
}

impl WriteBytes for Parameter {
    fn write_bytes(&self, mut buf: &mut [u8]) -> usize {
        let padding_len = match self.value().len() % 4 {
            1 => 3,
            2 => 2,
            3 => 1,
            _ => 0,
        };
        let length = self.value().len() + padding_len;
        let parameter_id = <u16>::from(self.parameter_id());
        parameter_id.write_bytes(&mut buf[0..]);
        (length as i16).write_bytes(&mut buf[2..]);
        buf = &mut buf[4..];
        buf[..self.value().len()].copy_from_slice(self.value().as_ref());
        buf[self.value().len()..length].fill(0);
        4 + length
    }
}

impl WriteBytes for &ParameterList {
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
        let mut length = 0;
        for parameter in self.parameter().iter() {
            length += parameter.write_bytes(&mut buf[length..]);
        }
        PID_SENTINEL.write_bytes(&mut buf[length..]);
        buf[length + 2..length + 4].fill(0);
        length + 4
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Data(Vec<u8>);

impl Data {
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl AsRef<[u8]> for Data {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl WriteBytes for &Data {
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
        buf[..self.0.len()].copy_from_slice(&self.0);
        let length_inclusive_padding = (self.0.len() + 3) & !3;
        buf[self.0.len()..length_inclusive_padding].fill(0);
        length_inclusive_padding
    }
}

impl FromBytes for Data {
    fn from_bytes<E: byteorder::ByteOrder>(v: &[u8]) -> Self {
        Self::new(v.to_vec())
    }
}

impl FromBytes for EntityId {
    fn from_bytes<E: byteorder::ByteOrder>(v: &[u8]) -> Self {
        Self::new([v[0], v[1], v[2]], v[3])
    }
}

impl FromBytes for GuidPrefix {
    fn from_bytes<E: byteorder::ByteOrder>(v: &[u8]) -> Self {
        [
            v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], v[8], v[9], v[10], v[11],
        ]
    }
}

impl FromBytes for SequenceNumber {
    fn from_bytes<E: byteorder::ByteOrder>(v: &[u8]) -> Self {
        let high = E::read_i32(&v[0..]);
        let low = E::read_i32(&v[4..]);
        let value = ((high as i64) << 32) + low as i64;
        SequenceNumber::from(value)
    }
}

impl FromBytes for Count {
    fn from_bytes<E: byteorder::ByteOrder>(v: &[u8]) -> Self {
        Self::new(E::read_i32(v))
    }
}

impl FromBytes for SequenceNumberSet {
    fn from_bytes<E: byteorder::ByteOrder>(v: &[u8]) -> Self {
        let high = E::read_i32(&v[0..]);
        let low = E::read_i32(&v[4..]);
        let base = ((high as i64) << 32) + low as i64;

        let num_bits = E::read_u32(&v[8..]);
        let number_of_bitmap_elements = ((num_bits + 31) / 32) as usize; //In standard referred to as "M"
        let mut bitmap = [0; 8];
        let mut buf = &v[12..];
        for bitmap_i in bitmap.iter_mut().take(number_of_bitmap_elements) {
            *bitmap_i = E::read_i32(buf);
            buf.consume(4);
        }

        let mut set = Vec::with_capacity(256);
        for delta_n in 0..num_bits as usize {
            if (bitmap[delta_n / 32] & (1 << (31 - delta_n % 32))) == (1 << (31 - delta_n % 32)) {
                set.push(SequenceNumber::from(base + delta_n as i64));
            }
        }
        SequenceNumberSet::new(SequenceNumber::from(base), set)
    }
}

impl FromBytes for u16 {
    fn from_bytes<E: byteorder::ByteOrder>(v: &[u8]) -> Self {
        E::read_u16(v)
    }
}

impl FromBytes for i16 {
    fn from_bytes<E: byteorder::ByteOrder>(v: &[u8]) -> Self {
        E::read_i16(v)
    }
}

impl FromBytes for u32 {
    fn from_bytes<E: byteorder::ByteOrder>(v: &[u8]) -> Self {
        E::read_u32(v)
    }
}

impl FromBytes for FragmentNumber {
    fn from_bytes<E: byteorder::ByteOrder>(v: &[u8]) -> Self {
        Self::new(E::read_u32(v))
    }
}

impl FromBytes for Locator {
    fn from_bytes<E: byteorder::ByteOrder>(v: &[u8]) -> Self {
        let kind = E::read_i32(&v[0..]);
        let port = E::read_u32(&v[4..]);
        let address = LocatorAddress::new([
            v[8], v[9], v[10], v[11], v[12], v[13], v[14], v[15], v[16], v[17], v[18], v[19],
            v[20], v[21], v[22], v[23],
        ]);
        Self::new(kind, port, address)
    }
}

impl FromBytes for LocatorList {
    fn from_bytes<E: byteorder::ByteOrder>(v: &[u8]) -> Self {
        let num_locators = E::read_u32(v);
        let mut buf = &v[4..];
        let mut locator_list = Vec::new();
        for _ in 0..num_locators {
            locator_list.push(Locator::from_bytes::<E>(buf));
            buf.consume(24)
        }
        Self::new(locator_list)
    }
}

impl FromBytes for ProtocolVersion {
    fn from_bytes<E: byteorder::ByteOrder>(v: &[u8]) -> Self {
        Self::new(v[0], v[1])
    }
}

impl FromBytes for VendorId {
    fn from_bytes<E: byteorder::ByteOrder>(v: &[u8]) -> Self {
        Self::new([v[0], v[1]])
    }
}

impl FromBytes for Time {
    fn from_bytes<E: byteorder::ByteOrder>(v: &[u8]) -> Self {
        let seconds = E::read_i32(&v[0..]);
        let fractions = E::read_u32(&v[4..]);
        Self::new(seconds, fractions)
    }
}

impl FromBytes for FragmentNumberSet {
    fn from_bytes<E: byteorder::ByteOrder>(v: &[u8]) -> Self {
        let base = E::read_u32(&v[0..]);
        let num_bits = E::read_u32(&v[4..]);
        let number_of_bitmap_elements = ((num_bits + 31) / 32) as usize; //In standard referred to as "M"
        let mut bitmap = [0; 8];
        let mut buf = &v[8..];
        for bitmap_i in bitmap.iter_mut().take(number_of_bitmap_elements) {
            *bitmap_i = E::read_i32(buf);
            buf.consume(4);
        }

        let mut set = Vec::with_capacity(256);
        for delta_n in 0..num_bits as usize {
            if (bitmap[delta_n / 32] & (1 << (31 - delta_n % 32))) == (1 << (31 - delta_n % 32)) {
                set.push(FragmentNumber::new(base + delta_n as u32));
            }
        }
        Self::new(FragmentNumber::new(base), set)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::rtps::{
        messages::overall_structure::into_bytes_vec,
        types::{Locator, LocatorAddress},
    };

    #[test]
    fn serialize_fragment_number_max_gap() {
        let fragment_number_set = FragmentNumberSet {
            base: FragmentNumber::new(2),
            set: vec![FragmentNumber::new(2), FragmentNumber::new(257)],
        };
        #[rustfmt::skip]
        assert_eq!(into_bytes_vec(fragment_number_set), vec![
            2, 0, 0, 0, // bitmapBase: (unsigned long)
            0, 1, 0, 0, // numBits (unsigned long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_1000_0000, // bitmap[0] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[1] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[2] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[3] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[4] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[5] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[6] (long)
            0b000_0001, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[7] (long)
        ]);
    }

    #[test]
    fn serialize_locator_list() {
        let locator_1 = Locator::new(1, 2, LocatorAddress::new([3; 16]));
        let locator_2 = Locator::new(2, 2, LocatorAddress::new([3; 16]));
        let locator_list = LocatorList::new(vec![locator_1, locator_2]);
        assert_eq!(
            into_bytes_vec(locator_list),
            vec![
                2, 0, 0, 0, // numLocators (unsigned long)
                1, 0, 0, 0, // kind (long)
                2, 0, 0, 0, // port (unsigned long)
                3, 3, 3, 3, // address (octet[16])
                3, 3, 3, 3, // address (octet[16])
                3, 3, 3, 3, // address (octet[16])
                3, 3, 3, 3, // address (octet[16])
                2, 0, 0, 0, // kind (long)
                2, 0, 0, 0, // port (unsigned long)
                3, 3, 3, 3, // address (octet[16])
                3, 3, 3, 3, // address (octet[16])
                3, 3, 3, 3, // address (octet[16])
                3, 3, 3, 3, // address (octet[16])
            ]
        );
    }

    #[test]
    fn deserialize_count() {
        let expected = Count::new(7);
        assert_eq!(
            expected,
            Count::from_bytes::<byteorder::LittleEndian>(&[
            7, 0, 0,0 , //value (long)
        ])
        );
    }

    #[test]
    fn deserialize_locator_list() {
        let locator_1 = Locator::new(1, 2, LocatorAddress::new([3; 16]));
        let locator_2 = Locator::new(2, 2, LocatorAddress::new([3; 16]));
        let expected = LocatorList::new(vec![locator_1, locator_2]);
        #[rustfmt::skip]
        let result = LocatorList::from_bytes::<byteorder::LittleEndian>(&[
            2, 0, 0, 0,  // numLocators (unsigned long)
            1, 0, 0, 0, // kind (long)
            2, 0, 0, 0, // port (unsigned long)
            3, 3, 3, 3, // address (octet[16])
            3, 3, 3, 3, // address (octet[16])
            3, 3, 3, 3, // address (octet[16])
            3, 3, 3, 3, // address (octet[16])
            2, 0, 0, 0, // kind (long)
            2, 0, 0, 0, // port (unsigned long)
            3, 3, 3, 3, // address (octet[16])
            3, 3, 3, 3, // address (octet[16])
            3, 3, 3, 3, // address (octet[16])
            3, 3, 3, 3, // address (octet[16])

        ]);
        assert_eq!(expected, result);
    }

    #[test]
    fn deserialize_fragment_number() {
        let expected = FragmentNumber::new(7);
        assert_eq!(
            expected,
            FragmentNumber::from_bytes::<byteorder::LittleEndian>(&[
                7, 0, 0, 0, // (unsigned long)
            ])
        );
    }

    #[test]
    fn deserialize_fragment_number_set_max_gap() {
        let expected = FragmentNumberSet {
            base: FragmentNumber::new(2),
            set: vec![FragmentNumber::new(2), FragmentNumber::new(257)],
        };
        #[rustfmt::skip]
        let result = FragmentNumberSet::from_bytes::<byteorder::LittleEndian>(&[
            2, 0, 0, 0, // bitmapBase: (unsigned long)
            0, 1, 0, 0, // numBits (unsigned long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_1000_0000, // bitmap[0] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[1] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[2] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[3] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[4] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[5] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[6] (long)
            0b000_0001, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[7] (long)

        ]);
        assert_eq!(expected, result);
    }

    #[test]
    fn deserialize_guid_prefix() {
        let expected = [1; 12];
        #[rustfmt::skip]
        assert_eq!(expected, GuidPrefix::from_bytes::<byteorder::LittleEndian>(&[
            1, 1, 1, 1,
            1, 1, 1, 1,
            1, 1, 1, 1,
        ]));
    }

    #[test]
    fn deserialize_protocol_version() {
        let expected = ProtocolVersion::new(2, 3);
        assert_eq!(
            expected,
            ProtocolVersion::from_bytes::<byteorder::LittleEndian>(&[2, 3])
        );
    }

    #[test]
    fn deserialize_vendor_id() {
        let expected = VendorId::new([1, 2]);
        assert_eq!(
            expected,
            VendorId::from_bytes::<byteorder::LittleEndian>(&[1, 2,])
        );
    }

    #[test]
    fn deserialize_sequence_number() {
        let expected = SequenceNumber::from(7);
        assert_eq!(
            expected,
            SequenceNumber::from_bytes::<byteorder::LittleEndian>(&[
                0, 0, 0, 0, // high (long)
                7, 0, 0, 0, // low (unsigned long)
            ])
        );
    }

    #[test]
    fn serialize_sequence_number_max_gap() {
        let sequence_number_set = SequenceNumberSet {
            base: SequenceNumber::from(2),
            set: vec![SequenceNumber::from(2), SequenceNumber::from(257)],
        };
        #[rustfmt::skip]
        assert_eq!(into_bytes_vec(sequence_number_set), vec![
            0, 0, 0, 0, // bitmapBase: high (long)
            2, 0, 0, 0, // bitmapBase: low (unsigned long)
            0, 1, 0, 0, // numBits (unsigned long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_1000_0000, // bitmap[0] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[1] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[2] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[3] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[4] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[5] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[6] (long)
            0b000_0001, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[7] (long)
        ]);
    }

    #[test]
    fn deserialize_sequence_number_set_max_gap() {
        let expected = SequenceNumberSet {
            base: SequenceNumber::from(2),
            set: vec![SequenceNumber::from(2), SequenceNumber::from(257)],
        };
        #[rustfmt::skip]
        let result = SequenceNumberSet::from_bytes::<byteorder::LittleEndian>(&[
            0, 0, 0, 0, // bitmapBase: high (long)
            2, 0, 0, 0, // bitmapBase: low (unsigned long)
            0, 1, 0, 0, // numBits (unsigned long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_1000_0000, // bitmap[0] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[1] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[2] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[3] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[4] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[5] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[6] (long)
            0b000_0001, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[7] (long)
        ]);
        assert_eq!(expected, result);
    }

    #[test]
    fn serialize_parameter() {
        let parameter = Parameter::new(ParameterId(2), vec![5, 6, 7, 8]);
        #[rustfmt::skip]
        assert_eq!(into_bytes_vec(parameter), vec![
            0x02, 0x00, 4, 0, // Parameter | length
            5, 6, 7, 8,       // value
        ]);
    }

    #[test]
    fn serialize_parameter_non_multiple_4() {
        let parameter = Parameter::new(ParameterId(2), vec![5, 6, 7]);
        #[rustfmt::skip]
        assert_eq!(into_bytes_vec(parameter), vec![
            0x02, 0x00, 4, 0, // Parameter | length
            5, 6, 7, 0,       // value
        ]);
    }

    #[test]
    fn serialize_parameter_zero_size() {
        let parameter = Parameter::new(ParameterId(2), vec![]);
        assert_eq!(
            into_bytes_vec(parameter),
            vec![
            0x02, 0x00, 0, 0, // Parameter | length
        ]
        );
    }

    #[test]
    fn serialize_parameter_list() {
        let parameter_1 = Parameter::new(ParameterId(2), vec![51, 61, 71, 81]);
        let parameter_2 = Parameter::new(ParameterId(3), vec![52, 62, 0, 0]);
        let parameter_list_submessage_element = &ParameterList::new(vec![parameter_1, parameter_2]);
        #[rustfmt::skip]
        assert_eq!(into_bytes_vec(parameter_list_submessage_element), vec![
            0x02, 0x00, 4, 0, // Parameter ID | length
            51, 61, 71, 81,   // value
            0x03, 0x00, 4, 0, // Parameter ID | length
            52, 62, 0, 0,   // value
            0x01, 0x00, 0, 0, // Sentinel: PID_SENTINEL | PID_PAD
        ]);
    }

    #[test]
    fn serialize_parameter_list_empty() {
        let parameter = &ParameterList::empty();
        #[rustfmt::skip]
        assert_eq!(into_bytes_vec(parameter), vec![
            0x01, 0x00, 0, 0, // Sentinel: PID_SENTINEL | PID_PAD
        ]);
    }

    #[test]
    fn deserialize_parameter_non_multiple_of_4() {
        let expected = Parameter::new(ParameterId(2), vec![5, 6, 7, 8, 9, 10, 11, 0]);
        #[rustfmt::skip]
        let result = Parameter::from_bytes::<byteorder::LittleEndian>(&[
            0x02, 0x00, 8, 0, // Parameter | length
            5, 6, 7, 8,       // value
            9, 10, 11, 0,     // value
        ]);
        assert_eq!(expected, result);
    }

    #[test]
    fn deserialize_parameter() {
        let expected = Parameter::new(ParameterId(2), vec![5, 6, 7, 8, 9, 10, 11, 12]);
        #[rustfmt::skip]
        let result = Parameter::from_bytes::<byteorder::LittleEndian>(&[
            0x02, 0x00, 8, 0, // Parameter | length
            5, 6, 7, 8,       // value
            9, 10, 11, 12,       // value
        ]);
        assert_eq!(expected, result);
    }

    #[test]
    fn deserialize_parameter_list() {
        let expected = ParameterList::new(vec![
            Parameter::new(ParameterId(2), vec![15, 16, 17, 18]),
            Parameter::new(ParameterId(3), vec![25, 26, 27, 28]),
        ]);
        #[rustfmt::skip]
        let result = ParameterList::from_bytes::<byteorder::LittleEndian>(&[
            0x02, 0x00, 4, 0, // Parameter ID | length
            15, 16, 17, 18,        // value
            0x03, 0x00, 4, 0, // Parameter ID | length
            25, 26, 27, 28,        // value
            0x01, 0x00, 0, 0, // Sentinel: Parameter ID | length
        ]);
        assert_eq!(expected, result);
    }

    #[test]
    fn deserialize_parameter_list_with_long_parameter_including_sentinel() {
        #[rustfmt::skip]
        let parameter_value_expected = vec![
            0x01, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00,
            0x01, 0x01, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01,
        ];

        let expected = ParameterList::new(vec![Parameter::new(
            ParameterId(0x32),
            parameter_value_expected,
        )]);
        #[rustfmt::skip]
        let result = ParameterList::from_bytes::<byteorder::LittleEndian>(&[
            0x32, 0x00, 24, 0x00, // Parameter ID | length
            0x01, 0x00, 0x00, 0x00, // Parameter value
            0x01, 0x00, 0x00, 0x00, // Parameter value
            0x01, 0x01, 0x01, 0x01, // Parameter value
            0x01, 0x01, 0x01, 0x01, // Parameter value
            0x01, 0x01, 0x01, 0x01, // Parameter value
            0x01, 0x01, 0x01, 0x01, // Parameter value
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, Length: 0
        ]);
        assert_eq!(expected, result);
    }

    #[test]
    fn deserialize_parameter_list_with_multiple_parameters_with_same_id() {
        #[rustfmt::skip]
        let parameter_value_expected1 = vec![
            0x01, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00,
            0x01, 0x01, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01,
        ];
        #[rustfmt::skip]
        let parameter_value_expected2 = vec![
            0x01, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00,
            0x02, 0x02, 0x02, 0x02,
            0x02, 0x02, 0x02, 0x02,
            0x02, 0x02, 0x02, 0x02,
            0x02, 0x02, 0x02, 0x02,
        ];

        let expected = ParameterList::new(vec![
            Parameter::new(ParameterId(0x32), parameter_value_expected1),
            Parameter::new(ParameterId(0x32), parameter_value_expected2),
        ]);
        #[rustfmt::skip]
        let result = ParameterList::from_bytes::<byteorder::LittleEndian>(&[
            0x32, 0x00, 24, 0x00, // Parameter ID | length
            0x01, 0x00, 0x00, 0x00, // Parameter value
            0x01, 0x00, 0x00, 0x00, // Parameter value
            0x01, 0x01, 0x01, 0x01, // Parameter value
            0x01, 0x01, 0x01, 0x01, // Parameter value
            0x01, 0x01, 0x01, 0x01, // Parameter value
            0x01, 0x01, 0x01, 0x01, // Parameter value
            0x32, 0x00, 24, 0x00, // Parameter ID | length
            0x01, 0x00, 0x00, 0x00, // Parameter value
            0x01, 0x00, 0x00, 0x00, // Parameter value
            0x02, 0x02, 0x02, 0x02, // Parameter value
            0x02, 0x02, 0x02, 0x02, // Parameter value
            0x02, 0x02, 0x02, 0x02, // Parameter value
            0x02, 0x02, 0x02, 0x02, // Parameter value
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, Length: 0
        ]);
        assert_eq!(expected, result);
    }
}
