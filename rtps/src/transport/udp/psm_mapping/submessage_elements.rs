use std::convert::TryInto;

use std::collections::BTreeSet;

use crate::messages::submessages::submessage_elements::{Long, Short, ULong, UShort, Timestamp, GuidPrefix, EntityId, VendorId, ProtocolVersion, SequenceNumber, SequenceNumberSet, FragmentNumber, FragmentNumberSet, LocatorList, Count, SerializedData, ParameterList};
use crate::messages::types::Time;
use crate::types;
use crate::messages::types::Endianness;

use super::{SizeCheck, UdpPsmMappingResult, UdpPsmMappingError};

pub fn serialize_long(
    long: &Long,
    writer: &mut impl std::io::Write,
    endianness: Endianness,
) -> UdpPsmMappingResult<()> {
    let value = match endianness {
        Endianness::BigEndian => long.to_be_bytes(),
        Endianness::LittleEndian => long.to_le_bytes(),
    };
    writer.write(&value)?;
    Ok(())
}

pub fn deserialize_long(bytes: &[u8], endianness: Endianness) -> UdpPsmMappingResult<Long> {
    bytes.check_size_equal(4)?;

    let value = match endianness {
        Endianness::BigEndian => i32::from_be_bytes(bytes[0..4].try_into()?),
        Endianness::LittleEndian => i32::from_le_bytes(bytes[0..4].try_into()?),
    };
    Ok(value)
}

pub fn serialize_ulong(
    ulong: &ULong,
    writer: &mut impl std::io::Write,
    endianness: Endianness,
) -> UdpPsmMappingResult<()> {
    let value = match endianness {
        Endianness::BigEndian => ulong.to_be_bytes(),
        Endianness::LittleEndian => ulong.to_le_bytes(),
    };
    writer.write(&value)?;
    Ok(())
}

pub fn deserialize_ulong(bytes: &[u8], endianness: Endianness) -> UdpPsmMappingResult<ULong> {
    bytes.check_size_equal(4)?;

    let value = match endianness {
        Endianness::BigEndian => u32::from_be_bytes(bytes[0..4].try_into()?),
        Endianness::LittleEndian => u32::from_le_bytes(bytes[0..4].try_into()?),
    };
    Ok(value)
}

pub fn serialize_short(
    short: &Short,
    writer: &mut impl std::io::Write,
    endianness: Endianness,
) -> UdpPsmMappingResult<()> {
    let value = match endianness {
        Endianness::BigEndian => short.to_be_bytes(),
        Endianness::LittleEndian => short.to_le_bytes(),
    };
    writer.write(&value)?;
    Ok(())
}

pub fn deserialize_short(bytes: &[u8], endianness: Endianness) -> UdpPsmMappingResult<Short> {
    bytes.check_size_equal(2)?;

    let value = match endianness {
        Endianness::BigEndian => i16::from_be_bytes(bytes[0..2].try_into()?),
        Endianness::LittleEndian => i16::from_le_bytes(bytes[0..2].try_into()?),
    };
    Ok(value)
}

pub fn serialize_ushort(
    ushort: &UShort,
    writer: &mut impl std::io::Write,
    endianness: Endianness,
) -> UdpPsmMappingResult<()> {
    let value = match endianness {
        Endianness::BigEndian => ushort.to_be_bytes(),
        Endianness::LittleEndian => ushort.to_le_bytes(),
    };
    writer.write(&value)?;
    Ok(())
}

pub fn deserialize_ushort(bytes: &[u8], endianness: Endianness) -> UdpPsmMappingResult<UShort> {
    bytes.check_size_equal(2)?;

    let value = match endianness {
        Endianness::BigEndian => u16::from_be_bytes(bytes[0..2].try_into()?),
        Endianness::LittleEndian => u16::from_le_bytes(bytes[0..2].try_into()?),
    };
    Ok(value)
}

pub fn serialize_guid_prefix(guid_prefix: &GuidPrefix, writer: &mut impl std::io::Write) -> UdpPsmMappingResult<()> {
    writer.write(guid_prefix)?;
    Ok(())
}

pub fn deserialize_guid_prefix(bytes: &[u8]) -> UdpPsmMappingResult<GuidPrefix> {
    bytes.check_size_equal(12)?;
    Ok(bytes[0..12].try_into()?)
}    

pub fn serialize_entity_id(entity_id: &EntityId, writer: &mut impl std::io::Write) -> UdpPsmMappingResult<()>{
    writer.write(&entity_id.entity_key())?;
    writer.write(&[entity_id.entity_kind() as u8])?;
    Ok(())
}

pub fn deserialize_entity_id(bytes: &[u8]) -> UdpPsmMappingResult<EntityId> {
    bytes.check_size_equal( 4)?;
    let entity_key = bytes[0..3].try_into()?;
    let entity_kind = num::FromPrimitive::from_u8(bytes[3]).ok_or(UdpPsmMappingError::InvalidEnumRepresentation)?;
    Ok(types::EntityId::new(entity_key, entity_kind))
}

pub fn serialize_vendor_id(vendor_id: &VendorId, writer: &mut impl std::io::Write) -> UdpPsmMappingResult<()> {
    writer.write(vendor_id)?;
    Ok(())
}

pub fn deserialize_vendor_id(bytes: &[u8]) -> UdpPsmMappingResult<VendorId> {
    bytes.check_size_equal(2)?;
    Ok(bytes[0..2].try_into()?)
}

pub fn serialize_protocol_version(protocol_version: &ProtocolVersion, writer: &mut impl std::io::Write) -> UdpPsmMappingResult<()> {
    writer.write(&[protocol_version.major])?;
    writer.write(&[protocol_version.minor])?;
    Ok(())
}

