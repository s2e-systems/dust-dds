/// 
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// Table 8.2 - Types of the attributes that appear in the RTPS Entities and Classes
///  

use std::convert::{TryInto, TryFrom, };
use num_derive::FromPrimitive;
use crate::serdes::{RtpsSerialize, RtpsDeserialize, Endianness, RtpsSerdesResult, SizeCheckers, RtpsSerdesError, };
use crate::primitive_types::{Long, ULong, };
use crate::inline_qos_types::StatusInfo;


pub mod constants {
    use super::{VendorId, EntityId, ProtocolVersion, EntityKey, EntityKind, SequenceNumber};

    pub const VENDOR_ID: VendorId = VendorId([99,99]);

    pub const SEQUENCE_NUMBER_UNKNOWN : SequenceNumber = SequenceNumber(i64::MIN);

    pub const PROTOCOL_VERSION_2_1 : ProtocolVersion = ProtocolVersion{major: 2, minor: 1};
    pub const PROTOCOL_VERSION_2_2 : ProtocolVersion = ProtocolVersion{major: 2, minor: 2};
    pub const PROTOCOL_VERSION_2_4 : ProtocolVersion = ProtocolVersion{major: 2, minor: 4};

    pub const ENTITYID_UNKNOWN: EntityId = EntityId {
        entity_key: EntityKey([0, 0, 0x00]),
        entity_kind: EntityKind::UserDefinedUnknown,
    };

    pub const ENTITYID_PARTICIPANT: EntityId = EntityId {
        entity_key: EntityKey([0, 0, 0x01]),
        entity_kind: EntityKind::BuiltInParticipant,
    };

    pub const ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER: EntityId = EntityId {
        entity_key: EntityKey([0, 0, 0x02]),
        entity_kind: EntityKind::BuiltInWriterWithKey,
    };

    pub const ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR: EntityId = EntityId {
        entity_key: EntityKey([0, 0, 0x02]),
        entity_kind: EntityKind::BuiltInReaderWithKey,
    };

    pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER: EntityId = EntityId {
        entity_key: EntityKey([0, 0, 0x03]),
        entity_kind: EntityKind::BuiltInWriterWithKey,
    };

    pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR: EntityId = EntityId {
        entity_key: EntityKey([0, 0, 0x03]),
        entity_kind: EntityKind::BuiltInReaderWithKey,
    };

    pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER: EntityId = EntityId {
        entity_key: EntityKey([0, 0, 0x04]),
        entity_kind: EntityKind::BuiltInWriterWithKey,
    };

    pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR: EntityId = EntityId {
        entity_key: EntityKey([0, 0, 0x04]),
        entity_kind: EntityKind::BuiltInReaderWithKey,
    };

    pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER: EntityId = EntityId {
        entity_key: EntityKey([0, 0x01, 0x00]),
        entity_kind: EntityKind::BuiltInWriterWithKey,
    };

    pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR: EntityId = EntityId {
        entity_key: EntityKey([0, 0x01, 0x00]),
        entity_kind: EntityKind::BuiltInReaderWithKey,
    };

    pub const ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER: EntityId = EntityId {
        entity_key: EntityKey([0, 0x02, 0x00]),
        entity_kind: EntityKind::BuiltInWriterWithKey,
    };

    pub const ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER: EntityId = EntityId {
        entity_key: EntityKey([0, 0x02, 0x00]),
        entity_kind: EntityKind::BuiltInReaderWithKey,
    };
}


#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct GUID {
    prefix: GuidPrefix,
    entity_id: EntityId,
}

impl GUID {
    pub fn new(prefix: GuidPrefix, entity_id: EntityId) -> GUID {
        GUID { prefix, entity_id }
    }

    pub fn prefix(&self) -> &GuidPrefix {
        &self.prefix
    }

    pub fn entity_id(&self) -> &EntityId {
        &self.entity_id
    }
}



#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub struct GuidPrefix(pub [u8; 12]);

