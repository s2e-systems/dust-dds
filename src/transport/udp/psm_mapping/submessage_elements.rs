use std::convert::TryInto;

use crate::messages::submessages::submessage_elements::{Long, Short, ULong, UShort};

use super::{SizeCheck, TransportEndianness, UdpPsmMappingResult};

pub fn serialize_long(
    long: &Long,
    writer: &mut impl std::io::Write,
    endianness: TransportEndianness,
) -> UdpPsmMappingResult<()> {
    let value = match endianness {
        TransportEndianness::BigEndian => long.0.to_be_bytes(),
        TransportEndianness::LittleEndian => long.0.to_le_bytes(),
    };
    writer.write(&value)?;
    Ok(())
}

pub fn deserialize_long(bytes: &[u8], endianness: TransportEndianness) -> UdpPsmMappingResult<Long> {
    bytes.check_size_equal(4)?;

    let value = match endianness {
        TransportEndianness::BigEndian => Long(i32::from_be_bytes(bytes[0..4].try_into()?)),
        TransportEndianness::LittleEndian => Long(i32::from_le_bytes(bytes[0..4].try_into()?)),
    };
    Ok(value)
}

pub fn serialize_ulong(
    ulong: &ULong,
    writer: &mut impl std::io::Write,
    endianness: TransportEndianness,
) -> UdpPsmMappingResult<()> {
    let value = match endianness {
        TransportEndianness::BigEndian => ulong.0.to_be_bytes(),
        TransportEndianness::LittleEndian => ulong.0.to_le_bytes(),
    };
    writer.write(&value)?;
    Ok(())
}

pub fn deserialize_ulong(bytes: &[u8], endianness: TransportEndianness) -> UdpPsmMappingResult<ULong> {
    bytes.check_size_equal(4)?;

    let value = match endianness {
        TransportEndianness::BigEndian => ULong(u32::from_be_bytes(bytes[0..4].try_into()?)),
        TransportEndianness::LittleEndian => ULong(u32::from_le_bytes(bytes[0..4].try_into()?)),
    };
    Ok(value)
}

pub fn serialize_short(
    short: &Short,
    writer: &mut impl std::io::Write,
    endianness: TransportEndianness,
) -> UdpPsmMappingResult<()> {
    let value = match endianness {
        TransportEndianness::BigEndian => short.0.to_be_bytes(),
        TransportEndianness::LittleEndian => short.0.to_le_bytes(),
    };
    writer.write(&value)?;
    Ok(())
}

pub fn deserialize_short(bytes: &[u8], endianness: TransportEndianness) -> UdpPsmMappingResult<Short> {
    bytes.check_size_equal(2)?;

    let value = match endianness {
        TransportEndianness::BigEndian => Short(i16::from_be_bytes(bytes[0..2].try_into()?)),
        TransportEndianness::LittleEndian => Short(i16::from_le_bytes(bytes[0..2].try_into()?)),
    };
    Ok(value)
}

pub fn serialize_ushort(
    ushort: &UShort,
    writer: &mut impl std::io::Write,
    endianness: TransportEndianness,
) -> UdpPsmMappingResult<()> {
    let value = match endianness {
        TransportEndianness::BigEndian => ushort.0.to_be_bytes(),
        TransportEndianness::LittleEndian => ushort.0.to_le_bytes(),
    };
    writer.write(&value)?;
    Ok(())
}

pub fn deserialize_ushort(bytes: &[u8], endianness: TransportEndianness) -> UdpPsmMappingResult<UShort> {
    bytes.check_size_equal(2)?;

    let value = match endianness {
        TransportEndianness::BigEndian => UShort(u16::from_be_bytes(bytes[0..2].try_into()?)),
        TransportEndianness::LittleEndian => UShort(u16::from_le_bytes(bytes[0..2].try_into()?)),
    };
    Ok(value)
}

// impl SubmessageElement for GuidPrefix {
//     fn serialize(&self, writer: &mut impl std::io::Write, _endianness: Endianness) -> RtpsSerdesResult<()> {
//         writer.write(&self.0)?;
//         Ok(())
//     }

//     fn deserialize(bytes: &[u8], _endianness: Endianness) -> RtpsSerdesResult<Self> {
//         bytes.check_size_equal(12)?;
//         Ok(GuidPrefix(bytes[0..12].try_into()?))
//     }    
// }

// impl SubmessageElement for EntityId {
//     fn serialize(&self, writer: &mut impl std::io::Write, _endianness: Endianness) -> RtpsSerdesResult<()>{
//         writer.write(&self.0.entity_key())?;
//         writer.write(&[self.0.entity_kind() as u8])?;
//         Ok(())
//     }

//     fn deserialize(bytes: &[u8], _endianness: Endianness) -> RtpsSerdesResult<Self> {
//         bytes.check_size_equal( 4)?;
//         let entity_key = bytes[0..3].try_into()?;
//         let entity_kind = num::FromPrimitive::from_u8(bytes[3]).ok_or(RtpsSerdesError::InvalidEnumRepresentation)?;
//         Ok(EntityId(types::EntityId::new(entity_key, entity_kind)))
//     }
// }

// impl SubmessageElement for VendorId {
//     fn serialize(&self, writer: &mut impl std::io::Write, _endianness: Endianness) -> RtpsSerdesResult<()> {
//         writer.write(&self.0)?;
//         Ok(())
//     }

//     fn deserialize(bytes: &[u8], _endianness: Endianness) -> RtpsSerdesResult<Self> {
//         bytes.check_size_equal(2)?;
//         Ok(VendorId(bytes[0..2].try_into()?))
//     }
// }

// impl SubmessageElement for ProtocolVersion {
//     fn serialize(&self, writer: &mut impl std::io::Write, _endianness: Endianness) -> RtpsSerdesResult<()> {
//         writer.write(&[self.0.major])?;
//         writer.write(&[self.0.minor])?;
//         Ok(())
//     }

//     fn deserialize(bytes: &[u8], _endianness: Endianness) -> RtpsSerdesResult<Self> {
//         bytes.check_size_equal(2)?;
//         let major = bytes[0];
//         let minor = bytes[1];
//         Ok(Self(types::ProtocolVersion{major, minor}))
//     }
// }

// impl SubmessageElement for SequenceNumber {
//     fn serialize(&self, writer: &mut impl std::io::Write, endianness: Endianness) -> RtpsSerdesResult<()>{
//         let msb = Long((self.0 >> 32) as i32);
//         let lsb = ULong((self.0 & 0x0000_0000_FFFF_FFFF) as u32);
//         msb.serialize(writer, endianness)?;
//         lsb.serialize(writer, endianness)?;
//         Ok(())
//     }