pub fn deserialize_protocol_version(bytes: &[u8]) -> UdpPsmMappingResult<ProtocolVersion> {
    bytes.check_size_equal(2)?;
    let major = bytes[0];
    let minor = bytes[1];
    Ok(types::ProtocolVersion{major, minor})
}

pub fn serialize_sequence_number(sequence_number: &SequenceNumber, writer: &mut impl std::io::Write, endianness: Endianness) -> UdpPsmMappingResult<()>{
    let msb = (sequence_number >> 32) as i32;
    let lsb = (sequence_number & 0x0000_0000_FFFF_FFFF) as u32;
    serialize_long(&msb, writer, endianness)?;
    serialize_ulong(&lsb, writer, endianness)?;
    Ok(())
}

pub fn deserialize_sequence_number(bytes: &[u8], endianness: Endianness) -> UdpPsmMappingResult<SequenceNumber> {
    bytes.check_size_equal(8)?;

    let msb = deserialize_long(&bytes[0..4], endianness)?;
    let lsb = deserialize_ulong(&bytes[4..8], endianness)?;

    let sequence_number = ((msb as i64) << 32) + lsb as i64;

    Ok(sequence_number)
}

pub fn serialize_sequence_number_set(sequence_number_set: &SequenceNumberSet, writer: &mut impl std::io::Write, endianness: Endianness) -> UdpPsmMappingResult<()> {
    let num_bits = if sequence_number_set.set().is_empty() {
        0 
    } else {
        (sequence_number_set.set().iter().last().unwrap() - sequence_number_set.base()) as usize + 1
    } as u32;
    let m = ((num_bits + 31) / 32) as usize;
    let mut bitmaps = vec![0_i32; m];
    serialize_sequence_number(sequence_number_set.base(), writer, endianness)?;
    serialize_ulong(&num_bits, writer, endianness)?;
    for seq_num in sequence_number_set.set() {
        let delta_n = (seq_num - sequence_number_set.base()) as usize;
        let bitmap_i = delta_n / 32;
        let bitmask = 1 << (31 - delta_n % 32);
        bitmaps[bitmap_i] |= bitmask;
    };
    for bitmap in bitmaps {
        serialize_long(&bitmap, writer, endianness)?;
    }
    Ok(())
}

pub fn deserialize_sequence_number_set(bytes: &[u8], endianness: Endianness) -> UdpPsmMappingResult<SequenceNumberSet> {
    bytes.check_size_bigger_equal_than(12)?;

    let base = deserialize_sequence_number(&bytes[0..8], endianness)?;
    let num_bits = deserialize_ulong(&bytes[8..12], endianness)?;

    // Get bitmaps from "long"s that follow the numBits field in the message
    // Note that the amount of bitmaps that are included in the message are 
    // determined by the number of bits (32 max per bitmap, and a max of 256 in 
    // total which means max 8 bitmap "long"s)
    let m = ((num_bits as usize) + 31) / 32;        
    let mut bitmaps = Vec::with_capacity(m);
    for i in 0..m {
        let index_of_byte_current_bitmap = 12 + i * 4;
        bytes.check_size_bigger_equal_than(index_of_byte_current_bitmap+4)?;
        bitmaps.push(deserialize_long(&bytes[index_of_byte_current_bitmap..index_of_byte_current_bitmap+4], endianness)?);
    };
    // Interpet the bitmaps and insert the sequence numbers if they are encode in the bitmaps
    let mut set = BTreeSet::new(); 
    for delta_n in 0..num_bits {
        let bitmask = 1 << (31 - delta_n % 32);
        if  bitmaps[delta_n as usize / 32] & bitmask == bitmask {               
            let seq_num = delta_n as i64 + base;
            set.insert(seq_num);
        }
    }
    Ok(SequenceNumberSet::new(base, set))
}

pub fn serialize_fragment_number(fragment_number: &FragmentNumber, writer: &mut impl std::io::Write, endianness: Endianness) -> UdpPsmMappingResult<()> {
    serialize_ulong(&fragment_number, writer, endianness)?;
    Ok(())
}    

pub fn deserialize_fragment_number(bytes: &[u8], endianness: Endianness) -> UdpPsmMappingResult<FragmentNumber> {
    Ok(deserialize_ulong(bytes, endianness)?)
}


pub fn serialize_fragment_number_set(fragment_number_set: &FragmentNumberSet, writer: &mut impl std::io::Write, endianness: Endianness) -> UdpPsmMappingResult<()> {
    let num_bits = if fragment_number_set.set().is_empty() {
        0 
    } else {
        fragment_number_set.set().iter().last().unwrap() - fragment_number_set.base() + 1
    };
    let m = ((num_bits + 31) / 32) as usize;
    let mut bitmaps = vec![0_i32; m];
    serialize_fragment_number(&fragment_number_set.base(), writer, endianness)?;
    serialize_ulong(&num_bits, writer, endianness)?;
    for seq_num in fragment_number_set.set().iter() {
        let delta_n = (seq_num - fragment_number_set.base()) as usize;
        let bitmap_i = delta_n / 32;
        let bitmask = 1 << (31 - delta_n % 32);
        bitmaps[bitmap_i] |= bitmask;
    };
    for bitmap in bitmaps {
        serialize_long(&bitmap, writer, endianness)?;
    }
    Ok(())
}

