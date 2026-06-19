use crate::{
    dcps::data_representation_builtin_endpoints::{
        parameter_id_values::ParameterId,
        spdp_discovered_participant_data::{BuiltinEndpointQos, BuiltinEndpointSet},
    },
    infrastructure::time::Duration,
    transport::types::{EntityId, Locator, Long, Octet, ProtocolVersion, UnsignedLong},
    xtypes::error::XTypesError,
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

#[derive(Clone, Copy)]
pub(crate) enum Endianness {
    Big,
    Little,
}

pub(crate) struct ParameterList<'a> {
    data: &'a [u8],
    endianness: Endianness,
}

impl<'a> ParameterList<'a> {
    pub(crate) fn new(data: &'a [u8]) -> CdrResult<Self> {
        let endianness = match data[1] {
            2 => Endianness::Big,
            3 => Endianness::Little,
            _ => return Err(CdrError::InvalidData),
        };
        Ok(Self { data, endianness })
    }

    pub(crate) fn get_optional_parameter<T: CdrDeserialize>(
        &self,
        pid: ParameterId,
        default: T,
    ) -> CdrResult<T> {
        if let Some(pid_data) = self.seek_to_pid(pid)? {
            CdrDeserialize::cdr_deserialize(&mut CdrDeserializer::new(pid_data, self.endianness))
        } else {
            Ok(default)
        }
    }
    pub(crate) fn get_non_optional_parameter<T: CdrDeserialize>(
        &self,
        pid: ParameterId,
    ) -> CdrResult<T> {
        CdrDeserialize::cdr_deserialize(&mut CdrDeserializer::new(
            self.seek_to_pid(pid)?.ok_or(CdrError::PidNotFound(pid))?,
            self.endianness,
        ))
    }
    pub(crate) fn get_locator_list(&self, pid: ParameterId) -> CdrResult<Vec<Locator>> {
        let mut locator_list = vec![];
        for pid_data in self.get_list(pid)? {
            locator_list.push(CdrDeserialize::cdr_deserialize(&mut CdrDeserializer::new(
                pid_data,
                self.endianness,
            ))?);
        }
        Ok(locator_list)
    }

    fn get_list(&self, pid: ParameterId) -> CdrResult<Vec<&'a [u8]>> {
        let mut list = Vec::new();
        let mut pointer = self.data;
        loop {
            let mut de = CdrDeserializer::new(pointer, self.endianness);
            let current_pid = i16::cdr_deserialize(&mut de)?;
            let length = u16::cdr_deserialize(&mut de)? as usize;
            if current_pid == pid {
                list.push(&pointer[4..length + 4]);
            } else if current_pid == 1 || pointer.len() < length + 4 {
                return Ok(list);
            }
            pointer = &pointer[length + 4..]
        }
    }
    fn seek_to_pid(&self, pid: ParameterId) -> CdrResult<Option<&'a [u8]>> {
        let mut pointer = self.data;
        loop {
            let mut de = CdrDeserializer::new(pointer, self.endianness);
            let current_pid = i16::cdr_deserialize(&mut de)?;
            let length = u16::cdr_deserialize(&mut de)? as usize;

            if current_pid == pid {
                return Ok(Some(&pointer[4..length + 4]));
            } else if current_pid == 1 || pointer.len() < length + 4 {
                return Ok(None);
            }
            pointer = &pointer[length + 4..]
        }
    }
}

pub(crate) trait CdrDeserialize: Sized {
    fn cdr_deserialize<'a>(de: &mut CdrDeserializer<'a>) -> CdrResult<Self>;
}

