/// 
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// 8.3.5 RTPS SubmessageElements
///  

use std::collections::BTreeSet;
use std::rc::Rc;
use std::io::Write;

use cdr::{LittleEndian, BigEndian, Infinite};

use crate::types::{SequenceNumber, Locator};
use crate::primitive_types::{Long, ULong, Short, };
use crate::serdes::{RtpsSerialize, RtpsDeserialize, Endianness, RtpsSerdesResult, SizeCheck};

use super::types::{ParameterId, Pid};

//  /////////   The GuidPrefix, and EntityId
// Same as in crate::types

//  /////////   VendorId
// Same as in crate::types

//  /////////   ProtocolVersion
// Same as in crate::types

//  /////////   SequenceNumber
// Same as in crate::types

//  /////////   SequenceNumberSet

#[derive(PartialEq, Debug)]
pub struct SequenceNumberSet {
    base: SequenceNumber,
    set: BTreeSet<SequenceNumber>,
}

impl SequenceNumberSet {
    pub fn new(base: SequenceNumber, set: BTreeSet<SequenceNumber>) -> Self {
        SequenceNumberSet {
            base,
            set,
        }
    }

    pub fn from_set(set: BTreeSet<SequenceNumber>) -> Self { 
        let base = *set.iter().next().unwrap_or(&SequenceNumber(0));
        Self {base, set } 
    }

    pub fn base(&self) -> &SequenceNumber {
        &self.base
    }

    pub fn set(&self) -> &BTreeSet<SequenceNumber> {
        &self.set
    }

    pub fn is_valid(&self) -> bool {
        let min = *self.set.iter().next().unwrap(); // First element. Must exist by the invariant
        let max = *self.set.iter().next_back().unwrap(); // Last element. Must exist by the invariant

        if min >= SequenceNumber(1) && max.0 - min.0 < 256 {
            true
        } else {
            false
        }
    }
}

impl RtpsSerialize for SequenceNumberSet {
    fn serialize(&self, writer: &mut impl Write, endianness: Endianness) -> RtpsSerdesResult<()> {
        let num_bits = if self.set.is_empty() {
            0 
        } else {
            (self.set.iter().last().unwrap().0 - self.base.0) as usize + 1
        };
        let m = ((num_bits + 31) / 32) as usize;
        let mut bitmaps = vec![0_u32; m];
        self.base.serialize(writer, endianness)?;
        (num_bits as ULong).serialize(writer, endianness)?;
        for seq_num in &self.set {
            let delta_n = (seq_num.0 - self.base.0) as usize;
            let bitmap_i = delta_n / 32;
            let bitmask = 1 << (31 - delta_n % 32);
            bitmaps[bitmap_i] |= bitmask;
        };
        for bitmap in bitmaps {
            (bitmap as ULong).serialize(writer, endianness)?;
        }
        Ok(())
    }
}

impl RtpsDeserialize for SequenceNumberSet {
    fn deserialize(bytes: &[u8], endianness: Endianness) -> RtpsSerdesResult<Self> {
        bytes.check_size_bigger_equal_than(12)?;

        let base = SequenceNumber::deserialize(&bytes[0..8], endianness)?;
        let num_bits = ULong::deserialize(&bytes[8..12], endianness)? as usize;

        // Get bitmaps from "long"s that follow the numBits field in the message
        // Note that the amount of bitmaps that are included in the message are 
        // determined by the number of bits (32 max per bitmap, and a max of 256 in 
        // total which means max 8 bitmap "long"s)
        let m = (num_bits + 31) / 32;        
        let mut bitmaps = Vec::with_capacity(m);
        for i in 0..m {
            let index_of_byte_current_bitmap = 12 + i * 4;
            bytes.check_size_bigger_equal_than(index_of_byte_current_bitmap+4)?;
            bitmaps.push(Long::deserialize(&bytes[index_of_byte_current_bitmap..index_of_byte_current_bitmap+4], endianness)?);
        };
        // Interpet the bitmaps and insert the sequence numbers if they are encode in the bitmaps
        let mut set = BTreeSet::new(); 
        for delta_n in 0..num_bits {
            let bitmask = 1 << (31 - delta_n % 32);
            if  bitmaps[delta_n / 32] & bitmask == bitmask {               
                let seq_num = SequenceNumber(delta_n as i64 + base.0);
                set.insert(seq_num);
            }
        }
        Ok(Self {base, set})
    }    
}



