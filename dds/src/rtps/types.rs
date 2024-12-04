use super::{
    error::RtpsResult,
    messages::overall_structure::{Endianness, TryReadFromBytes, WriteIntoBytes},
};
use crate::{
    infrastructure::qos_policy::{
        DurabilityQosPolicy, DurabilityQosPolicyKind, ReliabilityQosPolicy,
        ReliabilityQosPolicyKind,
    },
    transport::types::{
        DurabilityKind, EntityId, GuidPrefix, Locator, Long, Octet, ProtocolVersion,
        ReliabilityKind, SequenceNumber, UnsignedLong, VendorId,
    },
};

use std::io::{Read, Write};

///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.5
/// Table 8.2 - Types of the attributes that appear in the RTPS Entities and Classes
///

impl WriteIntoBytes for Octet {
    fn write_into_bytes(&self, buf: &mut dyn Write) {
        buf.write_all(&[*self]).expect("buffer big enough");
    }
}

impl WriteIntoBytes for Long {
    fn write_into_bytes(&self, buf: &mut dyn Write) {
        buf.write_all(self.to_le_bytes().as_slice())
            .expect("buffer big enough");
    }
}

impl WriteIntoBytes for UnsignedLong {
    fn write_into_bytes(&self, buf: &mut dyn Write) {
        buf.write_all(self.to_le_bytes().as_slice())
            .expect("buffer big enough");
    }
}

impl WriteIntoBytes for u16 {
    fn write_into_bytes(&self, buf: &mut dyn Write) {
        buf.write_all(self.to_le_bytes().as_slice())
            .expect("buffer big enough");
    }
}

impl WriteIntoBytes for i16 {
    fn write_into_bytes(&self, buf: &mut dyn Write) {
        buf.write_all(self.to_le_bytes().as_slice())
            .expect("buffer big enough");
    }
}

impl<const N: usize> WriteIntoBytes for [Octet; N] {
    fn write_into_bytes(&self, buf: &mut dyn Write) {
        buf.write_all(self).expect("buffer big enough");
    }
}

impl WriteIntoBytes for &[u8] {
    fn write_into_bytes(&self, buf: &mut dyn Write) {
        buf.write_all(self).expect("buffer big enough");
    }
}

impl TryReadFromBytes for GuidPrefix {
    fn try_read_from_bytes(data: &mut &[u8], _endianness: &Endianness) -> RtpsResult<Self> {
        let mut guid_prefix = [0; 12];
        data.read_exact(&mut guid_prefix)?;
        Ok(guid_prefix)
    }
}

impl TryReadFromBytes for EntityId {
    fn try_read_from_bytes(data: &mut &[u8], _endianness: &Endianness) -> RtpsResult<Self> {
        let mut entity_key = [0; 3];
        let mut entity_kind = [0; 1];
        data.read_exact(&mut entity_key)?;
        data.read_exact(&mut entity_kind)?;
        Ok(EntityId::new(entity_key, entity_kind[0]))
    }
}

impl WriteIntoBytes for EntityId {
    fn write_into_bytes(&self, buf: &mut dyn Write) {
        self.entity_key().write_into_bytes(buf);
        self.entity_kind().write_into_bytes(buf);
    }
}

impl TryReadFromBytes for SequenceNumber {
    fn try_read_from_bytes(data: &mut &[u8], endianness: &Endianness) -> RtpsResult<Self> {
        let high = i32::try_read_from_bytes(data, endianness)?;
        let low = u32::try_read_from_bytes(data, endianness)?;
        let value = ((high as i64) << 32) + low as i64;
        Ok(value)
    }
}

impl WriteIntoBytes for SequenceNumber {
    fn write_into_bytes(&self, buf: &mut dyn Write) {
        let high = (*self >> 32) as Long;
        let low = *self as UnsignedLong;
        high.write_into_bytes(buf);
        low.write_into_bytes(buf);
    }
}

impl WriteIntoBytes for Locator {
    fn write_into_bytes(&self, buf: &mut dyn Write) {
        self.kind().write_into_bytes(buf);
        self.port().write_into_bytes(buf);
        self.address().write_into_bytes(buf);
    }
}

impl TryReadFromBytes for Locator {
    fn try_read_from_bytes(data: &mut &[u8], endianness: &Endianness) -> RtpsResult<Self> {
        let kind = i32::try_read_from_bytes(data, endianness)?;
        let port = u32::try_read_from_bytes(data, endianness)?;
        let mut address = [0; 16];
        data.read_exact(&mut address)?;
        Ok(Self::new(kind, port, address))
    }
}

