use std::convert::TryInto;
use std::rc::Rc;
use std::io::Write;

use cdr;

#[derive(Debug, Copy, Clone)]
pub enum CdrEndianness {
    LittleEndian,
    BigEndian,
}

#[derive(PartialEq, Debug)]
struct RepresentationIdentifier([u8; 2]);

#[derive(PartialEq, Debug)]
struct RepresentationOptions([u8; 2]);

#[derive(PartialEq, Debug)]
struct SerializedPayloadHeader {
    representation_identifier: RepresentationIdentifier,
    representation_options: RepresentationOptions,
}

pub struct CdrParameterList {
    endianness: CdrEndianness,
    parameter_list: ParameterList,
}

impl CdrParameterList {
    pub fn new(endianness: CdrEndianness) -> Self {
        Self {
            endianness,
            parameter_list: ParameterList::new(),
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {

        let mut bytes = Vec::new();

        // Start by writing the header which depends on the endianness
        match self.endianness {
            CdrEndianness::BigEndian => bytes.write(&[0x00, 0x02, 0x00, 0x00]),
            CdrEndianness::LittleEndian => bytes.write(&[0x00, 0x03, 0x00, 0x00]),
        }.unwrap();

        bytes.append(&mut self.parameter_list.as_bytes(self.endianness));

        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        if bytes.len() < 4 {
            panic!("Message too small");
        }

        let endianness = match &bytes[0..4] {
            &[0x00, 0x02, 0x00, 0x00] => CdrEndianness::BigEndian,
            &[0x00, 0x03, 0x00, 0x00] => CdrEndianness::LittleEndian,
            _ => panic!("Invalid header"),
        };

        let parameter_list = ParameterList::from_bytes(&bytes[4..], endianness);

        Self {
            endianness,
            parameter_list,
        }
    }

    pub fn push<T: Pid + serde::Serialize + std::fmt::Debug + 'static>(&mut self, value: T) {
        self.parameter_list.push(value);
    }

    pub fn find<'de, T>(&self) -> Option<T>
        where T: Pid + serde::Deserialize<'de>
    {
        self.parameter_list.find(self.endianness)
    }

    pub fn find_all<'de, T>(&self) -> Vec<T>
        where T: Pid + serde::Deserialize<'de>
    {
        self.parameter_list.find_all(self.endianness)
    }
}

pub type ParameterId = i16;

pub trait Pid {
    fn pid() -> ParameterId;
}

//  /////////// ParameterList ///////////
pub trait ParameterOps : std::fmt::Debug{
    fn parameter_id(&self) -> ParameterId;

    fn length(&self) -> i16;