pub fn deserialize_fragment_number_set(bytes: &[u8], endianness: Endianness) -> UdpPsmMappingResult<FragmentNumberSet> {
    bytes.check_size_bigger_equal_than(8)?;
    let base = deserialize_fragment_number(&bytes[0..4], endianness)?;
    let num_bits = deserialize_ulong(&bytes[4..8], endianness)? as usize;

    // Get bitmaps from "long"s that follow the numBits field in the message
    // Note that the amount of bitmaps that are included in the message are 
    // determined by the number of bits (32 max per bitmap, and a max of 256 in 
    // total which means max 8 bitmap "long"s)
    let m = (num_bits + 31) / 32;        
    let mut bitmaps = Vec::with_capacity(m);
    for i in 0..m {
        let index_of_byte_current_bitmap = 8 + i * 4;
        bytes.check_size_bigger_equal_than(index_of_byte_current_bitmap+4)?;
        bitmaps.push(deserialize_long(&bytes[index_of_byte_current_bitmap..index_of_byte_current_bitmap+4], endianness)?);
    };
    // Interpet the bitmaps and insert the sequence numbers if they are encode in the bitmaps
    let mut set = BTreeSet::new(); 
    for delta_n in 0..num_bits {
        let bitmask = 1 << (31 - delta_n % 32);
        if  bitmaps[delta_n / 32] & bitmask == bitmask {               
            let frag_num = delta_n as u32 + base;
            set.insert(frag_num);
        }
    }
    Ok(FragmentNumberSet::new(base, set))
}    


pub fn serialize_timestamp(timestamp: &Timestamp, writer: &mut impl std::io::Write, endianness: Endianness) -> UdpPsmMappingResult<()>{
    serialize_ulong(&timestamp.seconds(), writer, endianness)?;
    serialize_ulong(&timestamp.fraction(), writer, endianness)?;
    Ok(())
}

pub fn deserialize_timestamp(bytes: &[u8], endianness: Endianness) -> UdpPsmMappingResult<Timestamp> {
    bytes.check_size_equal(8)?;

    let seconds = deserialize_ulong(&bytes[0..4], endianness)?;
    let fraction = deserialize_ulong(&bytes[4..8], endianness)?;

    Ok(Time::new(seconds, fraction))
}


pub fn serialize_count(count: &Count, writer: &mut impl std::io::Write, endianness: Endianness) -> UdpPsmMappingResult<()> {
    serialize_long(&count, writer, endianness)?;
    Ok(())
}

pub fn deserialize_count(bytes: &[u8], endianness: Endianness) -> UdpPsmMappingResult<Count> {
    let value = deserialize_long(bytes, endianness)?;
    Ok(value)
}


fn serialize_locator(locator: &types::Locator, writer: &mut impl std::io::Write, endianness: Endianness) -> UdpPsmMappingResult<()> {
    serialize_long(&locator.kind(), writer, endianness)?;
    serialize_ulong(&locator.port(), writer, endianness)?;
    writer.write(locator.address())?;
    Ok(())
}

fn deserialize_locator(bytes: &[u8], endianness: Endianness) -> UdpPsmMappingResult<types::Locator> {
    bytes.check_size_equal(24)?;
    let kind = deserialize_long(&bytes[0..4], endianness)?;
    let port = deserialize_ulong(&bytes[4..8], endianness)?;
    let address = bytes[8..24].try_into()?;
    Ok(types::Locator::new(kind, port, address))
}

pub fn serialize_locator_list(locator_list: &LocatorList, writer: &mut impl std::io::Write, endianness: Endianness) -> UdpPsmMappingResult<()> {
    serialize_ulong(&(locator_list.len() as u32), writer, endianness)?;
    for locator in locator_list {
        serialize_locator(locator, writer, endianness)?;
    };
    Ok(())
}

pub fn deserialize_locator_list(bytes: &[u8], endianness: Endianness) -> UdpPsmMappingResult<LocatorList> {
    bytes.check_size_bigger_equal_than(4)?;
    let size = bytes.len();
    let num_locators = deserialize_ulong(&bytes[0..4], endianness)?;
    let mut locator_list = Vec::new();
    let mut index = 4;
    while index < size && locator_list.len() < num_locators as usize {
        bytes.check_size_bigger_equal_than(index+24)?;
        let locator = deserialize_locator(&bytes[index..index+24], endianness)?;
        index += 24;
        locator_list.push(locator);
    };
    Ok(locator_list)
}

pub fn serialize_parameter_list(parameter_list: &ParameterList, writer: &mut impl std::io::Write, endianness: Endianness) -> UdpPsmMappingResult<()> {
    for parameter in &parameter_list.parameter {
        serialize_short(&parameter.parameter_id(), writer, endianness)?;
        serialize_short(&parameter.length(), writer, endianness)?;
        writer.write(parameter.value())?;
    }
    serialize_ushort(&1, writer, endianness)?; // PID_SENTINEL
    writer.write(&[0,0])?;  // LENGTH
    Ok(())
}