//     fn deserialize(bytes: &[u8], endianness: Endianness) -> RtpsSerdesResult<Self> {
//         bytes.check_size_equal(8)?;

//         let msb = Long::deserialize(&bytes[0..4], endianness)?;
//         let lsb = ULong::deserialize(&bytes[4..8], endianness)?;

//         let sequence_number = ((msb.0 as i64) << 32) + lsb.0 as i64;

//         Ok(SequenceNumber(sequence_number))
//     }
// }

// impl SubmessageElement for SequenceNumberSet {
//     fn serialize(&self, writer: &mut impl Write, endianness: Endianness) -> RtpsSerdesResult<()> {
//         let num_bits = ULong(if self.set.is_empty() {
//             0 
//         } else {
//             (self.set.iter().last().unwrap() - self.base) as usize + 1
//         } as u32);
//         let m = ((num_bits.0 + 31) / 32) as usize;
//         let mut bitmaps = vec![0_i32; m];
//         SequenceNumber(self.base).serialize(writer, endianness)?;
//         num_bits.serialize(writer, endianness)?;
//         for seq_num in &self.set {
//             let delta_n = (seq_num - self.base) as usize;
//             let bitmap_i = delta_n / 32;
//             let bitmask = 1 << (31 - delta_n % 32);
//             bitmaps[bitmap_i] |= bitmask;
//         };
//         for bitmap in bitmaps {
//             Long(bitmap).serialize(writer, endianness)?;
//         }
//         Ok(())
//     }

//     fn deserialize(bytes: &[u8], endianness: Endianness) -> RtpsSerdesResult<Self> {
//         bytes.check_size_bigger_equal_than(12)?;

//         let base = SequenceNumber::deserialize(&bytes[0..8], endianness)?.0;
//         let num_bits = ULong::deserialize(&bytes[8..12], endianness)?;

//         // Get bitmaps from "long"s that follow the numBits field in the message
//         // Note that the amount of bitmaps that are included in the message are 
//         // determined by the number of bits (32 max per bitmap, and a max of 256 in 
//         // total which means max 8 bitmap "long"s)
//         let m = ((num_bits.0 as usize) + 31) / 32;        
//         let mut bitmaps = Vec::with_capacity(m);
//         for i in 0..m {
//             let index_of_byte_current_bitmap = 12 + i * 4;
//             bytes.check_size_bigger_equal_than(index_of_byte_current_bitmap+4)?;
//             bitmaps.push(Long::deserialize(&bytes[index_of_byte_current_bitmap..index_of_byte_current_bitmap+4], endianness)?.0);
//         };
//         // Interpet the bitmaps and insert the sequence numbers if they are encode in the bitmaps
//         let mut set = BTreeSet::new(); 
//         for delta_n in 0..num_bits.0 {
//             let bitmask = 1 << (31 - delta_n % 32);
//             if  bitmaps[delta_n as usize / 32] & bitmask == bitmask {               
//                 let seq_num = delta_n as i64 + base;
//                 set.insert(seq_num);
//             }
//         }
//         Ok(Self {base, set})
//     }    
// }


// impl SubmessageElement for FragmentNumber {
//     fn serialize(&self, writer: &mut impl std::io::Write, endianness: Endianness) -> RtpsSerdesResult<()> {
//         ULong(self.0).serialize(writer, endianness)?;
//         Ok(())
//     }    

//     fn deserialize(bytes: &[u8], endianness: Endianness) -> RtpsSerdesResult<Self> {
//         Ok(Self(ULong::deserialize(bytes, endianness)?.0))
//     }    
// }


// impl SubmessageElement for FragmentNumberSet {
//     fn serialize(&self, writer: &mut impl Write, endianness: Endianness) -> RtpsSerdesResult<()> {
//         let num_bits = ULong(if self.set.is_empty() {
//             0 
//         } else {
//             self.set.iter().last().unwrap().0 - self.base.0 + 1
//         });
//         let m = ((num_bits.0 + 31) / 32) as usize;
//         let mut bitmaps = vec![0_i32; m];
//         self.base.serialize(writer, endianness)?;
//         num_bits.serialize(writer, endianness)?;
//         for seq_num in &self.set {
//             let delta_n = (seq_num.0 - self.base.0) as usize;
//             let bitmap_i = delta_n / 32;
//             let bitmask = 1 << (31 - delta_n % 32);
//             bitmaps[bitmap_i] |= bitmask;
//         };
//         for bitmap in bitmaps {
//             Long(bitmap).serialize(writer, endianness)?;
//         }
//         Ok(())
//     }

//     fn deserialize(bytes: &[u8], endianness: Endianness) -> RtpsSerdesResult<Self> {
//         bytes.check_size_bigger_equal_than(8)?;
//         let base = FragmentNumber::deserialize(&bytes[0..4], endianness)?;
//         let num_bits = ULong::deserialize(&bytes[4..8], endianness)?.0 as usize;

//         // Get bitmaps from "long"s that follow the numBits field in the message
//         // Note that the amount of bitmaps that are included in the message are 
//         // determined by the number of bits (32 max per bitmap, and a max of 256 in 
//         // total which means max 8 bitmap "long"s)
//         let m = (num_bits + 31) / 32;        
//         let mut bitmaps = Vec::with_capacity(m);
//         for i in 0..m {
//             let index_of_byte_current_bitmap = 8 + i * 4;
//             bytes.check_size_bigger_equal_than(index_of_byte_current_bitmap+4)?;
//             bitmaps.push(Long::deserialize(&bytes[index_of_byte_current_bitmap..index_of_byte_current_bitmap+4], endianness)?.0);
//         };
//         // Interpet the bitmaps and insert the sequence numbers if they are encode in the bitmaps
//         let mut set = BTreeSet::new(); 
//         for delta_n in 0..num_bits {
//             let bitmask = 1 << (31 - delta_n % 32);
//             if  bitmaps[delta_n / 32] & bitmask == bitmask {               
//                 let frag_num = FragmentNumber(delta_n as u32 + base.0);
//                 set.insert(frag_num);
//             }
//         }
//         Ok(Self {base, set})
//     }    
// }