impl From<&ReliabilityQosPolicy> for ReliabilityKind {
    fn from(value: &ReliabilityQosPolicy) -> Self {
        match value.kind {
            ReliabilityQosPolicyKind::BestEffort => ReliabilityKind::BestEffort,
            ReliabilityQosPolicyKind::Reliable => ReliabilityKind::Reliable,
        }
    }
}

impl From<&DurabilityQosPolicy> for DurabilityKind {
    fn from(value: &DurabilityQosPolicy) -> Self {
        match value.kind {
            DurabilityQosPolicyKind::Volatile => DurabilityKind::Volatile,
            DurabilityQosPolicyKind::TransientLocal => DurabilityKind::TransientLocal,
            DurabilityQosPolicyKind::Transient => DurabilityKind::Transient,
            DurabilityQosPolicyKind::Persistent => DurabilityKind::Persistent,
        }
    }
}

/// InstanceHandle_t
/// Type used to represent the identity of a data-object whose changes in value are communicated by the RTPS protocol.
// Defined elsewhere in DDS

impl TryReadFromBytes for ProtocolVersion {
    fn try_read_from_bytes(data: &mut &[u8], _endianness: &Endianness) -> RtpsResult<Self> {
        let mut bytes = [0; 2];
        data.read_exact(&mut bytes)?;
        Ok(Self::new(bytes[0], bytes[1]))
    }
}

impl WriteIntoBytes for ProtocolVersion {
    fn write_into_bytes(&self, buf: &mut dyn Write) {
        self._major().write_into_bytes(buf);
        self._minor().write_into_bytes(buf);
    }
}

pub const PROTOCOLVERSION: ProtocolVersion = PROTOCOLVERSION_2_4;
#[allow(dead_code)]
pub const PROTOCOLVERSION_1_0: ProtocolVersion = ProtocolVersion::new(1, 0);
#[allow(dead_code)]
pub const PROTOCOLVERSION_1_1: ProtocolVersion = ProtocolVersion::new(1, 1);
#[allow(dead_code)]
pub const PROTOCOLVERSION_2_0: ProtocolVersion = ProtocolVersion::new(2, 0);
#[allow(dead_code)]
pub const PROTOCOLVERSION_2_1: ProtocolVersion = ProtocolVersion::new(2, 1);
#[allow(dead_code)]
pub const PROTOCOLVERSION_2_2: ProtocolVersion = ProtocolVersion::new(2, 2);
#[allow(dead_code)]
pub const PROTOCOLVERSION_2_3: ProtocolVersion = ProtocolVersion::new(2, 3);
pub const PROTOCOLVERSION_2_4: ProtocolVersion = ProtocolVersion::new(2, 4);

impl TryReadFromBytes for VendorId {
    fn try_read_from_bytes(data: &mut &[u8], _endianness: &Endianness) -> RtpsResult<Self> {
        let mut bytes = [0; 2];
        data.read_exact(&mut bytes)?;
        Ok(bytes)
    }
}

#[allow(dead_code)]
pub const VENDOR_ID_UNKNOWN: VendorId = [0, 0];
pub const VENDOR_ID_S2E: VendorId = [0x01, 0x14];

#[cfg(test)]
mod tests {
    use crate::rtps::messages::overall_structure::write_into_bytes_vec;

    use super::*;

    #[test]
    fn deserialize_u16() {
        let mut data = &[7, 0, 123][..];
        let result = u16::try_read_from_bytes(&mut data, &Endianness::LittleEndian).unwrap();
        assert_eq!(result, 7);
        assert_eq!(data, &[123]);
    }

    #[test]
    fn serialize_sequence_number() {
        let data: SequenceNumber = 7;
        let result = write_into_bytes_vec(data);
        assert_eq!(
            result,
            vec![
                0, 0, 0, 0, // high (long)
                7, 0, 0, 0, // low (unsigned long)
            ]
        );
    }

    #[test]
    fn serialize_entity_id() {
        let data = EntityId::new([1, 2, 3], 0x04);
        assert_eq!(
            write_into_bytes_vec(data),
            vec![
            1, 2, 3, 0x04, //value (long)
        ]
        );
    }
}
