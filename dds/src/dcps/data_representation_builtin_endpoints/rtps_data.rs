use crate::{
    dcps::data_representation_builtin_endpoints::{
        parameter_id_values::ParameterId,
        spdp_discovered_participant_data::{BuiltinEndpointQos, BuiltinEndpointSet},
    },
    infrastructure::{qos_policy::UserDataQosPolicy, time::Duration},
    transport::types::{EntityId, Locator, Long, Octet, ProtocolVersion, UnsignedLong},
    xtypes::{
        deserializer::{
            Align, AsBytes, EncodingVersion1, LittleEndian, NextCdrReader, XTypesDeserializer,
        },
        dynamic_type::DynamicData,
        error::{XTypesError, XTypesResult},
        type_support::{Type, TypeSupport},
    },
};

pub type CdrResult<T> = Result<T, CdrError>;

#[derive(Debug, PartialEq)]
pub enum CdrError {
    InvalidData,
    PidNotFound(i16),
    NotEnoughData,
    XTypes(XTypesError),
}
impl From<XTypesError> for CdrError {
    fn from(value: XTypesError) -> Self {
        CdrError::XTypes(value)
    }
}

pub struct ParameterList<'a, E> {
    data: &'a [u8],
    _endianness: core::marker::PhantomData<E>,
}

impl<'a, E> ParameterList<'a, E> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            _endianness: core::marker::PhantomData,
        }
    }
    pub fn get_list(&self, pid: ParameterId) -> Vec<&'a [u8]> {
        let mut list = Vec::new();
        let mut pointer = &self.data[..];
        loop {
            let current_pid = ParameterId::from_le_bytes([pointer[0], pointer[1]]);
            let length = u16::from_le_bytes([pointer[2], pointer[3]]) as usize;
            if current_pid == pid {
                list.push(&pointer[4..length + 4]);
            } else if current_pid == 1 || pointer.len() < length + 4 {
                return list;
            }
            pointer = &pointer[length + 4..]
        }
    }
    pub fn get_optional(&self, pid: ParameterId) -> Option<&'a [u8]> {
        let mut pointer = &self.data[..];
        loop {
            let current_pid = ParameterId::from_le_bytes([pointer[0], pointer[1]]);
            let length = u16::from_le_bytes([pointer[2], pointer[3]]) as usize;
            if current_pid == pid {
                return Some(&pointer[4..length + 4]);
            } else if current_pid == 1 || pointer.len() < length + 4 {
                return None;
            }
            pointer = &pointer[length + 4..]
        }
    }
    pub fn get_non_optional(&self, pid: ParameterId) -> CdrResult<&'a [u8]> {
        self.get_optional(pid).ok_or(CdrError::PidNotFound(pid))
    }

    pub fn get_optional_parameter<T: CdrDeserialize<'a, E>>(
        &self,
        pid: ParameterId,
        default: T,
    ) -> CdrResult<T> {
        if let Some(pid_data) = self.get_optional(pid) {
            CdrDeserialize::cdr_deserialize(&mut CdrDeserializer::new(pid_data))
        } else {
            Ok(default)
        }
    }
    pub fn get_non_optional_parameter<T: CdrDeserialize<'a, E>>(
        &self,
        pid: ParameterId,
    ) -> CdrResult<T> {
        CdrDeserialize::cdr_deserialize(&mut CdrDeserializer::new(self.get_non_optional(pid)?))
    }
    pub fn get_locator_list(&self, pid: ParameterId) -> CdrResult<Vec<Locator>> {
        let mut locator_list = vec![];
        for pid_data in self.get_list(pid) {
            locator_list.push(CdrDeserialize::cdr_deserialize(
                &mut CdrDeserializer::<E>::new(pid_data),
            )?);
        }
        Ok(locator_list)
    }
}
pub struct CdrDeserializer<'a, E> {
    reader: NextCdrReader<'a, LittleEndian>,
    buffer: &'a [u8],
    pos: usize,
    _endianness: core::marker::PhantomData<E>,
}