// impl SubmessageElement for Timestamp {
//     fn serialize(&self, writer: &mut impl std::io::Write, endianness: Endianness) -> RtpsSerdesResult<()>{
//         ULong(self.0.seconds()).serialize(writer, endianness)?;
//         ULong(self.0.fraction()).serialize(writer, endianness)?;
//         Ok(())
//     }

//     fn deserialize(bytes: &[u8], endianness: Endianness) -> RtpsSerdesResult<Self> {
//         bytes.check_size_equal(8)?;

//         let seconds = ULong::deserialize(&bytes[0..4], endianness)?.0;
//         let fraction = ULong::deserialize(&bytes[4..8], endianness)?.0;

//         Ok(Self(crate::messages::types::Time::new(seconds, fraction)))
//     }
// }

// impl SubmessageElement for Count {
//     fn serialize(&self, writer: &mut impl std::io::Write, endianness: Endianness) -> RtpsSerdesResult<()> {
//         Long(self.0).serialize(writer, endianness)?;
//         Ok(())
//     }

//     fn deserialize(bytes: &[u8], endianness: Endianness) -> RtpsSerdesResult<Self> {
//         let value = Long::deserialize(bytes, endianness)?.0;
//         Ok(Count(value))
//     }
// }

// fn serialize_locator(locator: &Locator, writer: &mut impl std::io::Write, endianness: Endianness) -> RtpsSerdesResult<()> {
//     Long(locator.kind()).serialize(writer, endianness)?;
//     ULong(locator.port()).serialize(writer, endianness)?;
//     writer.write(locator.address())?;
//     Ok(())
// }

// fn deserialize_locator(bytes: &[u8], endianness: Endianness) -> RtpsSerdesResult<Locator> {
//     bytes.check_size_equal(24)?;
//     let kind = Long::deserialize(&bytes[0..4], endianness)?.0;
//     let port = ULong::deserialize(&bytes[4..8], endianness)?.0;
//     let address = bytes[8..24].try_into()?;
//     Ok(Locator::new(kind, port, address))
// }

// impl SubmessageElement for LocatorList {
//     fn serialize(&self, writer: &mut impl std::io::Write, endianness: Endianness) -> RtpsSerdesResult<()> {
//         ULong(self.0.len() as u32).serialize(writer, endianness)?;
//         for locator in &self.0 {
//             serialize_locator(locator, writer, endianness)?;
//         };
//         Ok(())
//     }

//     fn deserialize(bytes: &[u8], endianness: Endianness) -> RtpsSerdesResult<Self> {
//         bytes.check_size_bigger_equal_than(4)?;
//         let size = bytes.len();
//         let num_locators = ULong::deserialize(&bytes[0..4], endianness)?.0;
//         let mut locators = Vec::<Locator>::new();
//         let mut index = 4;
//         while index < size && locators.len() < num_locators as usize {
//             bytes.check_size_bigger_equal_than(index+24)?;
//             let locator = deserialize_locator(&bytes[index..index+24], endianness)?;
//             index += 24;
//             locators.push(locator);
//         };
//         Ok(Self(locators))
//     }
// }

// impl SubmessageElement for SerializedData {
//     fn serialize(&self, writer: &mut impl std::io::Write, _endianness: Endianness) -> RtpsSerdesResult<()> {
//         writer.write(self.0.as_slice())?;
//         Ok(())
//     }

//     fn deserialize(bytes: &[u8], _endianness: Endianness) -> RtpsSerdesResult<Self> {
//         Ok(SerializedData(Vec::from(bytes)))
//     }
// }

// impl SubmessageElement for SerializedDataFragment {
//     fn serialize(&self, writer: &mut impl std::io::Write, _endianness: Endianness) -> RtpsSerdesResult<()> {
//         writer.write(self.0.as_slice())?;
//         Ok(())
//     }

//     fn deserialize(bytes: &[u8], _endianness: Endianness) -> RtpsSerdesResult<Self> {
//         Ok(SerializedDataFragment(Vec::from(bytes)))
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::messages::types::{ Time, };
//     use crate::types::constants;

//     // ///////// The GuidPrefix, and EntityId Tests ///////////////////////////
//     #[test]
//     fn invalid_guid_prefix_deserialization() {
//         let too_big_vec = [1,2,3,4,5,6,7,8,9,10,11,12,13,14];

//         let expected_error = GuidPrefix::deserialize(&too_big_vec, Endianness::LittleEndian);
//         match expected_error {
//             Err(RtpsSerdesError::WrongSize) => assert!(true),
//             _ => assert!(false),
//         };

//         let too_small_vec = [1,2];

//         let expected_error = GuidPrefix::deserialize(&too_small_vec, Endianness::BigEndian);
//         match expected_error {
//             Err(RtpsSerdesError::WrongSize) => assert!(true),
//             _ => assert!(false),
//         };
//     }
       
//     #[test]
//     fn entity_id_serialization_deserialization_big_endian() {
//         let mut vec = Vec::new();
//         let test_entity_id = EntityId(constants::ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER);

//         const TEST_ENTITY_ID_BIG_ENDIAN : [u8;4] = [0, 0x02, 0x00, 0xc4];
//         test_entity_id.serialize(&mut vec, Endianness::BigEndian).unwrap();
//         assert_eq!(vec, TEST_ENTITY_ID_BIG_ENDIAN);
//         assert_eq!(EntityId::deserialize(&vec, Endianness::BigEndian).unwrap(), test_entity_id);
//     }

//     #[test]
//     fn entity_id_serialization_deserialization_little_endian() {
//         let mut vec = Vec::new();
//         let test_entity_id = EntityId(constants::ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER);

//         const TEST_ENTITY_ID_LITTLE_ENDIAN : [u8;4] = [0, 0x02, 0x00, 0xc4];
//         test_entity_id.serialize(&mut vec, Endianness::LittleEndian).unwrap();
//         assert_eq!(vec, TEST_ENTITY_ID_LITTLE_ENDIAN);
//         assert_eq!(EntityId::deserialize(&vec, Endianness::LittleEndian).unwrap(), test_entity_id);
//     }

//     // ///////// VendorId Tests ///////////////////////////////////////////////
//     #[test]
//     fn invalid_vendor_id_deserialization() {
//         let too_big_vec = [1,2,3,4,5,6,7,8,9,10,11,12,13,14,1,2,3,4,5,6,7,8,9,10,11,12,13,14,1,2,3,4,5,6,7,8,9,10,11,12,13,14,];

//         let expected_error = VendorId::deserialize(&too_big_vec, Endianness::LittleEndian);
//         match expected_error {
//             Err(RtpsSerdesError::WrongSize) => assert!(true),
//             _ => assert!(false),
//         };

