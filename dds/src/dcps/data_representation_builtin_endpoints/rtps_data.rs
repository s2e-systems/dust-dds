use crate::{
    infrastructure::{qos_policy::UserDataQosPolicy, type_support::TypeSupport},
    xtypes::{
        binding::XTypesBinding,
        deserializer::{
            AsBytes, EncodingVersion1, LittleEndian, NextCdrReader, XTypesDeserializer,
        },
        dynamic_type::DynamicData,
        error::{XTypesError, XTypesResult},
    },
};

struct ParameterList<'a, E> {
    data: &'a [u8],
    endianness: E,
}

impl<'a, E> ParameterList<'a, E> {
    fn new(data: &'a [u8], endianness: E) -> Self {
        Self {
            data,
            endianness: endianness,
        }
    }
    fn get_data_of_pid(&self, pid: u16) -> Option<&'a [u8]> {
        let mut pointer = &self.data[..];
        loop {
            let current_pid = u16::from_le_bytes([pointer[0], pointer[1]]);
            let length = u16::from_le_bytes([pointer[2], pointer[3]]) as usize;
            if current_pid == pid {
                return Some(&pointer[4..length + 4]);
            } else if current_pid == 1 || pointer.len() < length + 4 {
                return None;
            }
            pointer = &pointer[length + 4..]
        }
    }
}


#[derive(TypeSupport)]
struct _String(String);

#[derive(TypeSupport)]
struct ULong(u32);

impl<'a> ParameterList<'a, LittleEndian> {
    fn get(
        &self,
        pid: u16,
    ) -> XTypesResult<XTypesDeserializer<'a, LittleEndian, EncodingVersion1>> {
        Ok(XTypesDeserializer::new(
            self.get_data_of_pid(pid)
                .ok_or(XTypesError::PidNotFound(pid))?,
            EncodingVersion1,
            LittleEndian,
        ))
    }
    fn get_with_default<T>(
        &self,
        pid: u16,
        f: impl Fn(&mut XTypesDeserializer<'a, LittleEndian, EncodingVersion1>) -> XTypesResult<T>,
        default: T,
    ) -> XTypesResult<T> {
        if let Some(data) = self.get_data_of_pid(pid) {
            f(&mut XTypesDeserializer::new(
                data,
                EncodingVersion1,
                LittleEndian,
            ))
        } else {
            Ok(default)
        }
    }
}

#[derive(Debug, PartialEq)]
struct TestDiscoveryData {
    domain_id: i32,
    domain_tag: String,
    vendor_id: [u8; 2],
}

fn from_bytes(bytes: &[u8]) -> XTypesResult<TestDiscoveryData> {
    let pl = ParameterList::new(bytes, LittleEndian);
    _String::create_sample(&mut pl.get(0x4014)?.deserialize_as_nested(_String::TYPE)?);

    let domain_id = if let Some(pid_data) = pl.get_data_of_pid(15) {
        i32::as_bytes(&mut NextCdrReader::new(pid_data, LittleEndian))?
    } else {
        0
    };
    let domain_tag = if let Some(pid_data) = pl.get_data_of_pid(0x4014) {
        _String::create_sample(&mut pl.get(0x4014)?.deserialize_as_nested(_String::TYPE)?).0
    } else {
        String::from("")
    };
    let data = pl.get_data_of_pid(0x4014).ok_or(XTypesError::PidNotFound(0x4014))?;
    let vendor_id = [data[0], data[1]];
    Ok(TestDiscoveryData {
        domain_id,
        domain_tag,
        vendor_id,
    })
}

// #[test]
// fn derserialize_test_discovery_data() {
//     let bytes = [
//         15, 0x00, 0x04, 0x00, // PID_DOMAIN_ID, Length: 4
//         0x01, 0x00, 0x00, 0x00, // DomainId
//         0x14, 0x40, 0x08, 0x00, // PID_DOMAIN_TAG, Length: 8
//         3, 0x00, 0x00, 0x00, // DomainTag: string length (incl. terminator)
//         b'a', b'b', 0, 0x00, // DomainTag: string + padding (1 byte)
//         0x16, 0x00, 4, 0x00, // PID_VENDORID
//         73, 74, 0x00, 0x00, // VendorId
//         0x01, 0x00, 0x00, 0x00, // PID_SENTINEL
//     ];
//     let expected = TestDiscoveryData {
//         domain_id: 1,
//         domain_tag: String::from("ab"),
//     };
//     assert_eq!(from_bytes(&bytes).unwrap(), expected);
// }

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
        ParameterList::new(&data, LittleEndian).get_data_of_pid(15),
        Some(&[0x01, 0x00, 0x00, 0x00][..])
    );
    assert_eq!(
        ParameterList::new(&data, LittleEndian).get_data_of_pid(0x4014),
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
    assert_eq!(
        ParameterList::new(&data, LittleEndian).get_data_of_pid(10),
        None
    );
}

#[test]
fn get_non_existing_pid_sentinel_missing() {
    let data = [
        15, 0x00, 0x04, 0x00, // PID_DOMAIN_ID, Length: 4
        0x01, 0x00, 0x00, 0x00, // DomainId
        0x14, 0x40, 0x08, 0x00, // PID_DOMAIN_TAG, Length: 8
        3, 0x00, 0x00, 0x00, // DomainTag: string length (incl. terminator)
    ];
    assert_eq!(
        ParameterList::new(&data, LittleEndian).get_data_of_pid(10),
        None
    );
}
