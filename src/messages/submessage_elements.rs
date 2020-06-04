/// 
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// 8.3.5 RTPS SubmessageElements
///  

use std::ops::Index;
use std::slice::Iter;
use std::collections::BTreeSet;
use std::io::Write;
use std::convert::TryInto;
use crate::types::Locator;
use crate::types_primitives::{Long, ULong, };
use crate::serdes::{RtpsSerialize, RtpsDeserialize, EndianessFlag, RtpsSerdesResult, PrimitiveSerdes, SizeCheckers, SizeSerializer};
use crate::types::{SequenceNumber, };

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
    pub fn new(set: BTreeSet<SequenceNumber>) -> Self { 
        let base = *set.iter().next().unwrap_or(&SequenceNumber(0));
        Self {base, set } 
    }
}

impl RtpsSerialize for SequenceNumberSet {
    fn serialize(&self, writer: &mut impl Write, endianness: EndianessFlag) -> RtpsSerdesResult<()> {
        let num_bits = if self.set.is_empty() {
            0 
        } else {
            (self.set.iter().last().unwrap().0 - self.base.0) as usize + 1
        };
        let m = (num_bits + 31) / 32;
        let mut bitmaps = vec![0_u32; m];
        self.base.serialize(writer, endianness)?;
        ULong::from(num_bits).serialize(writer, endianness)?;
        for seq_num in &self.set {
            let delta_n = (seq_num.0 - self.base.0) as usize;
            let bitmap_i = delta_n / 32;
            let bitmask = 1 << (31 - delta_n % 32);
            bitmaps[bitmap_i] |= bitmask;
        };
        for bitmap in bitmaps {
            ULong(bitmap).serialize(writer, endianness)?;
        }
        Ok(())
    }
}

impl RtpsDeserialize for SequenceNumberSet {
    fn deserialize(bytes: &[u8], endianness: EndianessFlag) -> RtpsSerdesResult<Self> {
        let base = SequenceNumber::deserialize(&bytes[0..8], endianness)?;
        let num_bits = ULong::deserialize(&bytes[8..12], endianness)?.0 as usize;

        // Get bitmaps from "long"s that follow the numBits field in the message
        // Note that the amount of bitmaps that are included in the message are 
        // determined by the number of bits (32 max per bitmap, and a max of 256 in 
        // total which means max 8 bitmap "long"s)
        let m = (num_bits + 31) / 32;        
        let mut bitmaps = Vec::with_capacity(m);
        for i in 0..m {
            let index_of_byte_current_bitmap = 12 + i * 4;
            bitmaps.push(Long::deserialize(&bytes[index_of_byte_current_bitmap..], endianness)?.0);
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
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: EndianessFlag) -> RtpsSerdesResult<()> {
        self.0.serialize(writer, endianness)?;
        Ok(())
    }    
}

impl RtpsDeserialize for FragmentNumber {
    fn deserialize(bytes: &[u8], endianness: EndianessFlag) -> RtpsSerdesResult<Self> {
        Ok(Self(ULong::deserialize(&bytes, endianness)?))
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
        let base = *set.iter().next().unwrap_or(&FragmentNumber(ULong(0)));
        Self {base, set } 
    }
}

impl RtpsSerialize for FragmentNumberSet {
    fn serialize(&self, writer: &mut impl Write, endianness: EndianessFlag) -> RtpsSerdesResult<()> {
        let num_bits = if self.set.is_empty() {
            0 
        } else {
            (usize::from(self.set.iter().last().unwrap().0) - usize::from(self.base.0)) + 1
        };
        let m = (num_bits + 31) / 32;
        let mut bitmaps = vec![0_u32; m];
        self.base.serialize(writer, endianness)?;
        ULong::from(num_bits).serialize(writer, endianness)?;
        for seq_num in &self.set {
            let delta_n = (usize::from(seq_num.0) - usize::from(self.base.0)) as usize;
            let bitmap_i = delta_n / 32;
            let bitmask = 1 << (31 - delta_n % 32);
            bitmaps[bitmap_i] |= bitmask;
        };
        for bitmap in bitmaps {
            ULong(bitmap).serialize(writer, endianness)?;
        }
        Ok(())
    }
}
impl RtpsDeserialize for FragmentNumberSet {
    fn deserialize(bytes: &[u8], endianness: EndianessFlag) -> RtpsSerdesResult<Self> {
        let base = FragmentNumber::deserialize(&bytes[0..4], endianness)?;
        let num_bits = ULong::deserialize(&bytes[4..8], endianness)?.0 as usize;

        // Get bitmaps from "long"s that follow the numBits field in the message
        // Note that the amount of bitmaps that are included in the message are 
        // determined by the number of bits (32 max per bitmap, and a max of 256 in 
        // total which means max 8 bitmap "long"s)
        let m = (num_bits + 31) / 32;        
        let mut bitmaps = Vec::with_capacity(m);
        for i in 0..m {
            let index_of_byte_current_bitmap = 8 + i * 4;
            bitmaps.push(Long::deserialize(&bytes[index_of_byte_current_bitmap..], endianness)?.0);
        };
        // Interpet the bitmaps and insert the sequence numbers if they are encode in the bitmaps
        let mut set = BTreeSet::new(); 
        for delta_n in 0..num_bits {
            let bitmask = 1 << (31 - delta_n % 32);
            if  bitmaps[delta_n / 32] & bitmask == bitmask {               
                let frag_num = FragmentNumber(ULong::from(delta_n + usize::from(base.0)));
                set.insert(frag_num);
            }
        }
        Ok(Self {base, set})
    }    
}



//  ///////////   ParameterList

pub trait Parameter
where
    Self: std::marker::Sized
{
    fn new_from(parameter_id: u16, value: &[u8]) -> Option<Self>;

    fn parameter_id(&self) -> u16;

    fn value(&self) -> &[u8];
}

#[derive(Hash, Clone, Debug, PartialEq, Eq)]
pub struct ParameterList<T: Parameter>(Vec<T>);

impl<T: Parameter> ParameterList<T> {
    const PID_PAD: u16 = 0x0000;
    const PID_SENTINEL: u16 = 0x0001;