    fn value(&self, endianness: CdrEndianness) -> Vec<u8>;
}

impl<T> ParameterOps for T
    where T: Pid + serde::Serialize + std::fmt::Debug
{
    fn parameter_id(&self) -> ParameterId {
        T::pid()
    }

    fn length(&self) -> i16 {
        // rounded up to multple of 4 (that is besides the length of the value may not be a multiple of 4)
        (cdr::size::calc_serialized_data_size(self) + 3 & !3) as i16
    }

    fn value(&self, endianness: CdrEndianness) -> Vec<u8> {
        let mut value = match endianness {
            CdrEndianness::LittleEndian => cdr::ser::serialize_data::<_,_,cdr::LittleEndian>(&self, cdr::Infinite).unwrap(),       
            CdrEndianness::BigEndian => cdr::ser::serialize_data::<_,_,cdr::BigEndian>(&self, cdr::Infinite).unwrap(),
        };

        let padding = self.length() as usize - value.len();

        for _ in 0..padding {
            value.push(0);
        }

        value
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Parameter {
    parameter_id: ParameterId,
    length: i16, // length is rounded up to multple of 4
    value: Vec<u8>,
}

impl Parameter {
    pub fn new(parameter_id: ParameterId, value: Vec<u8>) -> Self {
        Self {
            parameter_id,
            length: (value.len() + 3 & !3) as i16,
            value,
        }
    }

    pub fn from_parameter_ops(input: &(impl ParameterOps + ?Sized) , endianness: CdrEndianness) -> Self {
        Self {
            parameter_id: input.parameter_id(),
            length: input.length(),
            value: input.value(endianness),
        }
    }

    pub fn value(&self) -> &Vec<u8> {
        &self.value
    }

    pub fn get<'de, T: Pid + serde::Deserialize<'de>>(&self, endianness: CdrEndianness) -> Option<T> {
        if self.parameter_id() == T::pid() {
            Some(match endianness {
                CdrEndianness::LittleEndian => cdr::de::deserialize_data::<T, cdr::LittleEndian>(&self.value).ok()?,
                CdrEndianness::BigEndian => cdr::de::deserialize_data::<T, cdr::BigEndian>(&self.value).ok()?,
            })
        } else {
            None
        }
    }
}



impl ParameterOps for Parameter {
    fn parameter_id(&self) -> ParameterId {
        self.parameter_id
    }

    fn length(&self) -> i16 {
        self.length
    }

    fn value(&self, _endianness: CdrEndianness) -> Vec<u8> {
        self.value.clone()
    }
}

#[derive(Debug, Clone)]
pub struct ParameterList {
    parameter: Vec<Rc<dyn ParameterOps>>,
}

impl PartialEq for ParameterList{
    fn eq(&self, other: &Self) -> bool {
        self.parameter.iter().zip(other.parameter.iter())
            .find(|(a,b)| 
                (a.parameter_id() != b.parameter_id()) && 
                (a.length() != b.length()) && 
                (a.value(CdrEndianness::LittleEndian) != b.value(CdrEndianness::LittleEndian)))
            .is_none()
    }
}

impl ParameterList {

    pub const PID_SENTINEL : ParameterId = 0x0001;

    pub fn new() -> Self {
        Self {parameter: Vec::new()}
    }

    pub fn parameter(&self) -> &Vec<Rc<dyn ParameterOps>> {
        &self.parameter
    }

    pub fn push<T: ParameterOps + 'static>(&mut self, value: T) {
        self.parameter.push(Rc::new(value));
    }

    pub fn find<'de, T>(&self, endianness: CdrEndianness) -> Option<T>
        where T: Pid + serde::Deserialize<'de>
    {
        let parameter = self.parameter.iter().find(|&x| x.parameter_id() == T::pid())?;
        Some(match endianness {
            CdrEndianness::LittleEndian => cdr::de::deserialize_data::<T, cdr::LittleEndian>(&parameter.value(endianness)).unwrap(),
            CdrEndianness::BigEndian => cdr::de::deserialize_data::<T, cdr::BigEndian>(&parameter.value(endianness)).unwrap(),
        })
    }

    pub fn find_all<'de, T>(&self, endianness: CdrEndianness) -> Vec<T>
        where T: Pid + serde::Deserialize<'de>
    {
            self.parameter.iter()
            .filter(|&x| x.parameter_id() == T::pid())
            .map(|parameter| match endianness {
                CdrEndianness::LittleEndian => cdr::de::deserialize_data::<T, cdr::LittleEndian>(&parameter.value(endianness)).unwrap(),
                CdrEndianness::BigEndian => cdr::de::deserialize_data::<T, cdr::BigEndian>(&parameter.value(endianness)).unwrap(),
            })
            .collect()
    }

    pub fn remove<T>(&mut self) 
        where T: Pid + ParameterOps
    {
        self.parameter.retain(|x| x.parameter_id() != T::pid());
    }

    pub fn len(&self) -> usize {
        self.parameter.len()
    }

    pub fn as_bytes(&self, endianness: CdrEndianness) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        for parameter in self.parameter().iter() {
            match endianness {
                CdrEndianness::LittleEndian => {
                    bytes.write(&parameter.parameter_id().to_le_bytes()).unwrap();
                    bytes.write(&parameter.length().to_le_bytes()).unwrap();
                },
                CdrEndianness::BigEndian => {
                    bytes.write(&parameter.parameter_id().to_be_bytes()).unwrap();
                    bytes.write(&parameter.length().to_be_bytes()).unwrap();
                }
            };

            bytes.write(parameter.value(endianness).as_slice()).unwrap();
            let padding = parameter.length() as usize - parameter.value(endianness).len();
            for _ in 0..padding {
                bytes.write(&[0_u8]).unwrap();
            }
        }

        match endianness {
            CdrEndianness::BigEndian => bytes.write(&ParameterList::PID_SENTINEL.to_be_bytes()).unwrap(),
            CdrEndianness::LittleEndian => bytes.write(&ParameterList::PID_SENTINEL.to_le_bytes()).unwrap(),
        };
        bytes.write(&[0,0]).unwrap(); // Sentinel length 0

        bytes
    }

    pub fn from_bytes(bytes: &[u8], endianness: CdrEndianness) -> Self {
        let mut parameter_start_index: usize = 0;
        let mut parameter_list = ParameterList::new();
        loop {
            let (parameter_id, length) = match endianness {
                CdrEndianness::BigEndian => {
                    let parameter_id = i16::from_be_bytes(bytes[parameter_start_index..parameter_start_index+2].try_into().unwrap());
                    let length = i16::from_be_bytes(bytes[parameter_start_index+2..parameter_start_index+4].try_into().unwrap());
                    (parameter_id, length)
                },
                CdrEndianness::LittleEndian => {
                    let parameter_id = i16::from_le_bytes(bytes[parameter_start_index..parameter_start_index+2].try_into().unwrap());
                    let length = i16::from_le_bytes(bytes[parameter_start_index+2..parameter_start_index+4].try_into().unwrap());
                    (parameter_id, length)
                },
            };

            if parameter_id == ParameterList::PID_SENTINEL {
                break;
            }     

            let bytes_end = parameter_start_index + (length + 4) as usize;
            let value = Vec::from(&bytes[parameter_start_index+4..bytes_end]);
            parameter_start_index = bytes_end;

            parameter_list.push(Parameter::new(parameter_id, value));
        }

        parameter_list
    }
}


#[derive(PartialEq, Debug)]
struct StandardSerializedPayload {
    header: SerializedPayloadHeader,
    data: Vec<u8>,
}


#[derive(PartialEq, Debug)]
pub struct SerializedPayload(pub Vec<u8>);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rtps::serialized_payload::{ParameterId, Pid};
    use serde::{Serialize, Deserialize, };
    use crate::rtps::inline_qos_types::{StatusInfo, KeyHash, };