impl RtpsSerialize for GuidPrefix {
    fn serialize(&self, writer: &mut impl std::io::Write, _endianness: Endianness) -> RtpsSerdesResult<()> {
        writer.write(&self.0)?;
        Ok(())
    }
}

impl RtpsDeserialize for GuidPrefix {
    fn deserialize(bytes: &[u8], _endianness: Endianness) -> RtpsSerdesResult<Self> {
        Ok(GuidPrefix(bytes[0..12].try_into()?))
    }    
}



#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct EntityKey(pub [u8;3]);

impl RtpsSerialize for EntityKey {
    fn serialize(&self, writer: &mut impl std::io::Write, _endianness: Endianness) -> RtpsSerdesResult<()>{
        writer.write(&self.0)?;
        Ok(())
    }
}

impl RtpsDeserialize for EntityKey {
    fn deserialize(bytes: &[u8], _endianness: Endianness) -> RtpsSerdesResult<Self> {
        SizeCheckers::check_size_equal(bytes, 3)?;

        Ok(EntityKey(bytes[0..3].try_into()?))
    }
}

#[derive(FromPrimitive, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum EntityKind {
    UserDefinedUnknown = 0x00,
    UserDefinedWriterWithKey = 0x02,
    UserDefinedWriterNoKey = 0x03,
    UserDefinedReaderWithKey = 0x04,
    UserDefinedReaderNoKey = 0x07,
    UserDefinedWriterGroup = 0x08,
    UserDefinedReaderGroup = 0x09,
    BuiltInUnknown = 0xc0,
    BuiltInParticipant = 0xc1,
    BuiltInWriterWithKey = 0xc2,
    BuiltInWriterNoKey = 0xc3,
    BuiltInReaderWithKey = 0xc4,
    BuiltInReaderNoKey = 0xc7,
    BuiltInWriterGroup = 0xc8,
    BuiltInReaderGroup = 0xc9,
}

impl RtpsSerialize for EntityKind {
    fn serialize(&self, writer: &mut impl std::io::Write, _endianness: Endianness) -> RtpsSerdesResult<()>{
        let entity_kind_u8 = *self as u8;
        writer.write(&[entity_kind_u8])?;
        Ok(())
    }
}

impl RtpsDeserialize for EntityKind {
    fn deserialize(bytes: &[u8], _endianness: Endianness) -> RtpsSerdesResult<Self> {
        SizeCheckers::check_size_equal(bytes, 1 /*expected_size*/)?;
        Ok(num::FromPrimitive::from_u8(bytes[0]).ok_or(RtpsSerdesError::InvalidEnumRepresentation)?)
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct EntityId {
    entity_key: EntityKey,
    entity_kind: EntityKind,
}

impl EntityId {
    pub fn new(entity_key: EntityKey, entity_kind: EntityKind) -> EntityId {
        EntityId {
            entity_key,
            entity_kind,
        }
    }
}

impl RtpsSerialize for EntityId {
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: Endianness) -> RtpsSerdesResult<()>{
        self.entity_key.serialize(writer, endianness)?;
        self.entity_kind.serialize(writer, endianness)
    }
}

impl RtpsDeserialize for EntityId{
    fn deserialize(bytes: &[u8], endianness: Endianness) -> RtpsSerdesResult<Self> {
        SizeCheckers::check_size_equal(bytes, 4 /*expected_size*/)?;
        let entity_key = EntityKey::deserialize(&bytes[0..3], endianness)?;
        let entity_kind = EntityKind::deserialize(&[bytes[3]], endianness)?;

        Ok(EntityId::new(entity_key, entity_kind))
    }
}



#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct SequenceNumber(pub i64);

impl std::ops::Sub<i64> for SequenceNumber {
    type Output = SequenceNumber;

    fn sub(self, rhs: i64) -> Self::Output {
        SequenceNumber(self.0 - rhs)
    }
}

impl std::ops::Add<i64> for SequenceNumber {
    type Output = SequenceNumber;