pub fn deserialize_parameter_list(bytes: &[u8], endianness: Endianness) -> UdpPsmMappingResult<ParameterList> {
    let mut parameter_start_index: usize = 0;
    let mut parameter_list = ParameterList::new();
    loop {
        let (parameter_id, length) = match endianness {
            Endianness::BigEndian => {
                let parameter_id = i16::from_be_bytes(bytes[parameter_start_index..parameter_start_index+2].try_into().unwrap());
                let length = i16::from_be_bytes(bytes[parameter_start_index+2..parameter_start_index+4].try_into().unwrap());
                (parameter_id, length)
            },
            Endianness::LittleEndian => {
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

        parameter_list.parameter.push(rust_dds_api::types::Parameter::new(parameter_id, value));
    }

    Ok(parameter_list)
}


pub fn serialize_serialized_data(serialized_data: &SerializedData, writer: &mut impl std::io::Write) -> UdpPsmMappingResult<()> {
    writer.write(serialized_data.as_slice())?;
    Ok(())
}

pub fn deserialize_serialized_data(bytes: &[u8]) -> UdpPsmMappingResult<SerializedData> {
    Ok(Vec::from(bytes))
}


// impl SubmessageElement for SerializedDataFragment {
//     fn serialize(&self, writer: &mut impl std::io::Write, _endianness: Endianness) -> RtpsSerdesResult<()> {
//         writer.write(self.0.as_slice())?;
//         Ok(())
//     }

//     fn deserialize(bytes: &[u8], _endianness: Endianness) -> RtpsSerdesResult<Self> {
//         Ok(SerializedDataFragment(Vec::from(bytes)))
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::types::Time;
    use crate::types::constants;

    // ///////// The GuidPrefix, and EntityId Tests ///////////////////////////
    #[test]
    fn invalid_guid_prefix_deserialization() {
        let too_big_vec = [1,2,3,4,5,6,7,8,9,10,11,12,13,14];

        let expected_error = deserialize_guid_prefix(&too_big_vec);
        match expected_error {
            Err(UdpPsmMappingError::WrongSize) => assert!(true),
            _ => assert!(false),
        };

        let too_small_vec = [1,2];

        let expected_error = deserialize_guid_prefix(&too_small_vec);
        match expected_error {
            Err(UdpPsmMappingError::WrongSize) => assert!(true),
            _ => assert!(false),
        };
    }
       
    #[test]
    fn entity_id_serialization_deserialization() {
        let mut vec = Vec::new();
        let test_entity_id = constants::ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER;

        const TEST_ENTITY_ID_BYTES : [u8;4] = [0, 0x02, 0x00, 0xc4];
        serialize_entity_id(&test_entity_id, &mut vec).unwrap();
        assert_eq!(vec, TEST_ENTITY_ID_BYTES);
        assert_eq!(deserialize_entity_id(&vec).unwrap(), test_entity_id);
    }

    // ///////// VendorId Tests ///////////////////////////////////////////////
    #[test]
    fn invalid_vendor_id_deserialization() {
        let too_big_vec = [1,2,3,4,5,6,7,8,9,10,11,12,13,14,1,2,3,4,5,6,7,8,9,10,11,12,13,14,1,2,3,4,5,6,7,8,9,10,11,12,13,14,];

        let expected_error = deserialize_vendor_id(&too_big_vec);
        match expected_error {
            Err(UdpPsmMappingError::WrongSize) => assert!(true),
            _ => assert!(false),
        };

        let too_small_vec = [1];

        let expected_error = deserialize_vendor_id(&too_small_vec);
        match expected_error {
            Err(UdpPsmMappingError::WrongSize) => assert!(true),
            _ => assert!(false),
        };
    }

    // ///////// ProtocolVersion Tests ////////////////////////////////////////
    #[test]
    fn invalid_protocol_version_deserialization() {
        let too_big_vec = [1,2,3,4,5,6,7,8,9,10,11,12,13,14,1,2,3,4,5,6,7,8,9,10,11,12,13,14,1,2,3,4,5,6,7,8,9,10,11,12,13,14,];

        let expected_error = deserialize_protocol_version(&too_big_vec);
        match expected_error {
            Err(UdpPsmMappingError::WrongSize) => assert!(true),
            _ => assert!(false),
        };

        let too_small_vec = [1];

        let expected_error = deserialize_protocol_version(&too_small_vec);
        match expected_error {
            Err(UdpPsmMappingError::WrongSize) => assert!(true),
            _ => assert!(false),
        };
    }

    // ///////// SequenceNumber Tests /////////////////////////////////////////
 
    #[test]
    fn sequence_number_serialization_deserialization_big_endian() {
        let mut vec = Vec::new();
        let test_sequence_number = 1987612345679;

        
        const TEST_SEQUENCE_NUMBER_BIG_ENDIAN : [u8;8] = [0x00, 0x00, 0x01, 0xCE, 0xC6, 0xED, 0x85, 0x4F];
        serialize_sequence_number(&test_sequence_number, &mut vec, Endianness::BigEndian).unwrap();
        assert_eq!(vec, TEST_SEQUENCE_NUMBER_BIG_ENDIAN);
        assert_eq!(deserialize_sequence_number(&vec, Endianness::BigEndian).unwrap(), test_sequence_number);
    }

    #[test]
    fn sequence_number_serialization_deserialization_little_endian() {
        let mut vec = Vec::new();
        let test_sequence_number = 1987612345679;

        
        const TEST_SEQUENCE_NUMBER_LITTLE_ENDIAN : [u8;8] = [0xCE, 0x01, 0x00, 0x00, 0x4F, 0x85, 0xED, 0xC6];
        serialize_sequence_number(&test_sequence_number, &mut vec, Endianness::LittleEndian).unwrap();
        assert_eq!(vec, TEST_SEQUENCE_NUMBER_LITTLE_ENDIAN);
        assert_eq!(deserialize_sequence_number(&vec, Endianness::LittleEndian).unwrap(), test_sequence_number);
    }

    #[test]
    fn sequence_number_serialization_deserialization_multiple_combinations() {
        let mut vec = Vec::new();
        
        {
            let test_sequence_number_i64_max = std::i64::MAX;
            serialize_sequence_number(&test_sequence_number_i64_max, &mut vec, Endianness::LittleEndian).unwrap();
            assert_eq!(deserialize_sequence_number(&vec, Endianness::LittleEndian).unwrap(), test_sequence_number_i64_max);
            vec.clear();

            serialize_sequence_number(&test_sequence_number_i64_max, &mut vec, Endianness::BigEndian).unwrap();
            assert_eq!(deserialize_sequence_number(&vec, Endianness::BigEndian).unwrap(), test_sequence_number_i64_max);
            vec.clear();
        }

        {
            let test_sequence_number_i64_min = std::i64::MIN;
            serialize_sequence_number(&test_sequence_number_i64_min, &mut vec, Endianness::LittleEndian).unwrap();
            assert_eq!(deserialize_sequence_number(&vec, Endianness::LittleEndian).unwrap(), test_sequence_number_i64_min);
            vec.clear();

            serialize_sequence_number( &test_sequence_number_i64_min, &mut vec, Endianness::BigEndian).unwrap();
            assert_eq!(deserialize_sequence_number(&vec, Endianness::BigEndian).unwrap(), test_sequence_number_i64_min);
            vec.clear();
        }

        {
            let test_sequence_number_zero = 0;
            serialize_sequence_number(&test_sequence_number_zero, &mut vec, Endianness::LittleEndian).unwrap();
            assert_eq!(deserialize_sequence_number(&vec, Endianness::LittleEndian).unwrap(), test_sequence_number_zero);
            vec.clear();

            serialize_sequence_number(&test_sequence_number_zero, &mut vec, Endianness::BigEndian).unwrap();
            assert_eq!(deserialize_sequence_number(&vec, Endianness::BigEndian).unwrap(), test_sequence_number_zero);
            vec.clear();
        }
    }

    #[test]
    fn invalid_sequence_number_deserialization() {
        let wrong_vec = [1,2,3,4];

        let expected_error = deserialize_sequence_number(&wrong_vec, Endianness::LittleEndian);
        match expected_error {
            Err(UdpPsmMappingError::WrongSize) => assert!(true),
            _ => assert!(false),
        };
    }


//     // /////////////////////// SequenceNumberSet Tests ////////////////////////
    
    #[test]
    fn deserialize_sequence_number_set_empty() {
        let expected = SequenceNumberSet::new(3, [].iter().cloned().collect());
        let bytes = vec![
            0, 0, 0, 0, // base
            0, 0, 0, 3, // base
            0, 0, 0, 0, // num bits
        ];
        let result = deserialize_sequence_number_set(&bytes, Endianness::BigEndian).unwrap();
        assert_eq!(expected, result);
    }
    
    #[test]
    fn deserialize_sequence_number_set_one_bitmap_be() {
        let expected = SequenceNumberSet::new(3, [3, 4].iter().cloned().collect());
        let bytes = vec![
            0, 0, 0, 0, // base
            0, 0, 0, 3, // base
            0, 0, 0, 2, // num bits
            0b_11000000, 0b_00000000, 0b_00000000, 0b_00000000, 
        ];
        let result = deserialize_sequence_number_set(&bytes, Endianness::BigEndian).unwrap();
        assert_eq!(expected, result);
    }
    
    #[test]
        fn deserialize_sequence_number_set_one_bitmap_le() {
        let expected = SequenceNumberSet::new(3,  [3, 4].iter().cloned().collect());
        let bytes = vec![
            0, 0, 0, 0, // base
            3, 0, 0, 0, // base
            2, 0, 0, 0, // num bits
            0b_00000000, 0b_00000000, 0b_00000000, 0b_11000000, 
        ];
        let result = deserialize_sequence_number_set(&bytes, Endianness::LittleEndian).unwrap();
        assert_eq!(expected, result);
    }
    
    #[test]
    fn deserialize_sequence_number_set_multiple_bitmaps() {
        let expected = SequenceNumberSet::new( 1000, [1001, 1003, 1032, 1033].iter().cloned().collect());
        let bytes = vec![
            0, 0, 0, 0, // base
            0, 0, 3, 232, // base
            0, 0, 0, 34, // num bits
            0b_01010000, 0b_00000000, 0b_00000000, 0b_00000000, 
            0b_11000000, 0b_00000000, 0b_00000000, 0b_00000000, 
        ];
        let result = deserialize_sequence_number_set(&bytes, Endianness::BigEndian).unwrap();
        assert_eq!(expected, result);
    }
    
    #[test]
    fn deserialize_sequence_number_set_max_bitmaps_big_endian() {
        let expected = SequenceNumberSet::new( 1000, [1000, 1255].iter().cloned().collect());
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
        let result = deserialize_sequence_number_set(&bytes, Endianness::BigEndian).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn deserialize_sequence_number_set_max_bitmaps_little_endian() {
        let expected = SequenceNumberSet::new(1000, [1000, 1255].iter().cloned().collect());
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
        let result = deserialize_sequence_number_set(&bytes, Endianness::LittleEndian).unwrap();
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
        let sequence_number_set = deserialize_sequence_number_set(&bytes, Endianness::BigEndian).unwrap();
        assert!(!sequence_number_set.set().contains(&1234));
        assert!(!sequence_number_set.set().contains(&1235));
        assert!(sequence_number_set.set().contains(&1236));
        assert!(sequence_number_set.set().contains(&1237));
        for seq_num in 1238..1245 {
            assert!(!sequence_number_set.set().contains(&seq_num));
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
        let sequence_number_set = deserialize_sequence_number_set(&bytes, Endianness::LittleEndian).unwrap();
        assert!(!sequence_number_set.set().contains(&1234));
        assert!(!sequence_number_set.set().contains(&1235));
        assert!(sequence_number_set.set().contains(&1236));
        assert!(sequence_number_set.set().contains(&1237));
        for seq_num in 1238..1245 {
            assert!(!sequence_number_set.set().contains(&seq_num));
        }
    }
        
    #[test]
    fn test_serialize_sequence_number_set() {
        let set = SequenceNumberSet::new(3, [3, 4].iter().cloned().collect());
        let mut writer = Vec::new();
        serialize_sequence_number_set(&set, &mut writer, Endianness::BigEndian).unwrap();
        let expected = vec![
            0, 0, 0, 0, // base
            0, 0, 0, 3, // base
            0, 0, 0, 2, // num bits
            0b_11000000, 0b_00000000, 0b_00000000, 0b_00000000, 
        ];
        assert_eq!(expected, writer);
    
    
        let set = SequenceNumberSet::new(1, [3, 4].iter().cloned().collect());
        let mut writer = Vec::new();
        serialize_sequence_number_set(&set, &mut writer, Endianness::LittleEndian).unwrap();
        let expected = vec![
            0, 0, 0, 0, // base
            1, 0, 0, 0, // base
            4, 0, 0, 0, // num bits
            0b_00000000, 0b_00000000, 0b_00000000, 0b_00110000, 
        ];
        assert_eq!(expected, writer);
    
        let mut writer = Vec::new();
        serialize_sequence_number_set(&set, &mut writer, Endianness::BigEndian).unwrap();
        let expected = vec![
            0, 0, 0, 0, // base
            0, 0, 0, 1, // base
            0, 0, 0, 4, // num bits
            0b_00110000, 0b_00000000, 0b_00000000, 0b_00000000,  
        ];
        assert_eq!(expected, writer);
    
    
        let set = SequenceNumberSet::new(1000, [1001, 1003, 1032, 1033].iter().cloned().collect());
        let mut writer = Vec::new();
        serialize_sequence_number_set(&set, &mut writer, Endianness::BigEndian).unwrap();
        let expected = vec![
            0, 0, 0, 0, // base
            0, 0, 3, 232, // base
            0, 0, 0, 34, // num bits
            0b_01010000, 0b_00000000, 0b_00000000, 0b_00000000, 
            0b_11000000, 0b_00000000, 0b_00000000, 0b_00000000, 
        ];
        assert_eq!(expected, writer);
    }

    // //////////////////////// FragmentNumber Tests //////////////////////////
    
    #[test]
    fn test_serialize_fragment_number() {
        let fragment_number = 100;
        let expected = vec![
            100, 0, 0, 0,
        ];
        let mut writer = Vec::new();
        serialize_fragment_number(&fragment_number, &mut writer, Endianness::LittleEndian).unwrap();
        assert_eq!(expected, writer);
    }

    #[test]
    fn test_deserialize_fragment_number() {
        let expected = 100;
        let bytes = vec![
            100, 0, 0, 0,
        ];
        let result = deserialize_fragment_number(&bytes, Endianness::LittleEndian).unwrap();
        assert_eq!(expected, result);
    }


    // /////////////////////// FragmentNumberSet Tests ////////////////////////



    #[test]
    fn test_deserialize_fragment_number_set() {
        let expected = FragmentNumberSet::new(3,[3, 4].iter().cloned().collect());
        let bytes = vec![
            3, 0, 0, 0, // base
            2, 0, 0, 0, // num bits
            0b_00000000, 0b_00000000, 0b_00000000, 0b_11000000, 
        ];
        let result = deserialize_fragment_number_set(&bytes, Endianness::LittleEndian).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_serialize_fragment_number_set() {
        let set = FragmentNumberSet::new(3,[3, 4].iter().cloned().collect());
        let mut writer = Vec::new();
        serialize_fragment_number_set(&set, &mut writer, Endianness::BigEndian).unwrap();
        let expected = vec![
            0, 0, 0, 3, // base
            0, 0, 0, 2, // num bits
            0b_11000000, 0b_00000000, 0b_00000000, 0b_00000000, 
        ];
        assert_eq!(expected, writer);
    }

    // /////////////////////// Timestamp Tests ////////////////////////////////
    #[test]
    fn test_time_serialization_deserialization_big_endian() {
        let mut vec = Vec::new();
        let test_time = Time::new(1234567, 98765432);

        
        const TEST_TIME_BIG_ENDIAN : [u8;8] = [0x00, 0x12, 0xD6, 0x87, 0x05, 0xE3, 0x0A, 0x78];
        serialize_timestamp(&test_time, &mut vec, Endianness::BigEndian).unwrap();
        assert_eq!(vec, TEST_TIME_BIG_ENDIAN);
        assert_eq!(deserialize_timestamp(&vec, Endianness::BigEndian).unwrap(), test_time);
    }

    #[test]
    fn test_time_serialization_deserialization_little_endian() {
        let mut vec = Vec::new();
        let test_time = Time::new(1234567, 98765432);
        
        const TEST_TIME_LITTLE_ENDIAN : [u8;8] = [0x87, 0xD6, 0x12, 0x00, 0x78, 0x0A, 0xE3, 0x05];
        serialize_timestamp(&test_time, &mut vec, Endianness::LittleEndian).unwrap();
        assert_eq!(vec, TEST_TIME_LITTLE_ENDIAN);
        assert_eq!(deserialize_timestamp(&vec, Endianness::LittleEndian).unwrap(), test_time);
    }

    #[test]
    fn test_invalid_time_deserialization() {
        let wrong_vec = vec![1,2,3,4];

        let expected_error = deserialize_timestamp(&wrong_vec, Endianness::LittleEndian);
        match expected_error {
            Err(UdpPsmMappingError::WrongSize) => assert!(true),
            _ => assert!(false),
        };
    }

    // /////////////////////// Count Tests ////////////////////////////////////
    // todo!  

    // /////////////////////// LocatorList Tests ////////////////////////
     
     #[test]
    fn serialize_locator_simple() {
        let locator = types::Locator::new(100, 200, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
        let expected = vec![
            100, 0, 0, 0, // kind
            200, 0, 0, 0, // port
             1,  2,  3,  4, // address
             5,  6,  7,  8, // address
             9, 10, 11, 12, // address
            13, 14, 15, 16, // address
        ];
        let mut writer = Vec::new();
        serialize_locator(&locator, &mut writer, Endianness::LittleEndian).unwrap();
        assert_eq!(expected, writer);
    }
    
    #[test]
    fn deserialize_locator_simple() {
        let expected = types::Locator::new(100, 200, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
        let bytes = vec![
            100, 0, 0, 0, // kind
            200, 0, 0, 0, // port
             1,  2,  3,  4, // address
             5,  6,  7,  8, // address
             9, 10, 11, 12, // address
            13, 14, 15, 16, // address
        ];
        let result = deserialize_locator(&bytes, Endianness::LittleEndian).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn invalid_locator_deserialization() {
        let too_big_vec = [1,2,3,4,5,6,7,8,9,10,11,12,13,14,1,2,3,4,5,6,7,8,9,10,11,12,13,14,1,2,3,4,5,6,7,8,9,10,11,12,13,14,];

        let expected_error = deserialize_locator(&too_big_vec, Endianness::LittleEndian);
        match expected_error {
            Err(UdpPsmMappingError::WrongSize) => assert!(true),
            _ => assert!(false),
        };

        let too_small_vec = [1,2];

        let expected_error = deserialize_locator(&too_small_vec, Endianness::BigEndian);
        match expected_error {
            Err(UdpPsmMappingError::WrongSize) => assert!(true),
            _ => assert!(false),
        };
    }

    #[test]
    fn test_serialize_locator_list() {
        let address = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let locator_list = vec![
            types::Locator::new(100, 200, address),
            types::Locator::new(101, 201, address),
        ];
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
        serialize_locator_list(&locator_list, &mut writer, Endianness::LittleEndian).unwrap();
        assert_eq!(expected, writer);
    }

    
    #[test]
    fn test_deserialize_locator_list() {
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
        let expected = vec![
            types::Locator::new(100, 200, address),
            types::Locator::new(101, 201, address),
        ];
        let result = deserialize_locator_list(&bytes, Endianness::LittleEndian).unwrap();
        assert_eq!(expected, result);
    }

    
    #[test]
    fn serialize_deserialize_ushort(){
        let mut buf = Vec::new();

        let val = 123;

        serialize_ushort(&val, &mut buf, Endianness::LittleEndian).unwrap();
        assert_eq!(buf, [123, 0]);
        assert_eq!(deserialize_ushort(&buf, Endianness::LittleEndian).unwrap(), val);
        buf.clear();

        serialize_ushort(&val, &mut buf, Endianness::BigEndian).unwrap();
        assert_eq!(buf, [0, 123]);
        assert_eq!(deserialize_ushort(&buf, Endianness::BigEndian).unwrap(), val);
        buf.clear();


        let max = u16::MAX;

        serialize_ushort(&max, &mut buf, Endianness::LittleEndian).unwrap();
        assert_eq!(buf, [0xFF, 0xFF]);
        assert_eq!(deserialize_ushort(&buf, Endianness::LittleEndian).unwrap(), max);
        buf.clear();

        serialize_ushort(&max, &mut buf, Endianness::BigEndian).unwrap();
        assert_eq!(buf, [0xFF, 0xFF]);
        assert_eq!(deserialize_ushort(&buf, Endianness::BigEndian).unwrap(), max);
        buf.clear();

        let min = u16::MIN;

        serialize_ushort(&min, &mut buf, Endianness::LittleEndian).unwrap();
        assert_eq!(buf, [0x00, 0x00]);
        assert_eq!(deserialize_ushort(&buf, Endianness::LittleEndian).unwrap(), min);
        buf.clear();

        serialize_ushort(&min, &mut buf, Endianness::BigEndian).unwrap();
        assert_eq!(buf, [0x00, 0x00]);
        assert_eq!(deserialize_ushort(&buf, Endianness::BigEndian).unwrap(), min);
        buf.clear();
    }

    #[test]
    fn invalid_ushort_deserialize() {
        let buf: [u8; 1] = [1];
        let result = deserialize_ushort(&buf, Endianness::BigEndian);
        match result {
            Err(UdpPsmMappingError::WrongSize) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn serialize_deserialize_short(){
        let mut buf = Vec::new();

        let val = 123;

        serialize_short(&val, &mut buf, Endianness::LittleEndian).unwrap();
        assert_eq!(buf, [123, 0]);
        assert_eq!(deserialize_short(&buf, Endianness::LittleEndian).unwrap(), val);
        buf.clear();

        serialize_short(&val, &mut buf, Endianness::BigEndian).unwrap();
        assert_eq!(buf, [0, 123]);
        assert_eq!(deserialize_short(&buf, Endianness::BigEndian).unwrap(), val);
        buf.clear();


        let max = i16::MAX;

        serialize_short(&max, &mut buf, Endianness::LittleEndian).unwrap();
        assert_eq!(buf, [0xFF, 0x7F]);
        assert_eq!(deserialize_short(&buf, Endianness::LittleEndian).unwrap(), max);
        buf.clear();

        serialize_short(&max, &mut buf, Endianness::BigEndian).unwrap();
        assert_eq!(buf, [0x7F, 0xFF]);
        assert_eq!(deserialize_short(&buf, Endianness::BigEndian).unwrap(), max);
        buf.clear();

        let min = i16::MIN;

        serialize_short(&min, &mut buf, Endianness::LittleEndian).unwrap();
        assert_eq!(buf, [0x00, 0x80]);
        assert_eq!(deserialize_short(&buf, Endianness::LittleEndian).unwrap(), min);
        buf.clear();

        serialize_short(&min, &mut buf, Endianness::BigEndian).unwrap();
        assert_eq!(buf, [0x80, 0x00]);
        assert_eq!(deserialize_short(&buf, Endianness::BigEndian).unwrap(), min);
        buf.clear();
    }

    #[test]
    fn invalid_short_deserialize() {
        let buf: [u8; 1] = [1];
        let result = deserialize_short(&buf, Endianness::BigEndian);
        match result {
            Err(UdpPsmMappingError::WrongSize) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn serialize_deserialize_long(){
        let mut buf = Vec::new();

        let val = 1230;

        serialize_long(&val, &mut buf, Endianness::LittleEndian).unwrap();
        assert_eq!(buf, [0xCE, 0x04, 0, 0]);
        assert_eq!(deserialize_long(&buf, Endianness::LittleEndian).unwrap(), val);
        buf.clear();

        serialize_long(&val, &mut buf, Endianness::BigEndian).unwrap();
        assert_eq!(buf, [0, 0, 0x04, 0xCE]);
        assert_eq!(deserialize_long(&buf, Endianness::BigEndian).unwrap(), val);
        buf.clear();


        let max = i32::MAX;

        serialize_long(&max, &mut buf, Endianness::LittleEndian).unwrap();
        assert_eq!(buf, [0xFF, 0xFF, 0xFF, 0x7F]);
        assert_eq!(deserialize_long(&buf, Endianness::LittleEndian).unwrap(), max);
        buf.clear();

        serialize_long(&max, &mut buf, Endianness::BigEndian).unwrap();
        assert_eq!(buf, [0x7F, 0xFF, 0xFF, 0xFF]);
        assert_eq!(deserialize_long(&buf, Endianness::BigEndian).unwrap(), max);
        buf.clear();

        let min = i32::MIN;

        serialize_long(&min, &mut buf, Endianness::LittleEndian).unwrap();
        assert_eq!(buf, [0x00, 0x00, 0x00, 0x80]);
        assert_eq!(deserialize_long(&buf, Endianness::LittleEndian).unwrap(), min);
        buf.clear();

        serialize_long(&min, &mut buf, Endianness::BigEndian).unwrap();
        assert_eq!(buf, [0x80, 0x00, 0x00, 0x00]);
        assert_eq!(deserialize_long(&buf, Endianness::BigEndian).unwrap(), min);
        buf.clear();
    }

    #[test]
    fn invalid_long_deserialize() {
        let buf: [u8; 3] = [1, 2, 3];
        let result = deserialize_long(&buf, Endianness::BigEndian);
        match result {
            Err(UdpPsmMappingError::WrongSize) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn serialize_deserialize_ulong(){
        let mut buf = Vec::new();

        let val = 1230;

        serialize_ulong(&val, &mut buf, Endianness::LittleEndian).unwrap();
        assert_eq!(buf, [0xCE, 0x04, 0, 0]);
        assert_eq!(deserialize_ulong(&buf, Endianness::LittleEndian).unwrap(), val);
        buf.clear();

        serialize_ulong(&val, &mut buf, Endianness::BigEndian).unwrap();
        assert_eq!(buf, [0, 0, 0x04, 0xCE]);
        assert_eq!(deserialize_ulong(&buf, Endianness::BigEndian).unwrap(), val);
        buf.clear();


        let max = u32::MAX;

        serialize_ulong(&max, &mut buf, Endianness::LittleEndian).unwrap();
        assert_eq!(buf, [0xFF, 0xFF, 0xFF, 0xFF]);
        assert_eq!(deserialize_ulong(&buf, Endianness::LittleEndian).unwrap(), max);
        buf.clear();

        serialize_ulong(&max, &mut buf, Endianness::BigEndian).unwrap();
        assert_eq!(buf, [0xFF, 0xFF, 0xFF, 0xFF]);
        assert_eq!(deserialize_ulong(&buf, Endianness::BigEndian).unwrap(), max);
        buf.clear();

        let min = u32::MIN;

        serialize_ulong(&min, &mut buf, Endianness::LittleEndian).unwrap();
        assert_eq!(buf, [0x00, 0x00, 0x00, 0x00]);
        assert_eq!(deserialize_ulong(&buf, Endianness::LittleEndian).unwrap(), min);
        buf.clear();

        serialize_ulong(&min, &mut buf, Endianness::BigEndian).unwrap();
        assert_eq!(buf, [0x00, 0x00, 0x00, 0x00]);
        assert_eq!(deserialize_ulong(&buf, Endianness::BigEndian).unwrap(), min);
        buf.clear();
    }

    #[test]
    fn invalid_ulong_deserialize() {
        let buf: [u8; 3] = [1, 2, 3];
        let result = deserialize_ulong(&buf, Endianness::BigEndian);
        match result {
            Err(UdpPsmMappingError::WrongSize) => assert!(true),
            _ => assert!(false),
        }
    }
}