    pub fn new() -> Self {
        ParameterList(Vec::new())
    }

    pub fn new_from_vec(value: Vec<T>) -> Self {
        ParameterList(value)
    }

    pub fn iter(&self) -> Iter<'_,T>{
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn push(&mut self, value: T) {
        self.0.push(value);
    }

    pub fn find_parameter(&self, id: u16) -> Option<&T> {
        self.0.iter().find(|&value| value.parameter_id() == id)
    }

    pub fn is_valid(&self) -> bool {
        todo!()
    }
}

impl<T> Index<usize> for ParameterList<T> 
where
    T: Parameter
{
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl<T> RtpsSerialize for ParameterList<T> 
where
    T: RtpsSerialize + Parameter,
{
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: EndianessFlag) -> RtpsSerdesResult<()> {
        for item in self.iter() {
            item.serialize(writer, endianness)?;
        }

        writer.write(&PrimitiveSerdes::serialize_u16(Self::PID_SENTINEL, endianness))?;
        writer.write(&[0,0])?;

        Ok(())
    }
}

impl<T> RtpsSerialize for T 
where
    T: Parameter
{
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: EndianessFlag) -> RtpsSerdesResult<()> {
        let mut size_serializer =  SizeSerializer::new();

        writer.write(&PrimitiveSerdes::serialize_u16(self.parameter_id(), endianness))?;
        
        //TODO: The size needs to be rounded to multiples of 4 and include padding
        size_serializer.write(self.value())?;
        writer.write(&PrimitiveSerdes::serialize_u16(size_serializer.get_size() as u16, endianness))?;

        writer.write(self.value())?;

        Ok(())
    }
}

impl<T: Parameter> RtpsDeserialize for ParameterList<T> 
{
    fn deserialize(bytes: &[u8], endianness: EndianessFlag) -> RtpsSerdesResult<Self> {
        SizeCheckers::check_size_bigger_equal_than(bytes, 4)?;

        let mut parameter_start_index: usize = 0;
        let mut parameter_list = ParameterList::<T>::new();

        loop {
            let parameter_id_first_index = parameter_start_index + 0;
            let parameter_id_last_index = parameter_start_index + 1;
            let parameter_size_first_index = parameter_start_index + 2;
            let parameter_size_last_index = parameter_start_index + 3;
 
            let parameter_id_u16 = PrimitiveSerdes::deserialize_u16(bytes[parameter_id_first_index..=parameter_id_last_index].try_into()?, endianness);
            let parameter_size = PrimitiveSerdes::deserialize_u16(bytes[parameter_size_first_index..=parameter_size_last_index].try_into()?, endianness) as usize;

            if parameter_id_u16 == Self::PID_SENTINEL {
                break;
            }

            let parameter_value_first_index = parameter_start_index + 4;
            let parameter_value_last_index = parameter_value_first_index + parameter_size;

            SizeCheckers::check_size_bigger_equal_than(bytes,parameter_value_last_index)?;

            // For the new_from do a non_inclusive retrieval of the bytes
            if let Some(parameter) = T::new_from(parameter_id_u16, &bytes[parameter_value_first_index..parameter_value_last_index]) {
                parameter_list.push(parameter);
            }

            parameter_start_index = parameter_value_last_index;

        }

        Ok(parameter_list)
    }
}