    #[allow(overflowing_literals)]
    const PID_VENDOR_TEST_0 : ParameterId = 0x0000 | 0x8000;
    #[allow(overflowing_literals)]
    const PID_VENDOR_TEST_1 : ParameterId = 0x0001 | 0x8000;
    #[allow(overflowing_literals)]
    const PID_VENDOR_TEST_3 : ParameterId = 0x0003 | 0x8000;
    #[allow(overflowing_literals)]
    const PID_VENDOR_TEST_4 : ParameterId = 0x0004 | 0x8000;
    #[allow(overflowing_literals)]
    const PID_VENDOR_TEST_5 : ParameterId = 0x0005 | 0x8000;
    #[allow(overflowing_literals)]
    const PID_VENDOR_TEST_SHORT : ParameterId = 0x0006 | 0x8000;

    // /////////////////////// ParameterList Tests ////////////////////////

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    pub struct VendorTest0(pub [u8; 0]);
    impl Pid for VendorTest0 {
        fn pid() -> ParameterId {
            PID_VENDOR_TEST_0
        }
    }
         
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    pub struct VendorTest1(pub [u8; 1]);
    impl Pid for VendorTest1 {
        fn pid() -> ParameterId {
            PID_VENDOR_TEST_1
        }
    }
    
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    pub struct VendorTest3(pub [u8; 3]);
    impl Pid for VendorTest3 {
        fn pid() -> ParameterId {
            PID_VENDOR_TEST_3
        }
    }
    
    #[derive(Debug, PartialEq, Serialize)]
    pub struct VendorTest4(pub [u8; 4]);
    impl Pid for VendorTest4 {
        fn pid() -> ParameterId {
            PID_VENDOR_TEST_4
        }
    }
    
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    pub struct VendorTest5(pub [u8; 5]);
    impl Pid for VendorTest5 {
        fn pid() -> ParameterId {
            PID_VENDOR_TEST_5
        }
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    pub struct VendorTestShort(pub i16);
    impl Pid for VendorTestShort {
        fn pid() -> ParameterId {
            PID_VENDOR_TEST_SHORT
        }
    }

    #[test]
     fn test_parameter_round_up_to_multiples_of_four() {
         let e= CdrEndianness::LittleEndian;
         assert_eq!(0, Parameter::from_parameter_ops(&VendorTest0([]), e).length());
         assert_eq!(4, Parameter::from_parameter_ops(&VendorTest1([b'X']), e).length());
         assert_eq!(4, Parameter::from_parameter_ops(&VendorTest3([b'X'; 3]), e).length());
         assert_eq!(4, Parameter::from_parameter_ops(&VendorTest4([b'X'; 4]), e).length());
         assert_eq!(8, Parameter::from_parameter_ops(&VendorTest5([b'X'; 5]), e).length());
     }

      #[test]
     fn find_parameter_list() {
         let endianness = CdrEndianness::LittleEndian;
         let expected = KeyHash([9; 16]);
         let parameter_list = ParameterList{parameter: vec![Rc::new(expected), Rc::new(StatusInfo([8; 4]))]};
         let result = parameter_list.find::<KeyHash>(endianness).unwrap();
         assert_eq!(expected, result);
     }
 
