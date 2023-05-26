use std::io::BufRead;

use crate::implementation::{
    data_representation_builtin_endpoints::parameter_id_values::PID_SENTINEL,
    rtps::{
        messages::types::FragmentNumber,
        types::{
            Count, EntityId, EntityKey, EntityKind, GuidPrefix, Locator, LocatorAddress,
            LocatorKind, LocatorPort, ProtocolVersion, SequenceNumber, VendorId,
        },
    },
};

use super::{
    types::{ParameterId, SerializedPayload, Time},
    FromBytes,
};

///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// 8.3.5 RTPS SubmessageElements
///

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SequenceNumberSet {
    pub base: SequenceNumber,
    pub set: Vec<SequenceNumber>,
}

impl SequenceNumberSet {
    pub fn new(base: SequenceNumber, set: Vec<SequenceNumber>) -> Self {
        Self { base, set }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FragmentNumberSet {
    pub base: FragmentNumber,
    pub set: Vec<FragmentNumber>,
}

impl FragmentNumberSet {
    pub fn new(base: FragmentNumber, set: Vec<FragmentNumber>) -> Self {
        Self { base, set }
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

impl<'a> FromBytes<'a> for Parameter {
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

impl<'a> FromBytes<'a> for ParameterList {
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

impl<'a> FromBytes<'a> for SerializedPayload<'a> {
    fn from_bytes<E: byteorder::ByteOrder>(v: &'a [u8]) -> Self {
        Self::new(v)
    }
}

impl FromBytes<'_> for EntityId {
    fn from_bytes<E: byteorder::ByteOrder>(v: &[u8]) -> Self {
        Self::new(EntityKey::new([v[0], v[1], v[2]]), EntityKind::new(v[3]))
    }
}

impl FromBytes<'_> for GuidPrefix {
    fn from_bytes<E: byteorder::ByteOrder>(v: &[u8]) -> Self {
        Self::new([
            v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], v[8], v[9], v[10], v[11],
        ])
    }
}

impl FromBytes<'_> for SequenceNumber {
    fn from_bytes<E: byteorder::ByteOrder>(v: &[u8]) -> Self {
        let high = E::read_i32(&v[0..]);
        let low = E::read_i32(&v[4..]);
        let value = ((high as i64) << 32) + low as i64;
        SequenceNumber::new(value)
    }
}

impl FromBytes<'_> for Count {
    fn from_bytes<E: byteorder::ByteOrder>(v: &[u8]) -> Self {
        Self::new(E::read_i32(v))
    }
}

impl FromBytes<'_> for SequenceNumberSet {
    fn from_bytes<E: byteorder::ByteOrder>(v: &[u8]) -> Self {
        let high = E::read_i32(&v[0..]);
        let low = E::read_i32(&v[4..]);
        let base = ((high as i64) << 32) + low as i64;

        let num_bits = E::read_u32(&v[8..]);
        let number_of_bitmap_elements = ((num_bits + 31) / 32) as usize; //In standard refered to as "M"
        let mut bitmap = [0; 8];
        let mut buf = &v[12..];
        for bitmap_i in bitmap.iter_mut().take(number_of_bitmap_elements) {
            *bitmap_i = E::read_i32(buf);
            buf.consume(4);
        }

        let mut set = Vec::with_capacity(256);
        for delta_n in 0..num_bits as usize {
            if (bitmap[delta_n / 32] & (1 << (31 - delta_n % 32))) == (1 << (31 - delta_n % 32)) {
                set.push(SequenceNumber::new(base + delta_n as i64));
            }
        }
        SequenceNumberSet::new(SequenceNumber::new(base), set)
    }
}

impl FromBytes<'_> for u16 {
    fn from_bytes<E: byteorder::ByteOrder>(v: &[u8]) -> Self {
        E::read_u16(v)
    }
}

impl FromBytes<'_> for i16 {
    fn from_bytes<E: byteorder::ByteOrder>(v: &[u8]) -> Self {
        E::read_i16(v)
    }
}

impl FromBytes<'_> for u32 {
    fn from_bytes<E: byteorder::ByteOrder>(v: &[u8]) -> Self {
        E::read_u32(v)
    }
}

impl FromBytes<'_> for FragmentNumber {
    fn from_bytes<E: byteorder::ByteOrder>(v: &[u8]) -> Self {
        Self::new(E::read_u32(v))
    }
}

impl FromBytes<'_> for Locator {
    fn from_bytes<E: byteorder::ByteOrder>(v: &[u8]) -> Self {
        let kind = LocatorKind::new(E::read_i32(&v[0..]));
        let port = LocatorPort::new(E::read_u32(&v[4..]));
        let address = LocatorAddress::new([
            v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], v[8], v[9], v[10], v[11], v[12], v[13],
            v[14], v[15],
        ]);
        Self::new(kind, port, address)
    }
}

impl FromBytes<'_> for LocatorList {
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

impl FromBytes<'_> for ProtocolVersion {
    fn from_bytes<E: byteorder::ByteOrder>(v: &[u8]) -> Self {
        Self::new(v[0], v[1])
    }
}

impl FromBytes<'_> for VendorId {
    fn from_bytes<E: byteorder::ByteOrder>(v: &[u8]) -> Self {
        Self::new([v[0], v[1]])
    }
}

impl FromBytes<'_> for Time {
    fn from_bytes<E: byteorder::ByteOrder>(v: &[u8]) -> Self {
        let seconds = E::read_i32(&v[0..]);
        let fractions = E::read_u32(&v[4..]);
        Self::new(seconds, fractions)
    }
}

impl FromBytes<'_> for FragmentNumberSet {
    fn from_bytes<E: byteorder::ByteOrder>(v: &[u8]) -> Self {
        let base = E::read_u32(&v[0..]);
        let num_bits = E::read_u32(&v[4..]);
        let number_of_bitmap_elements = ((num_bits + 31) / 32) as usize; //In standard refered to as "M"
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
