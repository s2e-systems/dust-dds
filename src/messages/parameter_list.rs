
use cdr::{LittleEndian, BigEndian, Infinite};
use std::rc::Rc;
use std::io::Write;

use super::types;
use super::submessages::submessage_elements::Short;
use super::{Endianness, };
use super::serdes::{SubmessageElement, RtpsSerdesResult, SizeCheck};

pub trait Pid {
    fn pid() -> types::ParameterId;
}
//  /////////// ParameterList ///////////
pub trait ParameterOps : std::fmt::Debug{
    fn parameter_id(&self) -> super::types::ParameterId;

    fn length(&self) -> Short;

    fn value(&self, endianness: Endianness) -> Vec<u8>;
}

impl<T> ParameterOps for T
    where T: Pid + serde::Serialize + std::fmt::Debug
{
    fn parameter_id(&self) -> super::types::ParameterId {
        T::pid()
    }

    fn length(&self) -> Short {
        // rounded up to multple of 4 (that is besides the length of the value may not be a multiple of 4)
        Short((cdr::size::calc_serialized_data_size(self) + 3 & !3) as i16)
    }

    fn value(&self, endianness: Endianness) -> Vec<u8> {
        match endianness {
            Endianness::LittleEndian => cdr::ser::serialize_data::<_,_,LittleEndian>(&self, Infinite).unwrap(),       
            Endianness::BigEndian => cdr::ser::serialize_data::<_,_,BigEndian>(&self, Infinite).unwrap(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Parameter {
    parameter_id: super::types::ParameterId,
    length: Short, // length is rounded up to multple of 4
    value: Vec<u8>,
}

impl Parameter {
    pub fn new(input: &(impl ParameterOps + ?Sized) , endianness: Endianness) -> Self {
        Self {
            parameter_id: input.parameter_id(),
            length: input.length(),
            value: input.value(endianness),
        }
    }

    pub fn get<'de, T: Pid + serde::Deserialize<'de>>(&self, endianness: Endianness) -> Option<T> {
        if self.parameter_id() == T::pid() {
            Some(match endianness {
                Endianness::LittleEndian => cdr::de::deserialize_data::<T, LittleEndian>(&self.value).ok()?,
                Endianness::BigEndian => cdr::de::deserialize_data::<T, BigEndian>(&self.value).ok()?,
            })
        } else {
            None
        }
    }
}

impl SubmessageElement for Parameter {
    fn serialize(&self, writer: &mut impl Write, endianness: Endianness) -> RtpsSerdesResult<()> {
        Short(self.parameter_id).serialize(writer, endianness)?;
        self.length.serialize(writer, endianness)?;
        writer.write(self.value.as_slice())?;
        let padding = self.length.0 as usize - self.value.len();
        for _ in 0..padding {
            writer.write(&[0_u8])?;
        }
        Ok(())
    }

    fn octets(&self) -> usize {       
        4 + self.length.0 as usize
    }    

    fn deserialize(bytes: &[u8], endianness: Endianness) -> RtpsSerdesResult<Self> {
        bytes.check_size_bigger_equal_than(4)?;
        let parameter_id = Short::deserialize(&bytes[0..2], endianness)?.0;
        let length = Short::deserialize(&bytes[2..4], endianness)?;
        let bytes_end = (length.0 + 4) as usize;
        bytes.check_size_bigger_equal_than(bytes_end)?;
        let value = Vec::from(&bytes[4..bytes_end]);
        Ok(Parameter {
            parameter_id, 
            length,
            value,
        })
    }    
}

impl ParameterOps for Parameter {
    fn parameter_id(&self) -> super::types::ParameterId {
        self.parameter_id
    }

    fn length(&self) -> Short {
        self.length
    }

    fn value(&self, _endianness: Endianness) -> Vec<u8> {
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
                (a.value(Endianness::LittleEndian) != b.value(Endianness::LittleEndian)))
            .is_none()
    }
}

impl ParameterList {

    const PID_SENTINEL : super::types::ParameterId = 0x0001;

    pub fn new() -> Self {
        Self {parameter: Vec::new()}
    }

    pub fn push<T: ParameterOps + 'static>(&mut self, value: T) {
        self.parameter.push(Rc::new(value));
    }

    pub fn find<'de, T>(&self, endianness: Endianness) -> Option<T>
        where T: Pid + serde::Deserialize<'de>
    {
        let parameter = self.parameter.iter().find(|&x| x.parameter_id() == T::pid())?;
        Some(match endianness {
            Endianness::LittleEndian => cdr::de::deserialize_data::<T, LittleEndian>(&parameter.value(endianness)).ok()?,
            Endianness::BigEndian => cdr::de::deserialize_data::<T, BigEndian>(&parameter.value(endianness)).ok()?,
        })
    }

    pub fn find_all<'de, T>(&self, endianness: Endianness) -> RtpsSerdesResult<Vec<T>> 
        where T: Pid + serde::Deserialize<'de>
    {
            Ok(self.parameter.iter()
            .filter(|&x| x.parameter_id() == T::pid())
            .map(|parameter| match endianness {
                Endianness::LittleEndian => cdr::de::deserialize_data::<T, LittleEndian>(&parameter.value(endianness)).unwrap(),
                Endianness::BigEndian => cdr::de::deserialize_data::<T, BigEndian>(&parameter.value(endianness)).unwrap(),
            })
            .collect())
    }

    pub fn remove<T>(&mut self) 
        where T: Pid + ParameterOps
    {
        self.parameter.retain(|x| x.parameter_id() != T::pid());
    }

    pub fn len(&self) -> usize {
        self.parameter.len()
    }
}

impl SubmessageElement for ParameterList {
    fn serialize(&self, writer: &mut impl Write, endianness: Endianness) -> RtpsSerdesResult<()> {
         for param in self.parameter.iter() {
            Parameter{parameter_id: param.parameter_id(), length: param.length(), value: param.value(endianness)}.serialize(writer, endianness)?;
        }       
        Short(ParameterList::PID_SENTINEL).serialize(writer, endianness)?;
        writer.write(&[0,0])?; // Sentinel length 0
        Ok(())
    }

    fn octets(&self) -> usize {
        let mut s = 4; //sentinel
        self.parameter.iter().for_each(|param| {s += 2 /*param.parameter_id().octets()*/ + 2 /*param.length().octets()*/ + (param.length().0 as usize) });
        s
    }   

    fn deserialize(bytes: &[u8], endianness: Endianness) -> RtpsSerdesResult<Self> {
        bytes.check_size_bigger_equal_than(2)?;
        let mut parameter_start_index: usize = 0;
        let mut parameters = ParameterList::new();
        loop {
            let parameter = Parameter::deserialize(&bytes[parameter_start_index..], endianness)?;          
            if &parameter.parameter_id == &ParameterList::PID_SENTINEL {
                break;
            }            
            parameter_start_index += parameter.octets();
            parameters.push(parameter);
        }
        Ok(parameters)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inline_qos_types::{StatusInfo, KeyHash, };
    use crate::messages::types::{ParameterId, };
    use serde::{Serialize, Deserialize, };
    // use super::super::serdes::RtpsSerdesError;

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
     fn serialize_parameter() {
         let parameter = Parameter::new(&VendorTest3([1, 2, 3]), Endianness::LittleEndian);
     
         let expected = vec![
             0x03, 0x80, 4, 0, //ParameterID, length
             1, 2, 3, 0, //VendorTest value       
         ];
         let mut writer = Vec::new();
         parameter.serialize(&mut writer, Endianness::LittleEndian).unwrap();
         assert_eq!(expected, writer);
     }
     
     #[test]
     fn deserialize_parameter() {
         let bytes = vec![
             0x03, 0x80, 4, 0, //ParameterID, length
             1, 2, 3, 0, //VendorTest value       
         ];    
         let expected = VendorTest3([1, 2, 3]);    
         let parameter = Parameter::deserialize(&bytes, Endianness::LittleEndian).unwrap();
         let result = parameter.get(Endianness::LittleEndian).unwrap();
     
         assert_eq!(expected, result);
     }    
     
     #[test]
     fn deserialize_parameter_liitle_endian() {
         let endianness = Endianness::LittleEndian;    
         let bytes = vec![
             0x71, 0x0, 4, 0, 
             1, 2, 3, 4,
         ];
         let expected = Parameter::new(&StatusInfo([1, 2, 3, 4]), endianness);
         let result = Parameter::deserialize(&bytes, endianness).unwrap();   
         assert_eq!(expected, result);
     }    
     
     // #[test]
     // fn test_parameter_round_up_to_multiples_of_four() {
     //     let e= Endianness::LittleEndian;
     //     assert_eq!(0, Parameter::new(&VendorTest0([]), e).length);
     //     assert_eq!(4, Parameter::new(&VendorTest1([b'X']), e).length);
     //     assert_eq!(4, Parameter::new(&VendorTest3([b'X'; 3]), e).length);
     //     assert_eq!(4, Parameter::new(&VendorTest4([b'X'; 4]), e).length);
     //     assert_eq!(8, Parameter::new(&VendorTest5([b'X'; 5]), e).length);
     // }
 
     #[test]
     fn serialize_parameter_short() {
         let endianness = Endianness::LittleEndian;
         let parameter = Parameter::new(&VendorTestShort(-1000), endianness);
         let expected = vec![
             0x06, 0x80, 4, 0, 
             0x18, 0xFC, 0, 0,
         ];
         let mut writer = Vec::new();
         parameter.serialize(&mut writer, endianness).unwrap();
         assert_eq!(expected, writer);
 
         let endianness = Endianness::BigEndian;
         let parameter = Parameter::new(&VendorTestShort(-1000), endianness);
         let expected = vec![
             0x80, 0x06, 0, 4, 
             0xFC, 0x18, 0, 0,
         ];
         let mut writer = Vec::new();
         parameter.serialize(&mut writer, endianness).unwrap();
         assert_eq!(expected, writer);
     }
 
     #[test]
     fn deserialize_parameter_short() {
         let endianness = Endianness::LittleEndian;
         let expected = VendorTestShort(-1000);
         let bytes = vec![
             0x06, 0x80, 4, 0, 
             0x18, 0xFC, 0, 0,
         ];
         let parameter = Parameter::deserialize(&bytes, endianness).unwrap();
         let result = parameter.get(endianness).unwrap();
         assert_eq!(expected, result);
 
         let endianness = Endianness::BigEndian;
         let expected = VendorTestShort(-1000);
         let bytes = vec![
             0x80, 0x06, 0, 4, 
             0xFC, 0x18, 0, 0,
         ];
         let parameter = Parameter::deserialize(&bytes, endianness).unwrap();
         let result = parameter.get(endianness).unwrap();
         assert_eq!(expected, result);
     }
 
     #[test]
     fn parameter_serialize_little_endian() {
         let endianness = Endianness::LittleEndian;
         
         let parameter = Parameter::new(&VendorTest3([1, 2, 3]), Endianness::LittleEndian);
         let expected_bytes = vec![
             0x03, 0x80, 4, 0, 
             1, 2, 3, 0,
         ];
         let expected_octets = expected_bytes.len();
         let mut result_bytes = Vec::new();
         parameter.serialize(&mut result_bytes, endianness).unwrap();
         let result_octets = parameter.octets();
         assert_eq!(expected_bytes, result_bytes);
         assert_eq!(expected_octets, result_octets);
     
         let parameter = Parameter::new(&VendorTest0([]), Endianness::LittleEndian);
         let expected_bytes = vec![0x00, 0x80, 0, 0, ];
         let expected_octets = expected_bytes.len();
         let mut result_bytes = Vec::new();
         parameter.serialize(&mut result_bytes, endianness).ok();
         let result_octets = parameter.octets();
         assert_eq!(expected_bytes, result_bytes);
         assert_eq!(expected_octets, result_octets);
     
         let parameter = Parameter::new(&VendorTest4([1,2,3,4]), Endianness::LittleEndian);
         let expected_bytes = vec![
             0x04, 0x80, 4, 0, 
             1, 2, 3, 4,
         ];
         let expected_octets = expected_bytes.len();
         let mut result_bytes = Vec::new();
         parameter.serialize(&mut result_bytes, endianness).ok();
         let result_octets = parameter.octets();
         assert_eq!(expected_bytes, result_bytes);
         assert_eq!(expected_octets, result_octets);
     
         let parameter = Parameter::new(&VendorTest5([1,2,3,4,5]), Endianness::LittleEndian);
         let expected_bytes = vec![
             0x05, 0x80, 8, 0, 
             1, 2, 3, 4,
             5, 0, 0, 0, 
         ];
         let expected_octets = expected_bytes.len();
         let mut result_bytes = Vec::new();
         parameter.serialize(&mut result_bytes, endianness).ok();
         let result_octets = parameter.octets();
         assert_eq!(expected_bytes, result_bytes);
         assert_eq!(expected_octets, result_octets);
     }
         
     #[test]
     fn find_parameter_list() {
         let endianness = Endianness::LittleEndian;
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
     fn parameter_list_deserialize_liitle_endian() {
         let endianness = Endianness::LittleEndian;
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
         let result = ParameterList::deserialize(&bytes, endianness).unwrap(); 
         let result1 = result.find::<VendorTest3>(endianness).unwrap();
         let result2 = result.find::<VendorTest5>(endianness).unwrap();
         assert_eq!(result1, expected1);
         assert_eq!(result2, expected2);
     }
 
     #[test]
     fn serialize_parameter_list() {
         let endianness = Endianness::LittleEndian;
         use crate::inline_qos_types::KeyHash;
         let key_hash = KeyHash([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
         let status_info = StatusInfo([101, 102, 103, 104]);
         let vendor1 = VendorTest1([b'X']);
         let vendor5 = VendorTest5([2;5]);
         let parameter_list = ParameterList{parameter: vec![Rc::new(key_hash), Rc::new(status_info), Rc::new(vendor1), Rc::new(vendor5)]};
     
         let mut writer = Vec::new();
         parameter_list.serialize(&mut writer, endianness).unwrap();
     
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
         assert_eq!(expected, writer);
     }
     
     #[test]
     fn deserialize_parameter_list() {
         let endianness = Endianness::LittleEndian;
         use crate::inline_qos_types::KeyHash;
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
         
         let result = ParameterList::deserialize(&bytes, endianness).unwrap();
         assert_eq!(expected, result);
    }
}