    fn add(self, rhs: i64) -> Self::Output {
        SequenceNumber(self.0 + rhs)
    }
}

impl RtpsSerialize for SequenceNumber {
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: Endianness) -> RtpsSerdesResult<()>{
        let msb = self.0 >> 32;
        let lsb = self.0 & 0x0000_0000_FFFF_FFFF;
        (msb as Long).serialize(writer, endianness)?;
        (lsb as ULong).serialize(writer, endianness)?;
        Ok(())
    }
}

impl RtpsDeserialize for SequenceNumber {
    fn deserialize(bytes: &[u8], endianness: Endianness) -> RtpsSerdesResult<Self> {
        SizeCheckers::check_size_equal(bytes, 8)?;

        let msb = Long::deserialize(&bytes[0..4], endianness)?;
        let lsb = ULong::deserialize(&bytes[4..8], endianness)?;

        let sequence_number = ((msb as i64) << 32) + lsb as i64;

        Ok(SequenceNumber(sequence_number))
    }
}



#[derive(PartialEq, Hash, Eq, Debug, Copy, Clone)]
pub struct Locator {
    pub kind: Long,
    pub port: ULong,
    pub address: [u8; 16],
}

impl Locator {
    pub fn new(kind: i32, port: u32, address: [u8; 16]) -> Locator {
        Locator {
            kind,
            port,
            address,
        }
    }
}

impl RtpsSerialize for Locator {
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: Endianness) -> RtpsSerdesResult<()> {
        self.kind.serialize(writer, endianness)?;
        self.port.serialize(writer, endianness)?;
        writer.write(&self.address)?;
        Ok(())
    }
}

impl RtpsDeserialize for Locator {
    fn deserialize(bytes: &[u8], endianness: Endianness) -> RtpsSerdesResult<Self> {
        let kind = Long::deserialize(&bytes[0..4], endianness)?;
        let port = ULong::deserialize(&bytes[4..8], endianness)?;
        let address = bytes[8..24].try_into()?;
        Ok(Self {kind, port, address})
    }
}



pub enum TopicKind {
    NoKey,
    WithKey,
}



#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub enum ChangeKind {
    Alive,
    AliveFiltered,
    NotAliveDisposed,
    NotAliveUnregistered,
}

impl TryFrom<StatusInfo> for ChangeKind {
    type Error = &'static str;

    fn try_from(status_info: StatusInfo) -> Result<Self, Self::Error> {
        if status_info.disposed_flag() && !status_info.unregistered_flag() && !status_info.filtered_flag() {
            Ok(ChangeKind::NotAliveDisposed)
        } else if !status_info.disposed_flag() && status_info.unregistered_flag() && !status_info.filtered_flag() {
            Ok(ChangeKind::NotAliveUnregistered)
        } else if !status_info.disposed_flag() && !status_info.unregistered_flag() && status_info.filtered_flag() {
                Ok(ChangeKind::AliveFiltered)
        } else if !status_info.disposed_flag() && !status_info.unregistered_flag() && !status_info.filtered_flag() {
                Ok(ChangeKind::Alive)
        } else {
            Err("Combination should not occur")
        }
    }
}



#[derive(PartialEq)]
pub enum ReliabilityKind {
    BestEffort,
    Reliable,
}



pub type InstanceHandle = [u8; 16];



#[derive(PartialEq, Debug, Clone, Copy, Hash, Eq)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
}

impl RtpsSerialize for ProtocolVersion {
    fn serialize(&self, writer: &mut impl std::io::Write, _endianness: Endianness) -> RtpsSerdesResult<()> {
        writer.write(&[self.major])?;
        writer.write(&[self.minor])?;
        Ok(())
    }
}

impl RtpsDeserialize for ProtocolVersion {
    fn deserialize(bytes: &[u8], _endianness: Endianness) -> RtpsSerdesResult<Self> {
        let major = bytes[0];
        let minor = bytes[1];
        Ok(ProtocolVersion{major, minor})
    }
}