impl CdrDeserialize for Octet {
    fn cdr_deserialize<'a>(de: &mut CdrDeserializer<'a>) -> CdrResult<Self> {
        if de.pos + 1 > de.buffer.len() {
            return Err(CdrError::NotEnoughData);
        }
        let ret = de.buffer[de.pos];
        de.pos += 1;
        Ok(ret)
    }
}
impl CdrDeserialize for i16 {
    fn cdr_deserialize<'a>(de: &mut CdrDeserializer<'a>) -> CdrResult<Self> {
        de.seek_padding(2)?;
        let bytes = CdrDeserialize::cdr_deserialize(de)?;
        Ok(match de.endianness {
            Endianness::Big => i16::from_be_bytes(bytes),
            Endianness::Little => i16::from_le_bytes(bytes),
        })
    }
}
impl CdrDeserialize for u16 {
    fn cdr_deserialize<'a>(de: &mut CdrDeserializer<'a>) -> CdrResult<Self> {
        de.seek_padding(2)?;
        let bytes = CdrDeserialize::cdr_deserialize(de)?;
        Ok(match de.endianness {
            Endianness::Big => u16::from_be_bytes(bytes),
            Endianness::Little => u16::from_le_bytes(bytes),
        })
    }
}
impl CdrDeserialize for Long {
    fn cdr_deserialize<'a>(de: &mut CdrDeserializer<'a>) -> CdrResult<Self> {
        de.seek_padding(4)?;
        let bytes = CdrDeserialize::cdr_deserialize(de)?;
        Ok(match de.endianness {
            Endianness::Big => i32::from_be_bytes(bytes),
            Endianness::Little => i32::from_le_bytes(bytes),
        })
    }
}
impl CdrDeserialize for UnsignedLong {
    fn cdr_deserialize<'a>(de: &mut CdrDeserializer<'a>) -> CdrResult<Self> {
        de.seek_padding(4)?;
        let bytes = CdrDeserialize::cdr_deserialize(de)?;
        Ok(match de.endianness {
            Endianness::Big => u32::from_be_bytes(bytes),
            Endianness::Little => u32::from_le_bytes(bytes),
        })
    }
}
impl CdrDeserialize for bool {
    fn cdr_deserialize<'a>(de: &mut CdrDeserializer<'a>) -> CdrResult<Self> {
        Ok(Octet::cdr_deserialize(de)? != 0)
    }
}
impl<const N: usize> CdrDeserialize for [u8; N] {
    fn cdr_deserialize<'a>(de: &mut CdrDeserializer<'a>) -> CdrResult<Self> {
        Ok(de.read_bytes(N)?.try_into().expect("must have size"))
    }
}
impl CdrDeserialize for String {
    fn cdr_deserialize<'a>(de: &mut CdrDeserializer<'a>) -> CdrResult<Self> {
        let length = UnsignedLong::cdr_deserialize(de)?;
        let character_data = de.read_bytes(length as usize - 1)?.to_vec();
        Octet::cdr_deserialize(de)?; // 0-termination
        String::from_utf8(character_data).map_err(|_| CdrError::InvalidData)
    }
}
impl CdrDeserialize for Locator {
    fn cdr_deserialize<'a>(de: &mut CdrDeserializer<'a>) -> CdrResult<Self> {
        Ok(Locator {
            kind: CdrDeserialize::cdr_deserialize(de)?,
            port: CdrDeserialize::cdr_deserialize(de)?,
            address: CdrDeserialize::cdr_deserialize(de)?,
        })
    }
}
impl CdrDeserialize for ProtocolVersion {
    fn cdr_deserialize<'a>(de: &mut CdrDeserializer<'a>) -> CdrResult<Self> {
        Ok(ProtocolVersion {
            bytes: CdrDeserialize::cdr_deserialize(de)?,
        })
    }
}
impl CdrDeserialize for BuiltinEndpointQos {
    fn cdr_deserialize<'a>(de: &mut CdrDeserializer<'a>) -> CdrResult<Self> {
        Ok(Self(CdrDeserialize::cdr_deserialize(de)?))
    }
}
impl CdrDeserialize for BuiltinEndpointSet {
    fn cdr_deserialize<'a>(de: &mut CdrDeserializer<'a>) -> CdrResult<Self> {
        Ok(Self(CdrDeserialize::cdr_deserialize(de)?))
    }
}
impl CdrDeserialize for Duration {
    fn cdr_deserialize<'a>(de: &mut CdrDeserializer<'a>) -> CdrResult<Self> {
        Ok(Duration::new(
            CdrDeserialize::cdr_deserialize(de)?,
            CdrDeserialize::cdr_deserialize(de)?,
        ))
    }
}
impl CdrDeserialize for EntityId {
    fn cdr_deserialize<'a>(de: &mut CdrDeserializer<'a>) -> CdrResult<Self> {
        Ok(EntityId {
            entity_key: CdrDeserialize::cdr_deserialize(de)?,
            entity_kind: CdrDeserialize::cdr_deserialize(de)?,
        })
    }
}

