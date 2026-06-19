use crate::{
    dcps::data_representation_builtin_endpoints::{
        parameter_id_values::ParameterId,
        spdp_discovered_participant_data::{BuiltinEndpointQos, BuiltinEndpointSet},
    },
    infrastructure::time::Duration,
    transport::types::{EntityId, Locator, Long, Octet, ProtocolVersion, UnsignedLong},
};
use alloc::{string::String, vec::Vec};

pub(crate) struct ParameterListSerializer<'a> {
    data: &'a mut Vec<u8>,
}

impl<'a> ParameterListSerializer<'a> {
    pub(crate) fn new(data: &'a mut Vec<u8>) -> Self {
        Self { data }
    }
    pub(crate) fn write_header(&mut self) {
        let mut ser = CdrSerializer::new(self.data);
        [0, 3].cdr_serialize(&mut ser);
        [0, 0].cdr_serialize(&mut ser);
    }
    pub(crate) fn write_sentinel(self) {
        let mut ser = CdrSerializer::new(self.data);
        1i16.cdr_serialize(&mut ser);
        0u16.cdr_serialize(&mut ser);
    }

    pub(crate) fn write_non_optional_parameter<T: CdrSerialize>(
        &mut self,
        pid: ParameterId,
        value: &T,
    ) {
        const ZEROS: [u8; 4] = [0; 4];
        let mut ser = CdrSerializer::new(self.data);
        pid.cdr_serialize(&mut ser);
        0u16.cdr_serialize(&mut ser);
        let position = ser.buffer.len();
        value.cdr_serialize(&mut ser);
        let position_after_value = ser.buffer.len();
        let alignment = 4;
        let padding_length =
            position_after_value.div_ceil(alignment) * alignment - position_after_value;
        self.data.extend_from_slice(&ZEROS[..padding_length]);
        let length = (self.data.len() - position) as u16;
        self.data[position - 2..position].clone_from_slice(&length.to_le_bytes());
    }
    pub(crate) fn write_optional_parameter<T: CdrSerialize + PartialEq>(
        &mut self,
        pid: ParameterId,
        value: &T,
        default: &T,
    ) {
        if default != value {
            self.write_non_optional_parameter(pid, value);
        }
    }
    pub(crate) fn write_list<T: CdrSerialize>(&mut self, pid: ParameterId, value_list: &[T]) {
        for value in value_list {
            self.write_non_optional_parameter(pid, value);
        }
    }
}

struct CdrSerializer<'a> {
    buffer: &'a mut Vec<u8>,
}

impl<'a> CdrSerializer<'a> {
    fn new(buffer: &'a mut Vec<u8>) -> Self {
        Self { buffer }
    }

    fn pad(&mut self, alignment: usize) {
        const ZEROS: [u8; 4] = [0; 4];
        let position = self.buffer.len();
        let padding_length = position.div_ceil(alignment) * alignment - position;
        self.buffer.extend_from_slice(&ZEROS[..padding_length]);
    }
}

trait CdrSerialize {
    fn cdr_serialize(&self, ser: &mut CdrSerializer);
}