//         let too_small_vec = [1];

//         let expected_error = VendorId::deserialize(&too_small_vec, Endianness::BigEndian);
//         match expected_error {
//             Err(RtpsSerdesError::WrongSize) => assert!(true),
//             _ => assert!(false),
//         };
//     }

//     // ///////// ProtocolVersion Tests ////////////////////////////////////////
//     #[test]
//     fn invalid_protocol_version_deserialization() {
//         let too_big_vec = [1,2,3,4,5,6,7,8,9,10,11,12,13,14,1,2,3,4,5,6,7,8,9,10,11,12,13,14,1,2,3,4,5,6,7,8,9,10,11,12,13,14,];

//         let expected_error = ProtocolVersion::deserialize(&too_big_vec, Endianness::LittleEndian);
//         match expected_error {
//             Err(RtpsSerdesError::WrongSize) => assert!(true),
//             _ => assert!(false),
//         };

//         let too_small_vec = [1];

//         let expected_error = ProtocolVersion::deserialize(&too_small_vec, Endianness::BigEndian);
//         match expected_error {
//             Err(RtpsSerdesError::WrongSize) => assert!(true),
//             _ => assert!(false),
//         };
//     }

//     // ///////// SequenceNumber Tests /////////////////////////////////////////
 
//     #[test]
//     fn sequence_number_serialization_deserialization_big_endian() {
//         let mut vec = Vec::new();
//         let test_sequence_number = SequenceNumber(1987612345679);

        
//         const TEST_SEQUENCE_NUMBER_BIG_ENDIAN : [u8;8] = [0x00, 0x00, 0x01, 0xCE, 0xC6, 0xED, 0x85, 0x4F];
//         test_sequence_number.serialize(&mut vec, Endianness::BigEndian).unwrap();
//         assert_eq!(vec, TEST_SEQUENCE_NUMBER_BIG_ENDIAN);
//         assert_eq!(SequenceNumber::deserialize(&vec, Endianness::BigEndian).unwrap(), test_sequence_number);
//     }

//     #[test]
//     fn sequence_number_serialization_deserialization_little_endian() {
//         let mut vec = Vec::new();
//         let test_sequence_number = SequenceNumber(1987612345679);

        
//         const TEST_SEQUENCE_NUMBER_LITTLE_ENDIAN : [u8;8] = [0xCE, 0x01, 0x00, 0x00, 0x4F, 0x85, 0xED, 0xC6];
//         test_sequence_number.serialize(&mut vec, Endianness::LittleEndian).unwrap();
//         assert_eq!(vec, TEST_SEQUENCE_NUMBER_LITTLE_ENDIAN);
//         assert_eq!(SequenceNumber::deserialize(&vec, Endianness::LittleEndian).unwrap(), test_sequence_number);
//     }

//     #[test]
//     fn sequence_number_serialization_deserialization_multiple_combinations() {
//         let mut vec = Vec::new();
        
//         {
//             let test_sequence_number_i64_max = SequenceNumber(std::i64::MAX);
//             test_sequence_number_i64_max.serialize(&mut vec, Endianness::LittleEndian).unwrap();
//             assert_eq!(SequenceNumber::deserialize(&vec, Endianness::LittleEndian).unwrap(), test_sequence_number_i64_max);
//             vec.clear();

//             test_sequence_number_i64_max.serialize(&mut vec, Endianness::BigEndian).unwrap();
//             assert_eq!(SequenceNumber::deserialize(&vec, Endianness::BigEndian).unwrap(), test_sequence_number_i64_max);
//             vec.clear();
//         }

//         {
//             let test_sequence_number_i64_min = SequenceNumber(std::i64::MIN);
//             test_sequence_number_i64_min.serialize(&mut vec, Endianness::LittleEndian).unwrap();
//             assert_eq!(SequenceNumber::deserialize(&vec, Endianness::LittleEndian).unwrap(), test_sequence_number_i64_min);
//             vec.clear();

//             test_sequence_number_i64_min.serialize(&mut vec, Endianness::BigEndian).unwrap();
//             assert_eq!(SequenceNumber::deserialize(&vec, Endianness::BigEndian).unwrap(), test_sequence_number_i64_min);
//             vec.clear();
//         }

//         {
//             let test_sequence_number_zero = SequenceNumber(0);
//             test_sequence_number_zero.serialize(&mut vec, Endianness::LittleEndian).unwrap();
//             assert_eq!(SequenceNumber::deserialize(&vec, Endianness::LittleEndian).unwrap(), test_sequence_number_zero);
//             vec.clear();

//             test_sequence_number_zero.serialize(&mut vec, Endianness::BigEndian).unwrap();
//             assert_eq!(SequenceNumber::deserialize(&vec, Endianness::BigEndian).unwrap(), test_sequence_number_zero);
//             vec.clear();
//         }
//     }

//     #[test]
//     fn invalid_sequence_number_deserialization() {
//         let wrong_vec = [1,2,3,4];

//         let expected_error = SequenceNumber::deserialize(&wrong_vec, Endianness::LittleEndian);
//         match expected_error {
//             Err(RtpsSerdesError::WrongSize) => assert!(true),
//             _ => assert!(false),
//         };
//     }


//     // /////////////////////// SequenceNumberSet Tests ////////////////////////

//     #[test]
//     fn sequence_number_set_constructor() {
//         let expected = SequenceNumberSet{
//             base: 1001,
//             set:  [1001, 1003].iter().cloned().collect(),
//         };
//         let result = SequenceNumberSet::from_set([1001, 1003].iter().cloned().collect());
//         assert_eq!(expected, result);
//     }

//     #[test]
//     fn sequence_number_set_constructor_empty_set() {        
//         let expected = SequenceNumberSet{
//             base: 0,
//             set:  [].iter().cloned().collect(),
//         };
//         let result = SequenceNumberSet::from_set([].iter().cloned().collect());
//         assert_eq!(expected, result);
//     }
    
//     #[test]
//     fn deserialize_sequence_number_set_empty() {
//         let expected = SequenceNumberSet{
//             base: 3,
//             set: [].iter().cloned().collect()
//         };
//         let bytes = vec![
//             0, 0, 0, 0, // base
//             0, 0, 0, 3, // base
//             0, 0, 0, 0, // num bits
//         ];
//         let result = SequenceNumberSet::deserialize(&bytes, Endianness::BigEndian).unwrap();
//         assert_eq!(expected, result);
//     }
    