//  /////////   FragmentNumber

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)] //Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash
pub struct FragmentNumber(pub ULong);

impl RtpsSerialize for FragmentNumber {
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: Endianness) -> RtpsSerdesResult<()> {
        self.0.serialize(writer, endianness)?;
        Ok(())
    }    
}

impl RtpsDeserialize for FragmentNumber {
    fn deserialize(bytes: &[u8], endianness: Endianness) -> RtpsSerdesResult<Self> {
        Ok(Self(ULong::deserialize(bytes, endianness)?))
    }    
}



//  ////////    FragmentNumberSet

#[derive(PartialEq, Debug)]
pub struct FragmentNumberSet {
    base: FragmentNumber,
    set: BTreeSet<FragmentNumber>,
}

impl FragmentNumberSet {
    pub fn new(set: BTreeSet<FragmentNumber>) -> Self { 
        let base = *set.iter().next().unwrap_or(&FragmentNumber(0));
        Self {base, set } 
    }

    pub fn is_valid(&self) -> bool {
        let min = *self.set.iter().next().unwrap(); // First element. Must exist by the invariant
        let max = *self.set.iter().next_back().unwrap(); // Last element. Must exist by the invariant

        if min >= FragmentNumber(1) && max.0 - min.0 < 256 {
            true
        } else {
            false
        }
    }
}

impl RtpsSerialize for FragmentNumberSet {
    fn serialize(&self, writer: &mut impl Write, endianness: Endianness) -> RtpsSerdesResult<()> {
        let num_bits = if self.set.is_empty() {
            0 
        } else {
            self.set.iter().last().unwrap().0 - self.base.0 + 1
        };
        let m = ((num_bits + 31) / 32) as usize;
        let mut bitmaps = vec![0_u32; m];
        self.base.serialize(writer, endianness)?;
        (num_bits as ULong).serialize(writer, endianness)?;
        for seq_num in &self.set {
            let delta_n = (seq_num.0 - self.base.0) as usize;
            let bitmap_i = delta_n / 32;
            let bitmask = 1 << (31 - delta_n % 32);
            bitmaps[bitmap_i] |= bitmask;
        };
        for bitmap in bitmaps {
            (bitmap as ULong).serialize(writer, endianness)?;
        }
        Ok(())
    }
}
impl RtpsDeserialize for FragmentNumberSet {
    fn deserialize(bytes: &[u8], endianness: Endianness) -> RtpsSerdesResult<Self> {
        bytes.check_size_bigger_equal_than(8)?;
        let base = FragmentNumber::deserialize(&bytes[0..4], endianness)?;
        let num_bits = ULong::deserialize(&bytes[4..8], endianness)? as usize;

        // Get bitmaps from "long"s that follow the numBits field in the message
        // Note that the amount of bitmaps that are included in the message are 
        // determined by the number of bits (32 max per bitmap, and a max of 256 in 
        // total which means max 8 bitmap "long"s)
        let m = (num_bits + 31) / 32;        
        let mut bitmaps = Vec::with_capacity(m);
        for i in 0..m {
            let index_of_byte_current_bitmap = 8 + i * 4;
            bytes.check_size_bigger_equal_than(index_of_byte_current_bitmap+4)?;
            bitmaps.push(Long::deserialize(&bytes[index_of_byte_current_bitmap..index_of_byte_current_bitmap+4], endianness)?);
        };
        // Interpet the bitmaps and insert the sequence numbers if they are encode in the bitmaps
        let mut set = BTreeSet::new(); 
        for delta_n in 0..num_bits {
            let bitmask = 1 << (31 - delta_n % 32);
            if  bitmaps[delta_n / 32] & bitmask == bitmask {               
                let frag_num = FragmentNumber(delta_n as u32 + base.0);
                set.insert(frag_num);
            }
        }
        Ok(Self {base, set})
    }    
}