pub(crate) struct CdrDeserializer<'a> {
    buffer: &'a [u8],
    pos: usize,
    endianness: Endianness,
}

impl<'a> CdrDeserializer<'a> {
    pub fn new(buffer: &'a [u8], endianness: Endianness) -> Self {
        Self {
            buffer,
            pos: 0,
            endianness,
        }
    }

    fn read_bytes(&mut self, length: usize) -> CdrResult<&'a [u8]> {
        if self.pos + length > self.buffer.len() {
            return Err(CdrError::NotEnoughData);
        }
        let ret = &self.buffer[self.pos..self.pos + length];
        self.pos += length;
        Ok(ret)
    }

    fn seek_padding(&mut self, alignment: usize) -> CdrResult<()> {
        let mask = alignment - 1;
        let seek = ((self.pos + mask) & !mask) - self.pos;
        if self.pos + seek > self.buffer.len() {
            return Err(CdrError::NotEnoughData);
        }
        self.pos += seek;
        Ok(())
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
        let pl = ParameterList::new(bytes).unwrap();

        Ok(TestDiscoveryData {
            domain_id: pl.get_optional_parameter(15, 0).unwrap(),
            domain_tag: pl.get_optional_parameter(0x4014, String::from("")).unwrap(),
            vendor_id: pl.get_non_optional_parameter(0x16).unwrap(),
            locator_list: pl.get_locator_list(72).unwrap(),
        })
    }

    #[test]
    fn derserialize_test_discovery_data() {
        let bytes = [
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
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
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

    // #[test]
    // fn get_existing_pid() {
    //     let data = [
    //         15, 0x00, 0x04, 0x00, // PID_DOMAIN_ID, Length: 4
    //         0x01, 0x00, 0x00, 0x00, // DomainId
    //         0x14, 0x40, 0x08, 0x00, // PID_DOMAIN_TAG, Length: 8
    //         3, 0x00, 0x00, 0x00, // DomainTag: string length (incl. terminator)
    //         b'a', b'b', 0, 0x00, // DomainTag: string + padding (1 byte)
    //         0x01, 0x00, 0x00, 0x00, // PID_SENTINEL
    //     ];
    //     assert_eq!(
    //         ParameterList::new(&data).seek_to_pid(15).unwrap(),
    //         Some(&[0x01, 0x00, 0x00, 0x00][..])
    //     );
    //     assert_eq!(
    //         ParameterList::new(&data).seek_to_pid(0x4014).unwrap(),
    //         Some(&[3, 0x00, 0x00, 0x00, b'a', b'b', 0, 0x00][..])
    //     );
    // }

    // #[test]
    // fn get_non_existing_pid() {
    //     let data = [
    //         15, 0x00, 0x04, 0x00, // PID_DOMAIN_ID, Length: 4
    //         0x01, 0x00, 0x00, 0x00, // DomainId
    //         0x14, 0x40, 0x08, 0x00, // PID_DOMAIN_TAG, Length: 8
    //         3, 0x00, 0x00, 0x00, // DomainTag: string length (incl. terminator)
    //         b'a', b'b', 0, 0x00, // DomainTag: string + padding (1 byte)
    //         0x01, 0x00, 0x00, 0x00, // PID_SENTINEL
    //     ];
    //     assert_eq!(ParameterList::new(&data,).seek_to_pid(10).unwrap(), None);
    // }

    // #[test]
    // fn get_non_existing_pid_sentinel_missing() {
    //     let data = [
    //         15, 0x00, 0x04, 0x00, // PID_DOMAIN_ID, Length: 4
    //         0x01, 0x00, 0x00, 0x00, // DomainId
    //         0x14, 0x40, 0x08, 0x00, // PID_DOMAIN_TAG, Length: 8
    //         3, 0x00, 0x00, 0x00, // DomainTag: string length (incl. terminator)
    //     ];
    //     assert_eq!(ParameterList::new(&data,).seek_to_pid(10).unwrap(), None);
    // }
}