//     #[test]
//     fn deserialize_sequence_number_set_one_bitmap_be() {
//         let expected = SequenceNumberSet{
//             base: 3,
//             set: [3, 4].iter().cloned().collect()
//         };
//         let bytes = vec![
//             0, 0, 0, 0, // base
//             0, 0, 0, 3, // base
//             0, 0, 0, 2, // num bits
//             0b_11000000, 0b_00000000, 0b_00000000, 0b_00000000, 
//         ];
//         let result = SequenceNumberSet::deserialize(&bytes, Endianness::BigEndian).unwrap();
//         assert_eq!(expected, result);
//     }
    
//     #[test]
//         fn deserialize_sequence_number_set_one_bitmap_le() {
//         let expected = SequenceNumberSet{
//             base: 3,
//             set: [3, 4].iter().cloned().collect()
//         };
//         let bytes = vec![
//             0, 0, 0, 0, // base
//             3, 0, 0, 0, // base
//             2, 0, 0, 0, // num bits
//             0b_00000000, 0b_00000000, 0b_00000000, 0b_11000000, 
//         ];
//         let result = SequenceNumberSet::deserialize(&bytes, Endianness::LittleEndian).unwrap();
//         assert_eq!(expected, result);
//     }
    
//     #[test]
//     fn deserialize_sequence_number_set_multiple_bitmaps() {
//         let expected = SequenceNumberSet{
//             base: 1000,
//             set: [1001, 1003, 1032, 1033].iter().cloned().collect()
//         };
//         let bytes = vec![
//             0, 0, 0, 0, // base
//             0, 0, 3, 232, // base
//             0, 0, 0, 34, // num bits
//             0b_01010000, 0b_00000000, 0b_00000000, 0b_00000000, 
//             0b_11000000, 0b_00000000, 0b_00000000, 0b_00000000, 
//         ];
//         let result = SequenceNumberSet::deserialize(&bytes, Endianness::BigEndian).unwrap();
//         assert_eq!(expected, result);
//     }
    
//     #[test]
//     fn deserialize_sequence_number_set_max_bitmaps_big_endian() {
//         let expected = SequenceNumberSet{
//             base: 1000,
//             set: [1000, 1255].iter().cloned().collect()
//         };
//         let bytes = vec![
//             0, 0, 0, 0, // base
//             0, 0, 0x03, 0xE8, // base
//             0, 0, 0x01, 0x00, // num bits
//             0b_10000000, 0b_00000000, 0b_00000000, 0b_00000000, 
//             0b_00000000, 0b_00000000, 0b_00000000, 0b_00000000, 
//             0b_00000000, 0b_00000000, 0b_00000000, 0b_00000000,
//             0b_00000000, 0b_00000000, 0b_00000000, 0b_00000000,
//             0b_00000000, 0b_00000000, 0b_00000000, 0b_00000000,
//             0b_00000000, 0b_00000000, 0b_00000000, 0b_00000000,
//             0b_00000000, 0b_00000000, 0b_00000000, 0b_00000000,
//             0b_00000000, 0b_00000000, 0b_00000000, 0b_00000001,
//         ];
//         let result = SequenceNumberSet::deserialize(&bytes, Endianness::BigEndian).unwrap();
//         assert_eq!(expected, result);
//     }

//     #[test]
//     fn deserialize_sequence_number_set_max_bitmaps_little_endian() {
//         let expected = SequenceNumberSet{
//             base: 1000,
//             set: [1000, 1255].iter().cloned().collect()
//         };
//         let bytes = vec![
//             0, 0, 0, 0, // base
//             0xE8, 0x03, 0, 0, // base
//             0x00, 0x01, 0, 0, // num bits
//             0b_00000000, 0b_00000000, 0b_00000000, 0b_10000000, 
//             0b_00000000, 0b_00000000, 0b_00000000, 0b_00000000, 
//             0b_00000000, 0b_00000000, 0b_00000000, 0b_00000000,
//             0b_00000000, 0b_00000000, 0b_00000000, 0b_00000000,
//             0b_00000000, 0b_00000000, 0b_00000000, 0b_00000000,
//             0b_00000000, 0b_00000000, 0b_00000000, 0b_00000000,
//             0b_00000000, 0b_00000000, 0b_00000000, 0b_00000000,
//             0b_00000001, 0b_00000000, 0b_00000000, 0b_00000000, 
//         ];
//         let result = SequenceNumberSet::deserialize(&bytes, Endianness::LittleEndian).unwrap();
//         assert_eq!(expected, result);
//     }

//     #[test]
//     fn deserialize_sequence_number_set_as_of_example_in_standard_be() {
//         // Example in standard "1234:/12:00110"
//         let bytes = [
//             0x00, 0x00, 0x00, 0x00, 
//             0x00, 0x00, 0x04, 0xD2, 
//             0x00, 0x00, 0x00, 0x0C, 
//             0x30, 0x00, 0x00, 0x00, 
//         ];
//         let set = SequenceNumberSet::deserialize(&bytes, Endianness::BigEndian).unwrap().set;
//         assert!(!set.contains(&1234));
//         assert!(!set.contains(&1235));
//         assert!(set.contains(&1236));
//         assert!(set.contains(&1237));
//         for seq_num in 1238..1245 {
//             assert!(!set.contains(&seq_num));
//         }
//     }
    
//     #[test]
//     fn deserialize_sequence_number_set_as_of_example_in_standard_le() {
//         // Example in standard "1234:/12:00110"
//         let bytes = [
//             0x00, 0x00, 0x00, 0x00, 
//             0xD2, 0x04, 0x00, 0x00, 
//             0x0C, 0x00, 0x00, 0x00, 
//             0x00, 0x00, 0x00, 0x30, 
//         ];
//         let set = SequenceNumberSet::deserialize(&bytes, Endianness::LittleEndian).unwrap().set;
//         assert!(!set.contains(&1234));
//         assert!(!set.contains(&1235));
//         assert!(set.contains(&1236));
//         assert!(set.contains(&1237));
//         for seq_num in 1238..1245 {
//             assert!(!set.contains(&seq_num));
//         }
//     }
        
    
//     #[test]
//     fn serialize_sequence_number_set() {
//         let set = SequenceNumberSet{
//             base: 3,
//             set: [3, 4].iter().cloned().collect()
//         };
//         let mut writer = Vec::new();
//         set.serialize(&mut writer, Endianness::BigEndian).unwrap();
//         let expected = vec![
//             0, 0, 0, 0, // base
//             0, 0, 0, 3, // base
//             0, 0, 0, 2, // num bits
//             0b_11000000, 0b_00000000, 0b_00000000, 0b_00000000, 
//         ];
//         assert_eq!(expected, writer);
    
    
//         let set = SequenceNumberSet{
//             base: 1,
//             set: [3, 4].iter().cloned().collect()
//         };
//         let mut writer = Vec::new();
//         set.serialize(&mut writer, Endianness::LittleEndian).unwrap();
//         let expected = vec![
//             0, 0, 0, 0, // base
//             1, 0, 0, 0, // base
//             4, 0, 0, 0, // num bits
//             0b_00000000, 0b_00000000, 0b_00000000, 0b_00110000, 
//         ];
//         assert_eq!(expected, writer);
    