#[derive(Debug, PartialEq, Copy, Clone)]
pub struct VendorId(pub [u8; 2]);

impl RtpsSerialize for VendorId {
    fn serialize(&self, writer: &mut impl std::io::Write, _endianness: Endianness) -> RtpsSerdesResult<()> {
        writer.write(&self.0)?;
        Ok(())
    }
}

impl RtpsDeserialize for VendorId {
    fn deserialize(bytes: &[u8], _endianness: Endianness) -> RtpsSerdesResult<Self> {
        Ok(VendorId(bytes[0..2].try_into()?))
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::serdes::RtpsSerdesError;

    ///////////////////////// GUID Tests ////////////////////////
    // n/a


    ///////////////////////// GuidPrefix Tests ////////////////////////
    // n/a


    ///////////////////////// Entity Id Tests ////////////////////////

    #[test]
    fn test_entity_key_serialization_deserialization_big_endian() {
        let mut vec = Vec::new();
        let test_entity_key = EntityKey([5,20,250]);

        
        const TEST_ENTITY_KEY_BIG_ENDIAN : [u8;3] = [5,20,250];
        test_entity_key.serialize(&mut vec, Endianness::BigEndian).unwrap();
        assert_eq!(vec, TEST_ENTITY_KEY_BIG_ENDIAN);
        assert_eq!(EntityKey::deserialize(&vec, Endianness::LittleEndian).unwrap(), test_entity_key);
    }

    #[test]
    fn test_entity_key_serialization_deserialization_little_endian() {
        let mut vec = Vec::new();
        let test_entity_key = EntityKey([5,20,250]);

        
        const TEST_ENTITY_KEY_BIG_ENDIAN : [u8;3] = [5,20,250];
        test_entity_key.serialize(&mut vec, Endianness::LittleEndian).unwrap();
        assert_eq!(vec, TEST_ENTITY_KEY_BIG_ENDIAN);
        assert_eq!(EntityKey::deserialize(&vec, Endianness::LittleEndian).unwrap(), test_entity_key);
    }

    #[test]
    fn test_invalid_entity_key_deserialization() {
        let too_big_vec = vec![1,2,3,4];

        let expected_error = EntityKey::deserialize(&too_big_vec, Endianness::LittleEndian);
        match expected_error {
            Err(RtpsSerdesError::WrongSize) => assert!(true),
            _ => assert!(false),
        };

        let too_small_vec = vec![1,2,3,4];

        let expected_error = EntityKey::deserialize(&too_small_vec, Endianness::LittleEndian);
        match expected_error {
            Err(RtpsSerdesError::WrongSize) => assert!(true),
            _ => assert!(false),
        };
    }




    #[test]
    fn test_entity_kind_serialization_deserialization_big_endian() {
        let mut vec = Vec::new();
        let test_entity_kind = EntityKind::BuiltInWriterWithKey;

        
        const TEST_ENTITY_KIND_BIG_ENDIAN : [u8;1] = [0xc2];
        test_entity_kind.serialize(&mut vec, Endianness::BigEndian).unwrap();
        assert_eq!(vec, TEST_ENTITY_KIND_BIG_ENDIAN);
        assert_eq!(EntityKind::deserialize(&vec, Endianness::BigEndian).unwrap(), test_entity_kind);
    }

    #[test]
    fn test_entity_kind_serialization_deserialization_little_endian() {
        let mut vec = Vec::new();
        let test_entity_kind = EntityKind::BuiltInWriterWithKey;

        
        const TEST_ENTITY_KIND_LITTLE_ENDIAN : [u8;1] = [0xc2];
        test_entity_kind.serialize(&mut vec, Endianness::LittleEndian).unwrap();
        assert_eq!(vec, TEST_ENTITY_KIND_LITTLE_ENDIAN);
        assert_eq!(EntityKind::deserialize(&vec, Endianness::LittleEndian).unwrap(), test_entity_kind);
    }

    #[test]
    fn test_invalid_entity_kind_deserialization() {
        let too_big_vec = vec![1,2,3,4];

        let expected_error = EntityKind::deserialize(&too_big_vec, Endianness::LittleEndian);
        match expected_error {
            Err(RtpsSerdesError::WrongSize) => assert!(true),
            _ => assert!(false),
        };

        let wrong_vec = vec![0xf3];

        let expected_error = EntityKind::deserialize(&wrong_vec, Endianness::LittleEndian);
        match expected_error {
            Err(RtpsSerdesError::InvalidEnumRepresentation) => assert!(true),
            _ => assert!(false),
        };
    }
   
        
    #[test]
    fn entity_id_serialization_deserialization_big_endian() {
        let mut vec = Vec::new();
        let test_entity_id = constants::ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER;

        const TEST_ENTITY_ID_BIG_ENDIAN : [u8;4] = [0, 0x02, 0x00, 0xc4];
        test_entity_id.serialize(&mut vec, Endianness::BigEndian).unwrap();
        assert_eq!(vec, TEST_ENTITY_ID_BIG_ENDIAN);
        assert_eq!(EntityId::deserialize(&vec, Endianness::BigEndian).unwrap(), test_entity_id);
    }

    #[test]
    fn entity_id_serialization_deserialization_little_endian() {
        let mut vec = Vec::new();
        let test_entity_id = constants::ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER;

        const TEST_ENTITY_ID_LITTLE_ENDIAN : [u8;4] = [0, 0x02, 0x00, 0xc4];
        test_entity_id.serialize(&mut vec, Endianness::LittleEndian).unwrap();
        assert_eq!(vec, TEST_ENTITY_ID_LITTLE_ENDIAN);
        assert_eq!(EntityId::deserialize(&vec, Endianness::LittleEndian).unwrap(), test_entity_id);
    }

    #[test]
    fn invalid_entity_id_deserialization() {
        let too_big_vec = vec![1,2,3,4,5,5];

        let expected_error = EntityId::deserialize(&too_big_vec, Endianness::LittleEndian);
        match expected_error {
            Err(RtpsSerdesError::WrongSize) => assert!(true),
            _ => assert!(false),
        };

        let wrong_vec = vec![1,2,3,0xf3];

        let expected_error = EntityId::deserialize(&wrong_vec, Endianness::LittleEndian);
        match expected_error {
            Err(RtpsSerdesError::InvalidEnumRepresentation) => assert!(true),
            _ => assert!(false),
        };
    }



    ///////////////////////// Sequence Number Tests ////////////////////////
    
    #[test]
    fn sequence_number_serialization_deserialization_big_endian() {
        let mut vec = Vec::new();
        let test_sequence_number = SequenceNumber(1987612345679);

        
        const TEST_SEQUENCE_NUMBER_BIG_ENDIAN : [u8;8] = [0x00, 0x00, 0x01, 0xCE, 0xC6, 0xED, 0x85, 0x4F];
        test_sequence_number.serialize(&mut vec, Endianness::BigEndian).unwrap();
        assert_eq!(vec, TEST_SEQUENCE_NUMBER_BIG_ENDIAN);
        assert_eq!(SequenceNumber::deserialize(&vec, Endianness::BigEndian).unwrap(), test_sequence_number);
    }

    #[test]
    fn sequence_number_serialization_deserialization_little_endian() {
        let mut vec = Vec::new();
        let test_sequence_number = SequenceNumber(1987612345679);

        
        const TEST_SEQUENCE_NUMBER_LITTLE_ENDIAN : [u8;8] = [0xCE, 0x01, 0x00, 0x00, 0x4F, 0x85, 0xED, 0xC6];
        test_sequence_number.serialize(&mut vec, Endianness::LittleEndian).unwrap();
        assert_eq!(vec, TEST_SEQUENCE_NUMBER_LITTLE_ENDIAN);
        assert_eq!(SequenceNumber::deserialize(&vec, Endianness::LittleEndian).unwrap(), test_sequence_number);
    }

    #[test]
    fn sequence_number_serialization_deserialization_multiple_combinations() {
        let mut vec = Vec::new();
        
        {
            let test_sequence_number_i64_max = SequenceNumber(std::i64::MAX);
            test_sequence_number_i64_max.serialize(&mut vec, Endianness::LittleEndian).unwrap();
            assert_eq!(SequenceNumber::deserialize(&vec, Endianness::LittleEndian).unwrap(), test_sequence_number_i64_max);
            vec.clear();

            test_sequence_number_i64_max.serialize(&mut vec, Endianness::BigEndian).unwrap();
            assert_eq!(SequenceNumber::deserialize(&vec, Endianness::BigEndian).unwrap(), test_sequence_number_i64_max);
            vec.clear();
        }

        {
            let test_sequence_number_i64_min = SequenceNumber(std::i64::MIN);
            test_sequence_number_i64_min.serialize(&mut vec, Endianness::LittleEndian).unwrap();
            assert_eq!(SequenceNumber::deserialize(&vec, Endianness::LittleEndian).unwrap(), test_sequence_number_i64_min);
            vec.clear();

            test_sequence_number_i64_min.serialize(&mut vec, Endianness::BigEndian).unwrap();
            assert_eq!(SequenceNumber::deserialize(&vec, Endianness::BigEndian).unwrap(), test_sequence_number_i64_min);
            vec.clear();
        }

        {
            let test_sequence_number_zero = SequenceNumber(0);
            test_sequence_number_zero.serialize(&mut vec, Endianness::LittleEndian).unwrap();
            assert_eq!(SequenceNumber::deserialize(&vec, Endianness::LittleEndian).unwrap(), test_sequence_number_zero);
            vec.clear();

            test_sequence_number_zero.serialize(&mut vec, Endianness::BigEndian).unwrap();
            assert_eq!(SequenceNumber::deserialize(&vec, Endianness::BigEndian).unwrap(), test_sequence_number_zero);
            vec.clear();
        }
    }

    #[test]
    fn invalid_sequence_number_deserialization() {
        let wrong_vec = vec![1,2,3,4];

        let expected_error = SequenceNumber::deserialize(&wrong_vec, Endianness::LittleEndian);
        match expected_error {
            Err(RtpsSerdesError::WrongSize) => assert!(true),
            _ => assert!(false),
        };
    }



    ///////////////////////// Locator Tests ////////////////////////

    #[test]
    fn serialize_locator() {
        let locator = Locator::new(100, 200, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
        let expected = vec![
            100, 0, 0, 0, // kind
            200, 0, 0, 0, // port
             1,  2,  3,  4, // address
             5,  6,  7,  8, // address
             9, 10, 11, 12, // address
            13, 14, 15, 16, // address
        ];
        let mut writer = Vec::new();
        locator.serialize(&mut writer, Endianness::LittleEndian).unwrap();
        assert_eq!(expected, writer);
    }
    
    #[test]
    fn deserialize_locator() {
        let expected = Locator::new(100, 200, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
        let bytes = vec![
            100, 0, 0, 0, // kind
            200, 0, 0, 0, // port
             1,  2,  3,  4, // address
             5,  6,  7,  8, // address
             9, 10, 11, 12, // address
            13, 14, 15, 16, // address
        ];
        let result = Locator::deserialize(&bytes, Endianness::LittleEndian).unwrap();
        assert_eq!(expected, result);
    }



    ///////////////////////// TopicKind Tests ///////////////////////
    // n/a

    ///////////////////////// ChangeKind Tests //////////////////////
    // n/a

    ///////////////////////// ReliabilityKind Tests /////////////////
    // n/a

    ///////////////////////// InstanceHandle Tests //////////////////
    // n/a

    ///////////////////////// ProtocolVersion Tests /////////////////
    // n/a
    
    ///////////////////////// VendorId Tests ////////////////////////
    // n/a

}