     #[test]
     fn remove_from_parameter_list() {
         let expected = ParameterList{parameter: vec![Rc::new(StatusInfo([8; 4]))]};
         let mut parameter_list = ParameterList{parameter: vec![Rc::new(KeyHash([9; 16])), Rc::new(StatusInfo([8; 4]))]};
         parameter_list.remove::<KeyHash>();
         assert_eq!(parameter_list, expected);
     }
     
    #[test]
    fn create_and_get_parameter() {
        let parameter = Parameter::from_parameter_ops(&VendorTest3([1, 2, 3]), CdrEndianness::LittleEndian);
    
        let expected_length = 4;
        #[allow(overflowing_literals)]
        let expected_pid = 0x8003;
        let expected_value = vec![
            // 0x03, 0x80, 4, 0, //ParameterID, length
            1, 2, 3, 0, //VendorTest value       
        ];
        
        assert_eq!(expected_pid, parameter.parameter_id());
        assert_eq!(expected_length, parameter.length());
        assert_eq!(&expected_value, parameter.value());

        let expected = VendorTest3([1, 2, 3]);    
        let result = parameter.get::<VendorTest3>(CdrEndianness::LittleEndian).unwrap();
    
        assert_eq!(expected, result);
    }
    
    #[test]
    fn parameter_list_from_bytes_liitle_endian() {
        let bytes = vec![
            0x03, 0x80, 4, 0, // ParameterID, length 
            1, 2, 3, 0, // value
            0x05, 0x80, 8, 0, // ParameterID, length 
            10, 20, 30, 40, // value
            50, 0, 0, 0, // value
            0x01, 0x00, 0, 0, // Sentinel
        ];  
        let expected1 = VendorTest3([1, 2, 3]);
        let expected2 = VendorTest5([10, 20, 30, 40, 50]);
    
        // Note that the complete list cannot simply be checked for equality here
        // this is because the value of each parameter needs to be interpreted for that
        // otherwise the value containes the padding 0's 
        let result = ParameterList::from_bytes(&bytes, CdrEndianness::LittleEndian); 
        let result1 = result.find::<VendorTest3>(CdrEndianness::LittleEndian).unwrap();
        let result2 = result.find::<VendorTest5>(CdrEndianness::LittleEndian).unwrap();
        assert_eq!(result1, expected1);
        assert_eq!(result2, expected2);

        let key_hash = KeyHash([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
        let status_info = StatusInfo([101, 102, 103, 104]);
    
        let mut expected = ParameterList::new();
        expected.push(key_hash);
        expected.push(status_info);

        let bytes = vec![
            0x70, 0x00, 16, 0, //ParameterID, length
            1, 2, 3, 4, //key_hash
            5, 6, 7, 8,  //key_hash
            9, 10, 11, 12,  //key_hash
            13, 14, 15, 16, //key_hash
            0x71, 0x00, 4, 0, //ParameterID, length
            101, 102, 103, 104, //status_info
            1, 0, 0, 0 // Sentinel
        ];
        
        let result = ParameterList::from_bytes(&bytes, CdrEndianness::LittleEndian);
        assert_eq!(expected, result);
    }

    #[test]
    fn parameter_list_as_bytes_little_endian() {
        let key_hash = KeyHash([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
        let status_info = StatusInfo([101, 102, 103, 104]);
        let vendor1 = VendorTest1([b'X']);
        let vendor5 = VendorTest5([2;5]);
        let mut parameter_list = ParameterList::new();
        parameter_list.push(key_hash);
        parameter_list.push(status_info);
        parameter_list.push(vendor1);
        parameter_list.push(vendor5);
    
        let bytes = parameter_list.as_bytes(CdrEndianness::LittleEndian);
    
        let expected = vec![
            0x70, 0x00, 16, 0, //ParameterID, length
            1, 2, 3, 4, //key_hash
            5, 6, 7, 8,  //key_hash
            9, 10, 11, 12,  //key_hash
            13, 14, 15, 16, //key_hash
            0x71, 0x00, 4, 0, //ParameterID, length
            101, 102, 103, 104, //status_info
            0x01, 0x80, 4, 0, //ParameterID, length
            b'X', 0, 0, 0, //vendor1
            0x05, 0x80, 8, 0, //ParameterID, length
            2, 2, 2, 2, //vendor5
            2, 0, 0, 0, //vendor5
            1, 0, 0, 0 // Sentinel
        ];
        assert_eq!(expected, bytes);
    }
}