//         let mut writer = Vec::new();
//         set.serialize(&mut writer, Endianness::BigEndian).unwrap();
//         let expected = vec![
//             0, 0, 0, 0, // base
//             0, 0, 0, 1, // base
//             0, 0, 0, 4, // num bits
//             0b_00110000, 0b_00000000, 0b_00000000, 0b_00000000,  
//         ];
//         assert_eq!(expected, writer);
    
    
//         let set = SequenceNumberSet{
//             base: 1000,
//             set: [1001, 1003, 1032, 1033].iter().cloned().collect()
//         };
//         let mut writer = Vec::new();
//         set.serialize(&mut writer, Endianness::BigEndian).unwrap();
//         let expected = vec![
//             0, 0, 0, 0, // base
//             0, 0, 3, 232, // base
//             0, 0, 0, 34, // num bits
//             0b_01010000, 0b_00000000, 0b_00000000, 0b_00000000, 
//             0b_11000000, 0b_00000000, 0b_00000000, 0b_00000000, 
//         ];
//         assert_eq!(expected, writer);
//     }

        

//     // //////////////////////// FragmentNumber Tests //////////////////////////
    
//     #[test]
//     fn serialize_fragment_number() {
//         let fragment_number = FragmentNumber(100);
//         let expected = vec![
//             100, 0, 0, 0,
//         ];
//         let mut writer = Vec::new();
//         fragment_number.serialize(&mut writer, Endianness::LittleEndian).unwrap();
//         assert_eq!(expected, writer);
//     }

//     #[test]
//     fn deserialize_fragment_number() {
//         let expected = FragmentNumber(100);
//         let bytes = vec![
//             100, 0, 0, 0,
//         ];
//         let result = FragmentNumber::deserialize(&bytes, Endianness::LittleEndian).unwrap();
//         assert_eq!(expected, result);
//     }


//     // /////////////////////// FragmentNumberSet Tests ////////////////////////

//     #[test]
//     fn fragment_number_set_constructor() {
//         let expected = FragmentNumberSet{
//             base: FragmentNumber(1001),
//             set:  [FragmentNumber(1001), FragmentNumber(1003)].iter().cloned().collect(),
//         };
//         let result = FragmentNumberSet::new([FragmentNumber(1001), FragmentNumber(1003)].iter().cloned().collect());
//         assert_eq!(expected, result);
//     }

//     #[test]
//     fn deserialize_fragment_number_set() {
//         let expected = FragmentNumberSet{
//             base: FragmentNumber(3),
//             set: [FragmentNumber(3), FragmentNumber(4)].iter().cloned().collect()
//         };
//         let bytes = vec![
//             3, 0, 0, 0, // base
//             2, 0, 0, 0, // num bits
//             0b_00000000, 0b_00000000, 0b_00000000, 0b_11000000, 
//         ];
//         let result = FragmentNumberSet::deserialize(&bytes, Endianness::LittleEndian).unwrap();
//         assert_eq!(expected, result);
//     }

//     #[test]
//     fn serialize_fragment_number_set() {
//         let set = FragmentNumberSet{
//             base: FragmentNumber(3),
//             set: [FragmentNumber(3), FragmentNumber(4)].iter().cloned().collect()
//         };
//         let mut writer = Vec::new();
//         set.serialize(&mut writer, Endianness::BigEndian).unwrap();
//         let expected = vec![
//             0, 0, 0, 3, // base
//             0, 0, 0, 2, // num bits
//             0b_11000000, 0b_00000000, 0b_00000000, 0b_00000000, 
//         ];
//         assert_eq!(expected, writer);
//     }

//     // /////////////////////// Timestamp Tests ////////////////////////////////
//     #[test]
//     fn test_time_serialization_deserialization_big_endian() {
//         let mut vec = Vec::new();
//         let test_time = Timestamp(Time::new(1234567, 98765432));

        
//         const TEST_TIME_BIG_ENDIAN : [u8;8] = [0x00, 0x12, 0xD6, 0x87, 0x05, 0xE3, 0x0A, 0x78];
//         test_time.serialize(&mut vec, Endianness::BigEndian).unwrap();
//         assert_eq!(vec, TEST_TIME_BIG_ENDIAN);
//         assert_eq!(Timestamp::deserialize(&vec, Endianness::BigEndian).unwrap().0, test_time.0);
//     }

//     #[test]
//     fn test_time_serialization_deserialization_little_endian() {
//         let mut vec = Vec::new();
//         let test_time = Timestamp(Time::new(1234567, 98765432));
        
//         const TEST_TIME_LITTLE_ENDIAN : [u8;8] = [0x87, 0xD6, 0x12, 0x00, 0x78, 0x0A, 0xE3, 0x05];
//         test_time.serialize(&mut vec, Endianness::LittleEndian).unwrap();
//         assert_eq!(vec, TEST_TIME_LITTLE_ENDIAN);
//         assert_eq!(Timestamp::deserialize(&vec, Endianness::LittleEndian).unwrap().0, test_time.0);
//     }

//     #[test]
//     fn test_invalid_time_deserialization() {
//         let wrong_vec = vec![1,2,3,4];

//         let expected_error = Timestamp::deserialize(&wrong_vec, Endianness::LittleEndian);
//         match expected_error {
//             Err(RtpsSerdesError::WrongSize) => assert!(true),
//             _ => assert!(false),
//         };
//     }

//     // /////////////////////// Count Tests ////////////////////////////////////
//     // todo!  

//     // /////////////////////// LocatorList Tests ////////////////////////
     
