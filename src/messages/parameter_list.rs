
use cdr::{LittleEndian, BigEndian, Infinite};
use std::rc::Rc;

use crate::serialized_payload::CdrEndianness;

use super::types;

pub trait Pid {
    fn pid() -> types::ParameterId;
}
//  /////////// ParameterList ///////////
pub trait ParameterOps : std::fmt::Debug{
    fn parameter_id(&self) -> super::types::ParameterId;

    fn length(&self) -> i16;

    fn value(&self, endianness: CdrEndianness) -> Vec<u8>;
}

impl<T> ParameterOps for T
    where T: Pid + serde::Serialize + std::fmt::Debug
{
    fn parameter_id(&self) -> super::types::ParameterId {
        T::pid()
    }

    fn length(&self) -> i16 {
        // rounded up to multple of 4 (that is besides the length of the value may not be a multiple of 4)
        (cdr::size::calc_serialized_data_size(self) + 3 & !3) as i16
    }

    fn value(&self, endianness: CdrEndianness) -> Vec<u8> {
        match endianness {
            CdrEndianness::LittleEndian => cdr::ser::serialize_data::<_,_,LittleEndian>(&self, Infinite).unwrap(),       
            CdrEndianness::BigEndian => cdr::ser::serialize_data::<_,_,BigEndian>(&self, Infinite).unwrap(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Parameter {
    parameter_id: super::types::ParameterId,
    length: i16, // length is rounded up to multple of 4
    value: Vec<u8>,
}

impl Parameter {
    pub fn new(input: &(impl ParameterOps + ?Sized) , endianness: CdrEndianness) -> Self {
        Self {
            parameter_id: input.parameter_id(),
            length: input.length(),
            value: input.value(endianness),
        }
    }

    pub fn from_raw(paramter_id: super::types::ParameterId, value: Vec<u8>) -> Self {
        Self {
            parameter_id: paramter_id,
            length: value.len() as i16,
            value

        }
    }

    pub fn parameter_id(&self) -> super::types::ParameterId {
        self.parameter_id
    }

    pub fn length(&self) -> i16 {
        self.length
    }

    pub fn value(&self) -> &Vec<u8> {
        &self.value
    }

    pub fn get<'de, T: Pid + serde::Deserialize<'de>>(&self, endianness: CdrEndianness) -> Option<T> {
        if self.parameter_id() == T::pid() {
            Some(match endianness {
                CdrEndianness::LittleEndian => cdr::de::deserialize_data::<T, LittleEndian>(&self.value).ok()?,
                CdrEndianness::BigEndian => cdr::de::deserialize_data::<T, BigEndian>(&self.value).ok()?,
            })
        } else {
            None
        }
    }
}



impl ParameterOps for Parameter {
    fn parameter_id(&self) -> super::types::ParameterId {
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

    const PID_SENTINEL : super::types::ParameterId = 0x0001;

    pub fn new() -> Self {
        Self {parameter: Vec::new()}
    }

    pub fn push<T: ParameterOps + 'static>(&mut self, value: T) {
        self.parameter.push(Rc::new(value));
    }

    pub fn find<'de, T>(&self, endianness: CdrEndianness) -> Option<T>
        where T: Pid + serde::Deserialize<'de>
    {
        let parameter = self.parameter.iter().find(|&x| x.parameter_id() == T::pid())?;
        Some(match endianness {
            CdrEndianness::LittleEndian => cdr::de::deserialize_data::<T, LittleEndian>(&parameter.value(endianness)).ok()?,
            CdrEndianness::BigEndian => cdr::de::deserialize_data::<T, BigEndian>(&parameter.value(endianness)).ok()?,
        })
    }

    pub fn find_all<'de, T>(&self, endianness: CdrEndianness) -> Vec<T>
        where T: Pid + serde::Deserialize<'de>
    {
            self.parameter.iter()
            .filter(|&x| x.parameter_id() == T::pid())
            .map(|parameter| match endianness {
                CdrEndianness::LittleEndian => cdr::de::deserialize_data::<T, LittleEndian>(&parameter.value(endianness)).unwrap(),
                CdrEndianness::BigEndian => cdr::de::deserialize_data::<T, BigEndian>(&parameter.value(endianness)).unwrap(),
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

     //  #[test]
    //  fn find_parameter_list() {
    //      let endianness = Endianness::LittleEndian;
    //      let expected = KeyHash([9; 16]);
    //      let parameter_list = ParameterList{parameter: vec![Rc::new(expected), Rc::new(StatusInfo([8; 4]))]};
    //      let result = parameter_list.find::<KeyHash>(endianness).unwrap();
    //      assert_eq!(expected, result);
    //  }
 
    //  #[test]
    //  fn remove_from_parameter_list() {
    //      let expected = ParameterList{parameter: vec![Rc::new(StatusInfo([8; 4]))]};
    //      let mut parameter_list = ParameterList{parameter: vec![Rc::new(KeyHash([9; 16])), Rc::new(StatusInfo([8; 4]))]};
    //      parameter_list.remove::<KeyHash>();
    //      assert_eq!(parameter_list, expected);
    //  }