//  ///////////   Count
// same as in super::types


//  ///////////   LocatorList

#[derive(Debug, PartialEq)]
pub struct LocatorList(pub Vec<Locator>);

impl RtpsSerialize for LocatorList {
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: EndianessFlag) -> RtpsSerdesResult<()> {
        let num_locators = ULong::from(self.0.len());
        num_locators.serialize(writer, endianness)?;
        for locator in &self.0 {
            locator.serialize(writer, endianness)?;
        };
        Ok(())
    }
}

impl RtpsDeserialize for LocatorList {
    fn deserialize(bytes: &[u8], endianness: EndianessFlag) -> RtpsSerdesResult<Self> {
        let size = bytes.len();
        let num_locators = ULong::deserialize(&bytes[0..4], endianness)?;
        let mut locators = Vec::<Locator>::new();
        let mut index = 4;
        while index < size && locators.len() < usize::from(num_locators) {
            let locator = Locator::deserialize( &bytes[index..], endianness)?;
            index += locator.octets();
            locators.push(locator);
        };
        Ok(Self(locators))
    }
}



//  ///////////   SerializedData
// todo

//  ///////////   SerializedDataFragment
// todo

//  ///////////   GroupDigest
// todo









#[cfg(test)]
mod tests {
    use super::*;

    ///////////////////////// SequenceNumberSet Tests ////////////////////////

    #[test]
    fn sequence_number_set_constructor() {
        let expected = SequenceNumberSet{
            base: SequenceNumber(1001),
            set:  [SequenceNumber(1001), SequenceNumber(1003)].iter().cloned().collect(),
        };
        let result = SequenceNumberSet::new([SequenceNumber(1001), SequenceNumber(1003)].iter().cloned().collect());
        assert_eq!(expected, result);
    }

    #[test]
    fn sequence_number_set_constructor_empty_set() {        
        let expected = SequenceNumberSet{
            base: SequenceNumber(0),
            set:  [].iter().cloned().collect(),
        };
        let result = SequenceNumberSet::new([].iter().cloned().collect());
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
        let result = SequenceNumberSet::deserialize(&bytes, EndianessFlag::BigEndian).unwrap();
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
        let result = SequenceNumberSet::deserialize(&bytes, EndianessFlag::BigEndian).unwrap();
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
        let result = SequenceNumberSet::deserialize(&bytes, EndianessFlag::LittleEndian).unwrap();
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
        let result = SequenceNumberSet::deserialize(&bytes, EndianessFlag::BigEndian).unwrap();
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
        let result = SequenceNumberSet::deserialize(&bytes, EndianessFlag::BigEndian).unwrap();
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
        let result = SequenceNumberSet::deserialize(&bytes, EndianessFlag::LittleEndian).unwrap();
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
        let set = SequenceNumberSet::deserialize(&bytes, EndianessFlag::BigEndian).unwrap().set;
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
        let set = SequenceNumberSet::deserialize(&bytes, EndianessFlag::LittleEndian).unwrap().set;
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
        set.serialize(&mut writer, EndianessFlag::BigEndian).unwrap();
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
        set.serialize(&mut writer, EndianessFlag::LittleEndian).unwrap();
        let expected = vec![
            0, 0, 0, 0, // base
            1, 0, 0, 0, // base
            4, 0, 0, 0, // num bits
            0b_00000000, 0b_00000000, 0b_00000000, 0b_00110000, 
        ];
        assert_eq!(expected, writer);
    
        let mut writer = Vec::new();
        set.serialize(&mut writer, EndianessFlag::BigEndian).unwrap();
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
        set.serialize(&mut writer, EndianessFlag::BigEndian).unwrap();
        let expected = vec![
            0, 0, 0, 0, // base
            0, 0, 3, 232, // base
            0, 0, 0, 34, // num bits
            0b_01010000, 0b_00000000, 0b_00000000, 0b_00000000, 
            0b_11000000, 0b_00000000, 0b_00000000, 0b_00000000, 
        ];
        assert_eq!(expected, writer);
    }

        