//      #[test]
//     fn serialize_locator_simple() {
//         let locator = Locator::new(100, 200, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
//         let expected = vec![
//             100, 0, 0, 0, // kind
//             200, 0, 0, 0, // port
//              1,  2,  3,  4, // address
//              5,  6,  7,  8, // address
//              9, 10, 11, 12, // address
//             13, 14, 15, 16, // address
//         ];
//         let mut writer = Vec::new();
//         serialize_locator(&locator, &mut writer, Endianness::LittleEndian).unwrap();
//         assert_eq!(expected, writer);
//     }
    
//     #[test]
//     fn deserialize_locator_simple() {
//         let expected = Locator::new(100, 200, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
//         let bytes = vec![
//             100, 0, 0, 0, // kind
//             200, 0, 0, 0, // port
//              1,  2,  3,  4, // address
//              5,  6,  7,  8, // address
//              9, 10, 11, 12, // address
//             13, 14, 15, 16, // address
//         ];
//         let result = deserialize_locator(&bytes, Endianness::LittleEndian).unwrap();
//         assert_eq!(expected, result);
//     }

//     #[test]
//     fn invalid_locator_deserialization() {
//         let too_big_vec = [1,2,3,4,5,6,7,8,9,10,11,12,13,14,1,2,3,4,5,6,7,8,9,10,11,12,13,14,1,2,3,4,5,6,7,8,9,10,11,12,13,14,];

//         let expected_error = deserialize_locator(&too_big_vec, Endianness::LittleEndian);
//         match expected_error {
//             Err(RtpsSerdesError::WrongSize) => assert!(true),
//             _ => assert!(false),
//         };

//         let too_small_vec = [1,2];

//         let expected_error = deserialize_locator(&too_small_vec, Endianness::BigEndian);
//         match expected_error {
//             Err(RtpsSerdesError::WrongSize) => assert!(true),
//             _ => assert!(false),
//         };
//     }

//     #[test]
//     fn serialize_locator_list() {
//         let address = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
//         let locator_list = LocatorList(vec![
//             Locator::new(100, 200, address),
//             Locator::new(101, 201, address),
//         ]);
//         let expected = vec![
//             2, 0, 0, 0, // numLocators
//             100, 0, 0, 0, // Locator 1: kind
//             200, 0, 0, 0, // Locator 1: port
//             1,  2,  3,  4, // Locator 1: address
//             5,  6,  7,  8, // Locator 1: address
//             9, 10, 11, 12, // Locator 1: address
//             13, 14, 15, 16, // Locator 1: address
//             101, 0, 0, 0, // Locator 2: kind
//             201, 0, 0, 0, // Locator 2: port
//             1,  2,  3,  4, // Locator 2: address
//             5,  6,  7,  8, // Locator 2: address
//             9, 10, 11, 12, // Locator 2: address
//             13, 14, 15, 16, // Locator 2: address
//         ];
//         let mut writer = Vec::new();
//         locator_list.serialize(&mut writer, Endianness::LittleEndian).unwrap();
//         assert_eq!(expected, writer);
//     }

    
//     #[test]
//     fn deserialize_locator_list() {
//         let bytes = vec![
//             2, 0, 0, 0,   // numLocators
//             100, 0, 0, 0, // Locator 1: kind
//             200, 0, 0, 0, // Locator 1: port
//             1,  2,  3,  4, // Locator 1: address
//             5,  6,  7,  8, // Locator 1: address
//             9, 10, 11, 12, // Locator 1: address
//             13, 14, 15, 16, // Locator 1: address
//             101, 0, 0, 0, // Locator 2: kind
//             201, 0, 0, 0, // Locator 2: port
//             1,  2,  3,  4, // Locator 2: address
//             5,  6,  7,  8, // Locator 2: address
//             9, 10, 11, 12, // Locator 2: address
//             13, 14, 15, 16, // Locator 2: address
//         ];
//         let address = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
//         let expected = LocatorList(vec![
//             Locator::new(100, 200, address),
//             Locator::new(101, 201, address),
//         ]);
//         let result = LocatorList::deserialize(&bytes, Endianness::LittleEndian).unwrap();
//         assert_eq!(expected, result);
//     }

    
//     #[test]
//     fn serialize_deserialize_ushort(){
//         let mut buf = Vec::new();

//         let val = UShort(123);

//         val.serialize(&mut buf, Endianness::LittleEndian).unwrap();
//         assert_eq!(buf, [123, 0]);
//         assert_eq!(UShort::deserialize(&buf, Endianness::LittleEndian).unwrap(), val);
//         buf.clear();

//         val.serialize(&mut buf, Endianness::BigEndian).unwrap();
//         assert_eq!(buf, [0, 123]);
//         assert_eq!(UShort::deserialize(&buf, Endianness::BigEndian).unwrap(), val);
//         buf.clear();


//         let max = UShort(u16::MAX);

//         max.serialize(&mut buf, Endianness::LittleEndian).unwrap();
//         assert_eq!(buf, [0xFF, 0xFF]);
//         assert_eq!(UShort::deserialize(&buf, Endianness::LittleEndian).unwrap(), max);
//         buf.clear();

//         max.serialize(&mut buf, Endianness::BigEndian).unwrap();
//         assert_eq!(buf, [0xFF, 0xFF]);
//         assert_eq!(UShort::deserialize(&buf, Endianness::BigEndian).unwrap(), max);
//         buf.clear();

//         let min = UShort(u16::MIN);

//         min.serialize(&mut buf, Endianness::LittleEndian).unwrap();
//         assert_eq!(buf, [0x00, 0x00]);
//         assert_eq!(UShort::deserialize(&buf, Endianness::LittleEndian).unwrap(), min);
//         buf.clear();

//         min.serialize(&mut buf, Endianness::BigEndian).unwrap();
//         assert_eq!(buf, [0x00, 0x00]);
//         assert_eq!(UShort::deserialize(&buf, Endianness::BigEndian).unwrap(), min);
//         buf.clear();
//     }

//     #[test]
//     fn invalid_ushort_deserialize() {
//         let buf: [u8; 1] = [1];
//         let result = UShort::deserialize(&buf, Endianness::BigEndian);
//         match result {
//             Err(RtpsSerdesError::WrongSize) => assert!(true),
//             _ => assert!(false),
//         }
//     }

//     #[test]
//     fn serialize_deserialize_short(){
//         let mut buf = Vec::new();

//         let val = Short(123);

//         val.serialize(&mut buf, Endianness::LittleEndian).unwrap();
//         assert_eq!(buf, [123, 0]);
//         assert_eq!(Short::deserialize(&buf, Endianness::LittleEndian).unwrap(), val);
//         buf.clear();