//  /////////// ParameterList ///////////
pub trait ParameterOps : std::fmt::Debug{
    fn parameter_id(&self) -> ParameterId;

    fn length(&self) -> Short;

    fn value(&self, endianness: Endianness) -> Vec<u8>;
}

impl<T> ParameterOps for T
    where T: Pid + serde::Serialize + std::fmt::Debug
{
    fn parameter_id(&self) -> ParameterId {
        T::pid()
    }

    fn length(&self) -> Short {
        // rounded up to multple of 4 (that is besides the length of the value may not be a multiple of 4)
        (cdr::size::calc_serialized_data_size(self) + 3 & !3) as Short
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
    parameter_id: ParameterId,
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

impl  RtpsSerialize for Parameter {
    fn serialize(&self, writer: &mut impl Write, endianness: Endianness) -> RtpsSerdesResult<()> {
        self.parameter_id.serialize(writer, endianness)?;
        self.length.serialize(writer, endianness)?;
        writer.write(self.value.as_slice())?;
        let padding = self.length as usize - self.value.len();
        for _ in 0..padding {
            writer.write(&[0_u8])?;
        }
        Ok(())
    }

    fn octets(&self) -> usize {       
        4 + self.length as usize
    }    
}

impl RtpsDeserialize for Parameter {
    fn deserialize(bytes: &[u8], endianness: Endianness) -> RtpsSerdesResult<Self> {
        bytes.check_size_bigger_equal_than(4)?;
        let parameter_id = ParameterId::deserialize(&bytes[0..2], endianness)?;
        let length = Short::deserialize(&bytes[2..4], endianness)?;
        let bytes_end = (length + 4) as usize;
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
    fn parameter_id(&self) -> ParameterId {
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

    const PID_SENTINEL : ParameterId = 0x0001;

    pub fn new() -> Self {
        Self {parameter: Vec::new()}
    }

    pub fn push<T: ParameterOps + 'static>(&mut self, value: T) {
        self.parameter.push(Rc::new(value));
    }

    pub fn parameter(&self) -> &Vec<Rc<dyn ParameterOps>> {
        &self.parameter
    }

    pub fn find<'de, T>(&self, endianness: Endianness) -> Option<T>
        where T: Pid + ParameterOps + serde::Deserialize<'de>
    {
        let parameter = self.parameter.iter().find(|&x| x.parameter_id() == T::pid())?;
        Some(match endianness {
            Endianness::LittleEndian => cdr::de::deserialize_data::<T, LittleEndian>(&parameter.value(endianness)).ok()?,
            Endianness::BigEndian => cdr::de::deserialize_data::<T, BigEndian>(&parameter.value(endianness)).ok()?,
        })
    }

    pub fn remove<T>(&mut self) 
        where T: Pid + ParameterOps
    {
        self.parameter.retain(|x| x.parameter_id() != T::pid());
    }
}

impl RtpsSerialize for ParameterList {
    fn serialize(&self, writer: &mut impl Write, endianness: Endianness) -> RtpsSerdesResult<()> {
         for param in self.parameter.iter() {
            Parameter{parameter_id: param.parameter_id(), length: param.length(), value: param.value(endianness)}.serialize(writer, endianness)?;
        }       
        ParameterList::PID_SENTINEL.serialize(writer, endianness)?;
        writer.write(&[0,0])?; // Sentinel length 0
        Ok(())
    }

    fn octets(&self) -> usize {
        let mut s = 4; //sentinel
        self.parameter.iter().for_each(|param| {s += param.parameter_id().octets() + param.length().octets() + (param.length() as usize) });
        s
    }   
}

impl RtpsDeserialize for ParameterList {
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



//  /////////// Count ///////////
// same as in super::types



//  ///////////   LocatorList

#[derive(Debug, PartialEq)]
pub struct LocatorList(pub Vec<Locator>);

impl RtpsSerialize for LocatorList {
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: Endianness) -> RtpsSerdesResult<()> {
        let num_locators = self.0.len() as ULong;
        num_locators.serialize(writer, endianness)?;
        for locator in &self.0 {
            locator.serialize(writer, endianness)?;
        };
        Ok(())
    }
}

impl RtpsDeserialize for LocatorList {
    fn deserialize(bytes: &[u8], endianness: Endianness) -> RtpsSerdesResult<Self> {
        bytes.check_size_bigger_equal_than(4)?;
        let size = bytes.len();
        let num_locators = ULong::deserialize(&bytes[0..4], endianness)?;
        let mut locators = Vec::<Locator>::new();
        let mut index = 4;
        while index < size && locators.len() < num_locators as usize {
            bytes.check_size_bigger_equal_than(index+24)?;
            let locator = Locator::deserialize( &bytes[index..index+24], endianness)?;
            index += locator.octets();
            locators.push(locator);
        };
        Ok(Self(locators))
    }
}



//  ///////////   SerializedData   //////////////////
// todo

//  ///////////   SerializedDataFragment  ///////////
// todo

//  ///////////   GroupDigest   //////////////////////
// todo


#[cfg(test)]
mod tests {
    use super::*;
    use crate::inline_qos_types::{StatusInfo, KeyHash};
    use crate::messages::types::ParameterId;
    use serde::{Serialize, Deserialize};

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

    // /////////////////////// SequenceNumberSet Tests ////////////////////////

    #[test]
    fn sequence_number_set_constructor() {
        let expected = SequenceNumberSet{
            base: SequenceNumber(1001),
            set:  [SequenceNumber(1001), SequenceNumber(1003)].iter().cloned().collect(),
        };
        let result = SequenceNumberSet::from_set([SequenceNumber(1001), SequenceNumber(1003)].iter().cloned().collect());
        assert_eq!(expected, result);
    }

    #[test]
    fn sequence_number_set_constructor_empty_set() {        
        let expected = SequenceNumberSet{
            base: SequenceNumber(0),
            set:  [].iter().cloned().collect(),
        };
        let result = SequenceNumberSet::from_set([].iter().cloned().collect());
        assert_eq!(expected, result);
    }
    
    #[test]
    fn deserialize_sequence_number_set_empty() {
        let expected = SequenceNumberSet{
            base: SequenceNumber(3),
            set: [].iter().cloned().collect()
        };
        let bytes = vec![
            0, 0, 0, 0, // base
            0, 0, 0, 3, // base
            0, 0, 0, 0, // num bits
        ];
        let result = SequenceNumberSet::deserialize(&bytes, Endianness::BigEndian).unwrap();
        assert_eq!(expected, result);
    }
    
    #[test]
    fn deserialize_sequence_number_set_one_bitmap_be() {
        let expected = SequenceNumberSet{
            base: SequenceNumber(3),
            set: [SequenceNumber(3), SequenceNumber(4)].iter().cloned().collect()
        };
        let bytes = vec![
            0, 0, 0, 0, // base
            0, 0, 0, 3, // base
            0, 0, 0, 2, // num bits
            0b_11000000, 0b_00000000, 0b_00000000, 0b_00000000, 
        ];
        let result = SequenceNumberSet::deserialize(&bytes, Endianness::BigEndian).unwrap();
        assert_eq!(expected, result);
    }
    
    #[test]
        fn deserialize_sequence_number_set_one_bitmap_le() {
        let expected = SequenceNumberSet{
            base: SequenceNumber(3),
            set: [SequenceNumber(3), SequenceNumber(4)].iter().cloned().collect()
        };
        let bytes = vec![
            0, 0, 0, 0, // base
            3, 0, 0, 0, // base
            2, 0, 0, 0, // num bits
            0b_00000000, 0b_00000000, 0b_00000000, 0b_11000000, 
        ];
        let result = SequenceNumberSet::deserialize(&bytes, Endianness::LittleEndian).unwrap();
        assert_eq!(expected, result);
    }
    
    #[test]
    fn deserialize_sequence_number_set_multiple_bitmaps() {
        let expected = SequenceNumberSet{
            base: SequenceNumber(1000),
            set: [SequenceNumber(1001), SequenceNumber(1003), SequenceNumber(1032), SequenceNumber(1033)].iter().cloned().collect()
        };
        let bytes = vec![
            0, 0, 0, 0, // base
            0, 0, 3, 232, // base
            0, 0, 0, 34, // num bits
            0b_01010000, 0b_00000000, 0b_00000000, 0b_00000000, 
            0b_11000000, 0b_00000000, 0b_00000000, 0b_00000000, 
        ];
        let result = SequenceNumberSet::deserialize(&bytes, Endianness::BigEndian).unwrap();
        assert_eq!(expected, result);
    }
    
    #[test]
    fn deserialize_sequence_number_set_max_bitmaps_big_endian() {
        let expected = SequenceNumberSet{
            base: SequenceNumber(1000),
            set: [SequenceNumber(1000), SequenceNumber(1255)].iter().cloned().collect()
        };
        let bytes = vec![
            0, 0, 0, 0, // base
            0, 0, 0x03, 0xE8, // base
            0, 0, 0x01, 0x00, // num bits
            0b_10000000, 0b_00000000, 0b_00000000, 0b_00000000, 
            0b_00000000, 0b_00000000, 0b_00000000, 0b_00000000, 
            0b_00000000, 0b_00000000, 0b_00000000, 0b_00000000,
            0b_00000000, 0b_00000000, 0b_00000000, 0b_00000000,
            0b_00000000, 0b_00000000, 0b_00000000, 0b_00000000,
            0b_00000000, 0b_00000000, 0b_00000000, 0b_00000000,
            0b_00000000, 0b_00000000, 0b_00000000, 0b_00000000,
            0b_00000000, 0b_00000000, 0b_00000000, 0b_00000001,
        ];
        let result = SequenceNumberSet::deserialize(&bytes, Endianness::BigEndian).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn deserialize_sequence_number_set_max_bitmaps_little_endian() {
        let expected = SequenceNumberSet{
            base: SequenceNumber(1000),
            set: [SequenceNumber(1000), SequenceNumber(1255)].iter().cloned().collect()
        };
        let bytes = vec![
            0, 0, 0, 0, // base
            0xE8, 0x03, 0, 0, // base
            0x00, 0x01, 0, 0, // num bits
            0b_00000000, 0b_00000000, 0b_00000000, 0b_10000000, 
            0b_00000000, 0b_00000000, 0b_00000000, 0b_00000000, 
            0b_00000000, 0b_00000000, 0b_00000000, 0b_00000000,
            0b_00000000, 0b_00000000, 0b_00000000, 0b_00000000,
            0b_00000000, 0b_00000000, 0b_00000000, 0b_00000000,
            0b_00000000, 0b_00000000, 0b_00000000, 0b_00000000,
            0b_00000000, 0b_00000000, 0b_00000000, 0b_00000000,
            0b_00000001, 0b_00000000, 0b_00000000, 0b_00000000, 
        ];
        let result = SequenceNumberSet::deserialize(&bytes, Endianness::LittleEndian).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn deserialize_sequence_number_set_as_of_example_in_standard_be() {
        // Example in standard "1234:/12:00110"
        let bytes = [
            0x00, 0x00, 0x00, 0x00, 
            0x00, 0x00, 0x04, 0xD2, 
            0x00, 0x00, 0x00, 0x0C, 
            0x30, 0x00, 0x00, 0x00, 
        ];
        let set = SequenceNumberSet::deserialize(&bytes, Endianness::BigEndian).unwrap().set;
        assert!(!set.contains(&SequenceNumber(1234)));
        assert!(!set.contains(&SequenceNumber(1235)));
        assert!(set.contains(&SequenceNumber(1236)));
        assert!(set.contains(&SequenceNumber(1237)));
        for seq_num in 1238..1245 {
            assert!(!set.contains(&SequenceNumber(seq_num)));
        }
    }
    
    #[test]
    fn deserialize_sequence_number_set_as_of_example_in_standard_le() {
        // Example in standard "1234:/12:00110"
        let bytes = [
            0x00, 0x00, 0x00, 0x00, 
            0xD2, 0x04, 0x00, 0x00, 
            0x0C, 0x00, 0x00, 0x00, 
            0x00, 0x00, 0x00, 0x30, 
        ];
        let set = SequenceNumberSet::deserialize(&bytes, Endianness::LittleEndian).unwrap().set;
        assert!(!set.contains(&SequenceNumber(1234)));
        assert!(!set.contains(&SequenceNumber(1235)));
        assert!(set.contains(&SequenceNumber(1236)));
        assert!(set.contains(&SequenceNumber(1237)));
        for seq_num in 1238..1245 {
            assert!(!set.contains(&SequenceNumber(seq_num)));
        }
    }
        
    
    #[test]
    fn serialize_sequence_number_set() {
        let set = SequenceNumberSet{
            base: SequenceNumber(3),
            set: [SequenceNumber(3), SequenceNumber(4)].iter().cloned().collect()
        };
        let mut writer = Vec::new();
        set.serialize(&mut writer, Endianness::BigEndian).unwrap();
        let expected = vec![
            0, 0, 0, 0, // base
            0, 0, 0, 3, // base
            0, 0, 0, 2, // num bits
            0b_11000000, 0b_00000000, 0b_00000000, 0b_00000000, 
        ];
        assert_eq!(expected, writer);
    
    
        let set = SequenceNumberSet{
            base: SequenceNumber(1),
            set: [SequenceNumber(3), SequenceNumber(4)].iter().cloned().collect()
        };
        let mut writer = Vec::new();
        set.serialize(&mut writer, Endianness::LittleEndian).unwrap();
        let expected = vec![
            0, 0, 0, 0, // base
            1, 0, 0, 0, // base
            4, 0, 0, 0, // num bits
            0b_00000000, 0b_00000000, 0b_00000000, 0b_00110000, 
        ];
        assert_eq!(expected, writer);
    
        let mut writer = Vec::new();
        set.serialize(&mut writer, Endianness::BigEndian).unwrap();
        let expected = vec![
            0, 0, 0, 0, // base
            0, 0, 0, 1, // base
            0, 0, 0, 4, // num bits
            0b_00110000, 0b_00000000, 0b_00000000, 0b_00000000,  
        ];
        assert_eq!(expected, writer);
    
    
        let set = SequenceNumberSet{
            base: SequenceNumber(1000),
            set: [SequenceNumber(1001), SequenceNumber(1003), SequenceNumber(1032), SequenceNumber(1033)].iter().cloned().collect()
        };
        let mut writer = Vec::new();
        set.serialize(&mut writer, Endianness::BigEndian).unwrap();
        let expected = vec![
            0, 0, 0, 0, // base
            0, 0, 3, 232, // base
            0, 0, 0, 34, // num bits
            0b_01010000, 0b_00000000, 0b_00000000, 0b_00000000, 
            0b_11000000, 0b_00000000, 0b_00000000, 0b_00000000, 
        ];
        assert_eq!(expected, writer);
    }

        

    // //////////////////////// FragmentNumber Tests ///////////////////////
    
    #[test]
    fn serialize_fragment_number() {
        let fragment_number = FragmentNumber(100);
        let expected = vec![
            100, 0, 0, 0,
        ];
        let mut writer = Vec::new();
        fragment_number.serialize(&mut writer, Endianness::LittleEndian).unwrap();
        assert_eq!(expected, writer);
    }

    #[test]
    fn deserialize_fragment_number() {
        let expected = FragmentNumber(100);
        let bytes = vec![
            100, 0, 0, 0,
        ];
        let result = FragmentNumber::deserialize(&bytes, Endianness::LittleEndian).unwrap();
        assert_eq!(expected, result);
    }


    // /////////////////////// FragmentNumberSet Tests ////////////////////////

    #[test]
    fn fragment_number_set_constructor() {
        let expected = FragmentNumberSet{
            base: FragmentNumber(1001),
            set:  [FragmentNumber(1001), FragmentNumber(1003)].iter().cloned().collect(),
        };
        let result = FragmentNumberSet::new([FragmentNumber(1001), FragmentNumber(1003)].iter().cloned().collect());
        assert_eq!(expected, result);
    }

    #[test]
    fn deserialize_fragment_number_set() {
        let expected = FragmentNumberSet{
            base: FragmentNumber(3),
            set: [FragmentNumber(3), FragmentNumber(4)].iter().cloned().collect()
        };
        let bytes = vec![
            3, 0, 0, 0, // base
            2, 0, 0, 0, // num bits
            0b_00000000, 0b_00000000, 0b_00000000, 0b_11000000, 
        ];
        let result = FragmentNumberSet::deserialize(&bytes, Endianness::LittleEndian).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn serialize_fragment_number_set() {
        let set = FragmentNumberSet{
            base: FragmentNumber(3),
            set: [FragmentNumber(3), FragmentNumber(4)].iter().cloned().collect()
        };
        let mut writer = Vec::new();
        set.serialize(&mut writer, Endianness::BigEndian).unwrap();
        let expected = vec![
            0, 0, 0, 3, // base
            0, 0, 0, 2, // num bits
            0b_11000000, 0b_00000000, 0b_00000000, 0b_00000000, 
        ];
        assert_eq!(expected, writer);
    }



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
    
    #[test]
    fn test_parameter_round_up_to_multiples_of_four() {
        let e= Endianness::LittleEndian;
        assert_eq!(0, Parameter::new(&VendorTest0([]), e).length);
        assert_eq!(4, Parameter::new(&VendorTest1([b'X']), e).length);
        assert_eq!(4, Parameter::new(&VendorTest3([b'X'; 3]), e).length);
        assert_eq!(4, Parameter::new(&VendorTest4([b'X'; 4]), e).length);
        assert_eq!(8, Parameter::new(&VendorTest5([b'X'; 5]), e).length);
    }

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
        let parameter_list = ParameterList{parameter: vec![Rc::new(key_hash), Rc::new(status_info)]};
    
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



    // /////////////////////// LocatorList Tests ////////////////////////
      
    #[test]
    fn serialize_locator_list() {
        let address = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let locator_list = LocatorList(vec![
            Locator::new(100, 200, address),
            Locator::new(101, 201, address),
        ]);
        let expected = vec![
            2, 0, 0, 0, // numLocators
            100, 0, 0, 0, // Locator 1: kind
            200, 0, 0, 0, // Locator 1: port
            1,  2,  3,  4, // Locator 1: address
            5,  6,  7,  8, // Locator 1: address
            9, 10, 11, 12, // Locator 1: address
            13, 14, 15, 16, // Locator 1: address
            101, 0, 0, 0, // Locator 2: kind
            201, 0, 0, 0, // Locator 2: port
            1,  2,  3,  4, // Locator 2: address
            5,  6,  7,  8, // Locator 2: address
            9, 10, 11, 12, // Locator 2: address
            13, 14, 15, 16, // Locator 2: address
        ];
        let mut writer = Vec::new();
        locator_list.serialize(&mut writer, Endianness::LittleEndian).unwrap();
        assert_eq!(expected, writer);
    }

    
    #[test]
    fn deserialize_locator_list() {
        let bytes = vec![
            2, 0, 0, 0,   // numLocators
            100, 0, 0, 0, // Locator 1: kind
            200, 0, 0, 0, // Locator 1: port
            1,  2,  3,  4, // Locator 1: address
            5,  6,  7,  8, // Locator 1: address
            9, 10, 11, 12, // Locator 1: address
            13, 14, 15, 16, // Locator 1: address
            101, 0, 0, 0, // Locator 2: kind
            201, 0, 0, 0, // Locator 2: port
            1,  2,  3,  4, // Locator 2: address
            5,  6,  7,  8, // Locator 2: address
            9, 10, 11, 12, // Locator 2: address
            13, 14, 15, 16, // Locator 2: address
        ];
        let address = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let expected = LocatorList(vec![
            Locator::new(100, 200, address),
            Locator::new(101, 201, address),
        ]);
        let result = LocatorList::deserialize(&bytes, Endianness::LittleEndian).unwrap();
        assert_eq!(expected, result);
    }
}