    ////////////////////////// FragmentNumber Tests ///////////////////////
    
    #[test]
    fn serialize_fragment_number() {
        let fragment_number = FragmentNumber(ULong(100));
        let expected = vec![
            100, 0, 0, 0,
        ];
        let mut writer = Vec::new();
        fragment_number.serialize(&mut writer, EndianessFlag::LittleEndian).unwrap();
        assert_eq!(expected, writer);
    }

    #[test]
    fn deserialize_fragment_number() {
        let expected = FragmentNumber(ULong(100));
        let bytes = vec![
            100, 0, 0, 0,
        ];
        let result = FragmentNumber::deserialize(&bytes, EndianessFlag::LittleEndian).unwrap();
        assert_eq!(expected, result);
    }


    ///////////////////////// FragmentNumberSet Tests ////////////////////////

    #[test]
    fn fragment_number_set_constructor() {
        let expected = FragmentNumberSet{
            base: FragmentNumber(ULong(1001)),
            set:  [FragmentNumber(ULong(1001)), FragmentNumber(ULong(1003))].iter().cloned().collect(),
        };
        let result = FragmentNumberSet::new([FragmentNumber(ULong(1001)), FragmentNumber(ULong(1003))].iter().cloned().collect());
        assert_eq!(expected, result);
    }

    #[test]
    fn deserialize_fragment_number_set() {
        let expected = FragmentNumberSet{
            base: FragmentNumber(ULong(3)),
            set: [FragmentNumber(ULong(3)), FragmentNumber(ULong(4))].iter().cloned().collect()
        };
        let bytes = vec![
            3, 0, 0, 0, // base
            2, 0, 0, 0, // num bits
            0b_00000000, 0b_00000000, 0b_00000000, 0b_11000000, 
        ];
        let result = FragmentNumberSet::deserialize(&bytes, EndianessFlag::LittleEndian).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn serialize_fragment_number_set() {
        let set = FragmentNumberSet{
            base: FragmentNumber(ULong(3)),
            set: [FragmentNumber(ULong(3)), FragmentNumber(ULong(4))].iter().cloned().collect()
        };
        let mut writer = Vec::new();
        set.serialize(&mut writer, EndianessFlag::BigEndian).unwrap();
        let expected = vec![
            0, 0, 0, 3, // base
            0, 0, 0, 2, // num bits
            0b_11000000, 0b_00000000, 0b_00000000, 0b_00000000, 
        ];
        assert_eq!(expected, writer);
    }

    ///////////////////////// Parameter List Tests ////////////////////////
    #[test]
    fn test_paramter_list_find() {
        #[derive(Debug,PartialEq)]
        enum SampleParameter {
            Parameter1,
            Parameter2,
        }

        impl Parameter for SampleParameter {
            fn new_from(_parameter_id: u16, _value: &[u8]) -> Option<Self> {
                unimplemented!()
            }

            fn parameter_id(&self) -> u16 {
                match self {
                    SampleParameter::Parameter1 => 0x0070,
                    SampleParameter::Parameter2 => 0x0071,
                }
            }

            fn value(&self) -> &[u8] {
                unimplemented!()
            }
        }

        let complete_list = ParameterList(vec![SampleParameter::Parameter1, SampleParameter::Parameter2]);
        assert_eq!(complete_list.find_parameter(SampleParameter::Parameter1.parameter_id()), Some(&SampleParameter::Parameter1));

        let partial_list = ParameterList(vec![SampleParameter::Parameter1]);
        assert_eq!(partial_list.find_parameter(SampleParameter::Parameter2.parameter_id()), None);
    }



    ///////////////////////// LocatorList Tests ////////////////////////
      
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
        locator_list.serialize(&mut writer, EndianessFlag::LittleEndian).unwrap();
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
        let result = LocatorList::deserialize(&bytes, EndianessFlag::LittleEndian).unwrap();
        assert_eq!(expected, result);
    }
}