impl CdrSerialize for Octet {
    fn cdr_serialize(&self, ser: &mut CdrSerializer) {
        ser.buffer.push(*self);
    }
}
impl CdrSerialize for i16 {
    fn cdr_serialize(&self, ser: &mut CdrSerializer) {
        ser.pad(2);
        ser.buffer.extend_from_slice(&self.to_le_bytes());
    }
}
impl CdrSerialize for u16 {
    fn cdr_serialize(&self, ser: &mut CdrSerializer) {
        ser.pad(2);
        ser.buffer.extend_from_slice(&self.to_le_bytes());
    }
}
impl CdrSerialize for Long {
    fn cdr_serialize(&self, ser: &mut CdrSerializer) {
        ser.pad(4);
        ser.buffer.extend_from_slice(&self.to_le_bytes());
    }
}
impl CdrSerialize for UnsignedLong {
    fn cdr_serialize(&self, ser: &mut CdrSerializer) {
        ser.pad(4);
        ser.buffer.extend_from_slice(&self.to_le_bytes());
    }
}
impl CdrSerialize for bool {
    fn cdr_serialize(&self, ser: &mut CdrSerializer) {
        ser.buffer.push(if *self { 1 } else { 0 });
    }
}
impl<const N: usize> CdrSerialize for [u8; N] {
    fn cdr_serialize(&self, ser: &mut CdrSerializer) {
        ser.buffer.extend_from_slice(self);
    }
}
impl CdrSerialize for String {
    fn cdr_serialize(&self, ser: &mut CdrSerializer) {
        let length = (self.len() + 1) as u32;
        length.cdr_serialize(ser);
        ser.buffer.extend_from_slice(self.as_bytes());
        ser.buffer.push(0);
    }
}
impl CdrSerialize for Locator {
    fn cdr_serialize(&self, ser: &mut CdrSerializer) {
        self.kind.cdr_serialize(ser);
        self.port.cdr_serialize(ser);
        self.address.cdr_serialize(ser);
    }
}
impl CdrSerialize for ProtocolVersion {
    fn cdr_serialize(&self, ser: &mut CdrSerializer) {
        self.bytes.cdr_serialize(ser);
    }
}
impl CdrSerialize for BuiltinEndpointQos {
    fn cdr_serialize(&self, ser: &mut CdrSerializer) {
        self.0.cdr_serialize(ser);
    }
}
impl CdrSerialize for BuiltinEndpointSet {
    fn cdr_serialize(&self, ser: &mut CdrSerializer) {
        self.0.cdr_serialize(ser);
    }
}
impl CdrSerialize for Duration {
    fn cdr_serialize(&self, ser: &mut CdrSerializer) {
        self.sec.cdr_serialize(ser);
        self.nanosec.cdr_serialize(ser);
    }
}
impl CdrSerialize for EntityId {
    fn cdr_serialize(&self, ser: &mut CdrSerializer) {
        self.entity_key.cdr_serialize(ser);
        self.entity_kind.cdr_serialize(ser);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct TestDiscoveryData {
        domain_id: i32,
        domain_tag: String,
        vendor_id: [u8; 2],
        locator_list: Vec<Locator>,
    }

    impl TestDiscoveryData {
        fn to_bytes(&self) -> Vec<u8> {
            let mut buffer = Vec::new();
            let mut pl = ParameterListSerializer::new(&mut buffer);
            pl.write_header();
            pl.write_optional_parameter(15, &self.domain_id, &0);
            pl.write_optional_parameter(0x4014, &self.domain_tag, &String::from(""));
            pl.write_non_optional_parameter(0x16, &self.vendor_id);
            pl.write_list(72, &self.locator_list);
            pl.write_sentinel();
            buffer
        }
    }

    #[test]
    fn serialize_test_discovery_data() {
        let expected = [
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            15, 0x00, 0x04, 0x00, // PID_DOMAIN_ID, Length: 4
            0x01, 0x00, 0x00, 0x00, // DomainId
            0x14, 0x40, 0x08, 0x00, // PID_DOMAIN_TAG, Length: 8
            3, 0x00, 0x00, 0x00, // DomainTag: string length (incl. terminator)
            b'a', b'b', 0, 0x00, // DomainTag: string + padding (1 byte)
            0x16, 0x00, 4, 0x00, // PID_VENDORID
            73, 74, 0x00, 0x00, // VendorId
            72, 0x00, 24, 0x00, // PID_DEFAULT_MULTICAST_LOCATOR
            11, 0x00, 0x00, 0x00, // Locator{kind
            12, 0x00, 0x00, 0x00, // port,
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // address
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // }
            72, 0x00, 24, 0x00, // PID_DEFAULT_MULTICAST_LOCATOR
            21, 0x00, 0x00, 0x00, // Locator{kind
            22, 0x00, 0x00, 0x00, // port,
            0x02, 0x02, 0x02, 0x02, //
            0x02, 0x02, 0x02, 0x02, // address
            0x02, 0x02, 0x02, 0x02, //
            0x02, 0x02, 0x02, 0x02, // }
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL
        ];
        assert_eq!(
            TestDiscoveryData {
                domain_id: 1,
                domain_tag: String::from("ab"),
                vendor_id: [73, 74],
                locator_list: vec![Locator::new(11, 12, [1; 16]), Locator::new(21, 22, [2; 16])],
            }
            .to_bytes(),
            expected.to_vec()
        );
    }
}