pub trait CdrDeserialize<'a, E>: Sized {
    fn cdr_deserialize(de: &mut CdrDeserializer<'a, E>) -> CdrResult<Self>;
}
impl<'a, E> CdrDeserialize<'a, E> for Octet {
    fn cdr_deserialize(de: &mut CdrDeserializer<'a, E>) -> CdrResult<Self> {
        de.deserialize_primitive()
    }
}
impl<'a, E> CdrDeserialize<'a, E> for Long {
    fn cdr_deserialize(de: &mut CdrDeserializer<'a, E>) -> CdrResult<Self> {
        de.reader.seek_padding(4);
        de.deserialize_primitive()
    }
}
impl<'a, E> CdrDeserialize<'a, E> for UnsignedLong {
    fn cdr_deserialize(de: &mut CdrDeserializer<'a, E>) -> CdrResult<Self> {
        de.reader.seek_padding(4);
        de.deserialize_primitive()
    }
}
impl<'a, E> CdrDeserialize<'a, E> for bool {
    fn cdr_deserialize(de: &mut CdrDeserializer<'a, E>) -> CdrResult<Self> {
        de.deserialize_primitive()
    }
}

impl<'a, E, const N: usize> CdrDeserialize<'a, E> for [u8; N] {
    fn cdr_deserialize(de: &mut CdrDeserializer<'a, E>) -> CdrResult<Self> {
        Ok(de.deserialize_bytes(N)?.try_into().expect("must have size"))
    }
}
impl<'a, E> CdrDeserialize<'a, E> for String {
    fn cdr_deserialize(de: &mut CdrDeserializer<'a, E>) -> CdrResult<Self> {
        de.deserialize_string()
    }
}
impl<'a, E> CdrDeserialize<'a, E> for Locator {
    fn cdr_deserialize(de: &mut CdrDeserializer<'a, E>) -> CdrResult<Self> {
        Ok(Locator::new(
            CdrDeserialize::cdr_deserialize(de)?,
            CdrDeserialize::cdr_deserialize(de)?,
            CdrDeserialize::cdr_deserialize(de)?,
        ))
    }
}
impl<'a, E> CdrDeserialize<'a, E> for ProtocolVersion {
    fn cdr_deserialize(de: &mut CdrDeserializer<'a, E>) -> CdrResult<Self> {
        Ok(ProtocolVersion {
            bytes: CdrDeserialize::cdr_deserialize(de)?,
        })
    }
}
impl<'a, E> CdrDeserialize<'a, E> for BuiltinEndpointQos {
    fn cdr_deserialize(de: &mut CdrDeserializer<'a, E>) -> CdrResult<Self> {
        Ok(Self(CdrDeserialize::cdr_deserialize(de)?))
    }
}
impl<'a, E> CdrDeserialize<'a, E> for BuiltinEndpointSet {
    fn cdr_deserialize(de: &mut CdrDeserializer<'a, E>) -> CdrResult<Self> {
        Ok(Self(CdrDeserialize::cdr_deserialize(de)?))
    }
}
impl<'a, E> CdrDeserialize<'a, E> for Duration {
    fn cdr_deserialize(de: &mut CdrDeserializer<'a, E>) -> CdrResult<Self> {
        Ok(Duration::new(
            CdrDeserialize::cdr_deserialize(de)?,
            CdrDeserialize::cdr_deserialize(de)?,
        ))
    }
}
impl<'a, E> CdrDeserialize<'a, E> for EntityId {
    fn cdr_deserialize(de: &mut CdrDeserializer<'a, E>) -> CdrResult<Self> {
        Ok(EntityId {
            entity_key: CdrDeserialize::cdr_deserialize(de)?,
            entity_kind: CdrDeserialize::cdr_deserialize(de)?,
        })
    }
}

impl<'a, E> CdrDeserializer<'a, E> {
    pub fn new(buffer: &'a [u8]) -> Self {
        Self {
            reader: NextCdrReader::new(buffer, LittleEndian),
            buffer,
            pos: 0,
            _endianness: core::marker::PhantomData,
        }
    }