//         val.serialize(&mut buf, Endianness::BigEndian).unwrap();
//         assert_eq!(buf, [0, 123]);
//         assert_eq!(Short::deserialize(&buf, Endianness::BigEndian).unwrap(), val);
//         buf.clear();


//         let max = Short(i16::MAX);

//         max.serialize(&mut buf, Endianness::LittleEndian).unwrap();
//         assert_eq!(buf, [0xFF, 0x7F]);
//         assert_eq!(Short::deserialize(&buf, Endianness::LittleEndian).unwrap(), max);
//         buf.clear();

//         max.serialize(&mut buf, Endianness::BigEndian).unwrap();
//         assert_eq!(buf, [0x7F, 0xFF]);
//         assert_eq!(Short::deserialize(&buf, Endianness::BigEndian).unwrap(), max);
//         buf.clear();

//         let min = Short(i16::MIN);

//         min.serialize(&mut buf, Endianness::LittleEndian).unwrap();
//         assert_eq!(buf, [0x00, 0x80]);
//         assert_eq!(Short::deserialize(&buf, Endianness::LittleEndian).unwrap(), min);
//         buf.clear();

//         min.serialize(&mut buf, Endianness::BigEndian).unwrap();
//         assert_eq!(buf, [0x80, 0x00]);
//         assert_eq!(Short::deserialize(&buf, Endianness::BigEndian).unwrap(), min);
//         buf.clear();
//     }

//     #[test]
//     fn invalid_short_deserialize() {
//         let buf: [u8; 1] = [1];
//         let result = Short::deserialize(&buf, Endianness::BigEndian);
//         match result {
//             Err(RtpsSerdesError::WrongSize) => assert!(true),
//             _ => assert!(false),
//         }
//     }

//     #[test]
//     fn serialize_deserialize_long(){
//         let mut buf = Vec::new();

//         let val = Long(1230);

//         val.serialize(&mut buf, Endianness::LittleEndian).unwrap();
//         assert_eq!(buf, [0xCE, 0x04, 0, 0]);
//         assert_eq!(Long::deserialize(&buf, Endianness::LittleEndian).unwrap(), val);
//         buf.clear();

//         val.serialize(&mut buf, Endianness::BigEndian).unwrap();
//         assert_eq!(buf, [0, 0, 0x04, 0xCE]);
//         assert_eq!(Long::deserialize(&buf, Endianness::BigEndian).unwrap(), val);
//         buf.clear();


//         let max = Long(i32::MAX);

//         max.serialize(&mut buf, Endianness::LittleEndian).unwrap();
//         assert_eq!(buf, [0xFF, 0xFF, 0xFF, 0x7F]);
//         assert_eq!(Long::deserialize(&buf, Endianness::LittleEndian).unwrap(), max);
//         buf.clear();

//         max.serialize(&mut buf, Endianness::BigEndian).unwrap();
//         assert_eq!(buf, [0x7F, 0xFF, 0xFF, 0xFF]);
//         assert_eq!(Long::deserialize(&buf, Endianness::BigEndian).unwrap(), max);
//         buf.clear();

//         let min = Long(i32::MIN);

//         min.serialize(&mut buf, Endianness::LittleEndian).unwrap();
//         assert_eq!(buf, [0x00, 0x00, 0x00, 0x80]);
//         assert_eq!(Long::deserialize(&buf, Endianness::LittleEndian).unwrap(), min);
//         buf.clear();

//         min.serialize(&mut buf, Endianness::BigEndian).unwrap();
//         assert_eq!(buf, [0x80, 0x00, 0x00, 0x00]);
//         assert_eq!(Long::deserialize(&buf, Endianness::BigEndian).unwrap(), min);
//         buf.clear();
//     }

//     #[test]
//     fn invalid_long_deserialize() {
//         let buf: [u8; 3] = [1, 2, 3];
//         let result = Long::deserialize(&buf, Endianness::BigEndian);
//         match result {
//             Err(RtpsSerdesError::WrongSize) => assert!(true),
//             _ => assert!(false),
//         }
//     }

//     #[test]
//     fn serialize_deserialize_ulong(){
//         let mut buf = Vec::new();

//         let val = ULong(1230);

//         val.serialize(&mut buf, Endianness::LittleEndian).unwrap();
//         assert_eq!(buf, [0xCE, 0x04, 0, 0]);
//         assert_eq!(ULong::deserialize(&buf, Endianness::LittleEndian).unwrap(), val);
//         buf.clear();

//         val.serialize(&mut buf, Endianness::BigEndian).unwrap();
//         assert_eq!(buf, [0, 0, 0x04, 0xCE]);
//         assert_eq!(ULong::deserialize(&buf, Endianness::BigEndian).unwrap(), val);
//         buf.clear();


//         let max = ULong(u32::MAX);

//         max.serialize(&mut buf, Endianness::LittleEndian).unwrap();
//         assert_eq!(buf, [0xFF, 0xFF, 0xFF, 0xFF]);
//         assert_eq!(ULong::deserialize(&buf, Endianness::LittleEndian).unwrap(), max);
//         buf.clear();

//         max.serialize(&mut buf, Endianness::BigEndian).unwrap();
//         assert_eq!(buf, [0xFF, 0xFF, 0xFF, 0xFF]);
//         assert_eq!(ULong::deserialize(&buf, Endianness::BigEndian).unwrap(), max);
//         buf.clear();

//         let min = ULong(u32::MIN);

//         min.serialize(&mut buf, Endianness::LittleEndian).unwrap();
//         assert_eq!(buf, [0x00, 0x00, 0x00, 0x00]);
//         assert_eq!(ULong::deserialize(&buf, Endianness::LittleEndian).unwrap(), min);
//         buf.clear();

//         min.serialize(&mut buf, Endianness::BigEndian).unwrap();
//         assert_eq!(buf, [0x00, 0x00, 0x00, 0x00]);
//         assert_eq!(ULong::deserialize(&buf, Endianness::BigEndian).unwrap(), min);
//         buf.clear();
//     }

//     #[test]
//     fn invalid_ulong_deserialize() {
//         let buf: [u8; 3] = [1, 2, 3];
//         let result = ULong::deserialize(&buf, Endianness::BigEndian);
//         match result {
//             Err(RtpsSerdesError::WrongSize) => assert!(true),
//             _ => assert!(false),
//         }
//     }
// }