    fn deserialize_primitive<T: AsBytes>(&mut self) -> CdrResult<T> {
        Ok(T::as_bytes(&mut self.reader)?)
    }
    fn deserialize_bytes(&mut self, length: usize) -> CdrResult<&[u8]> {
        Ok(self.reader.read_all(length)?)
    }
    fn deserialize_string(&mut self) -> CdrResult<String> {
        let length = u32::as_bytes(&mut self.reader)?;
        let character_data = self.reader.read_all(length as usize - 1)?.to_vec();
        self.reader.read_byte()?; // 0-termination
        String::from_utf8(character_data).map_err(|_| CdrError::InvalidData)
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

    fn from_bytes(bytes: &[u8]) -> CdrResult<TestDiscoveryData> {
        let pl = ParameterList::<()>::new(bytes);

        let domain_id = if let Some(pid_data) = pl.get_optional(15) {
            CdrDeserialize::cdr_deserialize(&mut CdrDeserializer::<()>::new(pid_data))?
        } else {
            0
        };
        let domain_tag = if let Some(pid_data) = pl.get_optional(0x4014) {
            CdrDeserialize::cdr_deserialize(&mut CdrDeserializer::<()>::new(pid_data))?
        } else {
            String::from("")
        };
        let vendor_id =
            CdrDeserialize::cdr_deserialize(&mut CdrDeserializer::<()>::new(pl.get_non_optional(0x16)?))?;

        let mut locator_list = vec![];
        for pid_data in pl.get_list(72) {
            locator_list.push(CdrDeserialize::cdr_deserialize(&mut CdrDeserializer::<()>::new(
                pid_data,
            ))?);
        }

        Ok(TestDiscoveryData {
            domain_id,
            domain_tag,
            vendor_id,
            locator_list,
        })
    }

    #[test]
    fn derserialize_test_discovery_data() {
        let bytes = [
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
        let expected = TestDiscoveryData {
            domain_id: 1,
            domain_tag: String::from("ab"),
            vendor_id: [73, 74],
            locator_list: vec![Locator::new(11, 12, [1; 16]), Locator::new(21, 22, [2; 16])],
        };
        assert_eq!(from_bytes(&bytes).unwrap(), expected);
    }

    #[test]
    fn derserialize_test_discovery_data_with_default() {
        let bytes = [
            0x16, 0x00, 4, 0x00, // PID_VENDORID
            73, 74, 0x00, 0x00, // VendorId
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL
        ];
        let expected = TestDiscoveryData {
            domain_id: 0,
            domain_tag: String::from(""),
            vendor_id: [73, 74],
            locator_list: vec![],
        };
        assert_eq!(from_bytes(&bytes).unwrap(), expected);
    }

    #[test]
    fn get_existing_pid() {
        let data = [
            15, 0x00, 0x04, 0x00, // PID_DOMAIN_ID, Length: 4
            0x01, 0x00, 0x00, 0x00, // DomainId
            0x14, 0x40, 0x08, 0x00, // PID_DOMAIN_TAG, Length: 8
            3, 0x00, 0x00, 0x00, // DomainTag: string length (incl. terminator)
            b'a', b'b', 0, 0x00, // DomainTag: string + padding (1 byte)
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL
        ];
        assert_eq!(
            ParameterList::<()>::new(&data).get_optional(15),
            Some(&[0x01, 0x00, 0x00, 0x00][..])
        );
        assert_eq!(
            ParameterList::<()>::new(&data).get_optional(0x4014),
            Some(&[3, 0x00, 0x00, 0x00, b'a', b'b', 0, 0x00][..])
        );
    }

    #[test]
    fn get_non_existing_pid() {
        let data = [
            15, 0x00, 0x04, 0x00, // PID_DOMAIN_ID, Length: 4
            0x01, 0x00, 0x00, 0x00, // DomainId
            0x14, 0x40, 0x08, 0x00, // PID_DOMAIN_TAG, Length: 8
            3, 0x00, 0x00, 0x00, // DomainTag: string length (incl. terminator)
            b'a', b'b', 0, 0x00, // DomainTag: string + padding (1 byte)
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL
        ];
        assert_eq!(ParameterList::<()>::new(&data,).get_optional(10), None);
    }

    #[test]
    fn get_non_existing_pid_sentinel_missing() {
        let data = [
            15, 0x00, 0x04, 0x00, // PID_DOMAIN_ID, Length: 4
            0x01, 0x00, 0x00, 0x00, // DomainId
            0x14, 0x40, 0x08, 0x00, // PID_DOMAIN_TAG, Length: 8
            3, 0x00, 0x00, 0x00, // DomainTag: string length (incl. terminator)
        ];
        assert_eq!(ParameterList::<()>::new(&data,).get_optional(10), None);
    }
}
