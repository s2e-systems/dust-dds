/// 
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// Table 8.2 - Types of the attributes that appear in the RTPS Entities and Classes
///  

use std::convert::{TryFrom, };
use crate::primitive_types::{Long, ULong, };
use crate::inline_qos_types::StatusInfo;


pub mod constants {
    use super::{VendorId, EntityId, ProtocolVersion, EntityKind, SequenceNumber};

    pub const VENDOR_ID: VendorId = [99,99];

    pub const SEQUENCE_NUMBER_UNKNOWN : SequenceNumber = std::i64::MIN;

    pub const PROTOCOL_VERSION_2_1 : ProtocolVersion = ProtocolVersion{major: 2, minor: 1};
    pub const PROTOCOL_VERSION_2_2 : ProtocolVersion = ProtocolVersion{major: 2, minor: 2};
    pub const PROTOCOL_VERSION_2_4 : ProtocolVersion = ProtocolVersion{major: 2, minor: 4};

    pub const USER_DEFINED_UNKNOWN: EntityKind = 0x00;
    pub const USER_DEFINED_WRITER_WITH_KEY: EntityKind = 0x02;
    pub const USER_DEFINED_WRITER_NO_KEY: EntityKind = 0x03;
    pub const USER_DEFINED_READER_WITH_KEY: EntityKind = 0x04;
    pub const USER_DEFINED_READER_NO_KEY: EntityKind = 0x07;
    pub const USER_DEFINED_WRITER_GROUP: EntityKind = 0x08;
    pub const USER_DEFINED_READER_GROUP: EntityKind = 0x09;
    pub const BUILTIN_UNKNOWN: EntityKind = 0xc0;
    pub const BUILTIN_PARTICIPANT: EntityKind = 0xc1;
    pub const BUILTIN_WRITER_WITH_KEY: EntityKind = 0xc2;
    pub const BUILTIN_WRITER_NO_KEY: EntityKind = 0xc3;
    pub const BUILTIN_READER_WITH_KEY: EntityKind = 0xc4;
    pub const BUILTIN_READER_NO_KEY: EntityKind = 0xc7;
    pub const BUILTIN_WRITER_GROUP: EntityKind = 0xc8;
    pub const BUILTIN_READER_GROUP: EntityKind = 0xc9;

    pub const ENTITYID_UNKNOWN: EntityId = EntityId {
        entity_key: [0, 0, 0x00],
        entity_kind: USER_DEFINED_UNKNOWN,
    };

    pub const ENTITYID_PARTICIPANT: EntityId = EntityId {
        entity_key: [0, 0, 0x01],
        entity_kind: BUILTIN_PARTICIPANT,
    };

    pub const ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER: EntityId = EntityId {
        entity_key: [0, 0, 0x02],
        entity_kind: BUILTIN_WRITER_WITH_KEY,
    };

    pub const ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR: EntityId = EntityId {
        entity_key: [0, 0, 0x02],
        entity_kind: BUILTIN_READER_WITH_KEY,
    };

    pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER: EntityId = EntityId {
        entity_key: [0, 0, 0x03],
        entity_kind: BUILTIN_WRITER_WITH_KEY,
    };

    pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR: EntityId = EntityId {
        entity_key: [0, 0, 0x03],
        entity_kind: BUILTIN_READER_WITH_KEY,
    };

    pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER: EntityId = EntityId {
        entity_key: [0, 0, 0x04],
        entity_kind: BUILTIN_WRITER_WITH_KEY,
    };

    pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR: EntityId = EntityId {
        entity_key: [0, 0, 0x04],
        entity_kind: BUILTIN_READER_WITH_KEY,
    };

    pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER: EntityId = EntityId {
        entity_key: [0, 0x01, 0x00],
        entity_kind: BUILTIN_WRITER_WITH_KEY,
    };

    pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR: EntityId = EntityId {
        entity_key: [0, 0x01, 0x00],
        entity_kind: BUILTIN_READER_WITH_KEY,
    };

    pub const ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER: EntityId = EntityId {
        entity_key: [0, 0x02, 0x00],
        entity_kind: BUILTIN_WRITER_WITH_KEY,
    };

    pub const ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER: EntityId = EntityId {
        entity_key: [0, 0x02, 0x00],
        entity_kind: BUILTIN_READER_WITH_KEY,
    };
}


// ////////// GUID_t ////////////////////////////
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

// ////////// GuidPrefix_t //////////////////////
pub type GuidPrefix = [u8; 12];

// ////////// EntityId_t ////////////////////////
pub type EntityKey = [u8; 3];
pub type EntityKind = u8;
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
    pub fn entity_key(&self) -> EntityKey{
        self.entity_key
    }
    pub fn entity_kind(&self) -> EntityKind{
        self.entity_kind
    }
}


// ////////// SequenceNumber_t //////////////////
pub type SequenceNumber = i64;

// ////////// Locator_t /////////////////////////
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

// ////////// TopicKind_t ///////////////////////
pub enum TopicKind {
    NoKey,
    WithKey,
}

// ////////// ChangeKind_t //////////////////////
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
// ////////// ReliabilityKind_t /////////////////
#[derive(PartialEq)]
pub enum ReliabilityKind {
    BestEffort,
    Reliable,
}

// ////////// InstanceHandle_t //////////////////
pub type InstanceHandle = [u8; 16];

// ////////// ProtocolVersion_t /////////////////
#[derive(PartialEq, Debug, Clone, Copy, Hash, Eq)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
}

// ////////// VendorId_t ////////////////////////
pub type VendorId = [u8; 2];



#[cfg(test)]
mod tests {
    use super::*;
    use crate::serdes::RtpsSerdesError;

    ///////////////////////// GUID Tests ////////////////////////
    // n/a


    ///////////////////////// GuidPrefix Tests ////////////////////////
    // #[test]
    // fn invalid_guid_prefix_deserialization() {
    //     let too_big_vec = [1,2,3,4,5,6,7,8,9,10,11,12,13,14];

    //     let expected_error = GuidPrefix::deserialize(&too_big_vec, Endianness::LittleEndian);
    //     match expected_error {
    //         Err(RtpsSerdesError::WrongSize) => assert!(true),
    //         _ => assert!(false),
    //     };

    //     let too_small_vec = [1,2];

    //     let expected_error = GuidPrefix::deserialize(&too_small_vec, Endianness::BigEndian);
    //     match expected_error {
    //         Err(RtpsSerdesError::WrongSize) => assert!(true),
    //         _ => assert!(false),
    //     };
    // }


    // ///////////////////////// Entity Id Tests ////////////////////////

    // #[test]
    // fn entity_key_serialization_deserialization_big_endian() {
    //     let mut vec = Vec::new();
    //     let test_entity_key = EntityKey([5,20,250]);

        
    //     const TEST_ENTITY_KEY_BIG_ENDIAN : [u8;3] = [5,20,250];
    //     test_entity_key.serialize(&mut vec, Endianness::BigEndian).unwrap();
    //     assert_eq!(vec, TEST_ENTITY_KEY_BIG_ENDIAN);
    //     assert_eq!(EntityKey::deserialize(&vec, Endianness::LittleEndian).unwrap(), test_entity_key);
    // }

    // #[test]
    // fn entity_key_serialization_deserialization_little_endian() {
    //     let mut vec = Vec::new();
    //     let test_entity_key = EntityKey([5,20,250]);

        
    //     const TEST_ENTITY_KEY_BIG_ENDIAN : [u8;3] = [5,20,250];
    //     test_entity_key.serialize(&mut vec, Endianness::LittleEndian).unwrap();
    //     assert_eq!(vec, TEST_ENTITY_KEY_BIG_ENDIAN);
    //     assert_eq!(EntityKey::deserialize(&vec, Endianness::LittleEndian).unwrap(), test_entity_key);
    // }

    // #[test]
    // fn invalid_entity_key_deserialization() {
    //     let too_big_vec = [1,2,3,4];

    //     let expected_error = EntityKey::deserialize(&too_big_vec, Endianness::LittleEndian);
    //     match expected_error {
    //         Err(RtpsSerdesError::WrongSize) => assert!(true),
    //         _ => assert!(false),
    //     };

    //     let too_small_vec = [1,2,3,4];

    //     let expected_error = EntityKey::deserialize(&too_small_vec, Endianness::LittleEndian);
    //     match expected_error {
    //         Err(RtpsSerdesError::WrongSize) => assert!(true),
    //         _ => assert!(false),
    //     };
    // }




    // #[test]
    // fn entity_kind_serialization_deserialization_big_endian() {
    //     let mut vec = Vec::new();
    //     let test_entity_kind = EntityKind::BuiltIn_Writer_With_Key;

        
    //     const TEST_ENTITY_KIND_BIG_ENDIAN : [u8;1] = [0xc2];
    //     test_entity_kind.serialize(&mut vec, Endianness::BigEndian).unwrap();
    //     assert_eq!(vec, TEST_ENTITY_KIND_BIG_ENDIAN);
    //     assert_eq!(EntityKind::deserialize(&vec, Endianness::BigEndian).unwrap(), test_entity_kind);
    // }

    // #[test]
    // fn entity_kind_serialization_deserialization_little_endian() {
    //     let mut vec = Vec::new();
    //     let test_entity_kind = EntityKind::BuiltIn_Writer_With_Key;

        
    //     const TEST_ENTITY_KIND_LITTLE_ENDIAN : [u8;1] = [0xc2];
    //     test_entity_kind.serialize(&mut vec, Endianness::LittleEndian).unwrap();
    //     assert_eq!(vec, TEST_ENTITY_KIND_LITTLE_ENDIAN);
    //     assert_eq!(EntityKind::deserialize(&vec, Endianness::LittleEndian).unwrap(), test_entity_kind);
    // }

    // #[test]
    // fn invalid_entity_kind_deserialization() {
    //     let too_big_vec = [1,2,3,4];

    //     let expected_error = EntityKind::deserialize(&too_big_vec, Endianness::LittleEndian);
    //     match expected_error {
    //         Err(RtpsSerdesError::WrongSize) => assert!(true),
    //         _ => assert!(false),
    //     };

    //     let wrong_vec = [0xf3];

    //     let expected_error = EntityKind::deserialize(&wrong_vec, Endianness::LittleEndian);
    //     match expected_error {
    //         Err(RtpsSerdesError::InvalidEnumRepresentation) => assert!(true),
    //         _ => assert!(false),
    //     };
    // }
   
        
    // #[test]
    // fn entity_id_serialization_deserialization_big_endian() {
    //     let mut vec = Vec::new();
    //     let test_entity_id = constants::ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER;

    //     const TEST_ENTITY_ID_BIG_ENDIAN : [u8;4] = [0, 0x02, 0x00, 0xc4];
    //     test_entity_id.serialize(&mut vec, Endianness::BigEndian).unwrap();
    //     assert_eq!(vec, TEST_ENTITY_ID_BIG_ENDIAN);
    //     assert_eq!(EntityId::deserialize(&vec, Endianness::BigEndian).unwrap(), test_entity_id);
    // }

    // #[test]
    // fn entity_id_serialization_deserialization_little_endian() {
    //     let mut vec = Vec::new();
    //     let test_entity_id = constants::ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER;

    //     const TEST_ENTITY_ID_LITTLE_ENDIAN : [u8;4] = [0, 0x02, 0x00, 0xc4];
    //     test_entity_id.serialize(&mut vec, Endianness::LittleEndian).unwrap();
    //     assert_eq!(vec, TEST_ENTITY_ID_LITTLE_ENDIAN);
    //     assert_eq!(EntityId::deserialize(&vec, Endianness::LittleEndian).unwrap(), test_entity_id);
    // }

    // #[test]
    // fn invalid_entity_id_deserialization() {
    //     let too_big_vec = [1,2,3,4,5,5];

    //     let expected_error = EntityId::deserialize(&too_big_vec, Endianness::LittleEndian);
    //     match expected_error {
    //         Err(RtpsSerdesError::WrongSize) => assert!(true),
    //         _ => assert!(false),
    //     };

    //     let wrong_vec = [1,2,3,0xf3];

    //     let expected_error = EntityId::deserialize(&wrong_vec, Endianness::LittleEndian);
    //     match expected_error {
    //         Err(RtpsSerdesError::InvalidEnumRepresentation) => assert!(true),
    //         _ => assert!(false),
    //     };
    // }





    // ///////////////////////// Locator Tests ////////////////////////

    // #[test]
    // fn serialize_locator() {
    //     let locator = Locator::new(100, 200, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    //     let expected = vec![
    //         100, 0, 0, 0, // kind
    //         200, 0, 0, 0, // port
    //          1,  2,  3,  4, // address
    //          5,  6,  7,  8, // address
    //          9, 10, 11, 12, // address
    //         13, 14, 15, 16, // address
    //     ];
    //     let mut writer = Vec::new();
    //     locator.serialize(&mut writer, Endianness::LittleEndian).unwrap();
    //     assert_eq!(expected, writer);
    // }
    
    // #[test]
    // fn deserialize_locator() {
    //     let expected = Locator::new(100, 200, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    //     let bytes = vec![
    //         100, 0, 0, 0, // kind
    //         200, 0, 0, 0, // port
    //          1,  2,  3,  4, // address
    //          5,  6,  7,  8, // address
    //          9, 10, 11, 12, // address
    //         13, 14, 15, 16, // address
    //     ];
    //     let result = Locator::deserialize(&bytes, Endianness::LittleEndian).unwrap();
    //     assert_eq!(expected, result);
    // }

    // #[test]
    // fn invalid_locator_deserialization() {
    //     let too_big_vec = [1,2,3,4,5,6,7,8,9,10,11,12,13,14,1,2,3,4,5,6,7,8,9,10,11,12,13,14,1,2,3,4,5,6,7,8,9,10,11,12,13,14,];

    //     let expected_error = Locator::deserialize(&too_big_vec, Endianness::LittleEndian);
    //     match expected_error {
    //         Err(RtpsSerdesError::WrongSize) => assert!(true),
    //         _ => assert!(false),
    //     };

    //     let too_small_vec = [1,2];

    //     let expected_error = Locator::deserialize(&too_small_vec, Endianness::BigEndian);
    //     match expected_error {
    //         Err(RtpsSerdesError::WrongSize) => assert!(true),
    //         _ => assert!(false),
    //     };
    // }



    // ///////////////////////// TopicKind Tests ///////////////////////
    // // n/a

    // ///////////////////////// ChangeKind Tests //////////////////////
    // // n/a

    // ///////////////////////// ReliabilityKind Tests /////////////////
    // // n/a

    // ///////////////////////// InstanceHandle Tests //////////////////
    // // n/a

    // ///////////////////////// ProtocolVersion Tests /////////////////
    // #[test]
    // fn invalid_protocol_version_deserialization() {
    //     let too_big_vec = [1,2,3,4,5,6,7,8,9,10,11,12,13,14,1,2,3,4,5,6,7,8,9,10,11,12,13,14,1,2,3,4,5,6,7,8,9,10,11,12,13,14,];

    //     let expected_error = ProtocolVersion::deserialize(&too_big_vec, Endianness::LittleEndian);
    //     match expected_error {
    //         Err(RtpsSerdesError::WrongSize) => assert!(true),
    //         _ => assert!(false),
    //     };

    //     let too_small_vec = [1];

    //     let expected_error = ProtocolVersion::deserialize(&too_small_vec, Endianness::BigEndian);
    //     match expected_error {
    //         Err(RtpsSerdesError::WrongSize) => assert!(true),
    //         _ => assert!(false),
    //     };
    // }
    
    // ///////////////////////// VendorId Tests ////////////////////////
    // #[test]
    // fn invalid_vendor_id_deserialization() {
    //     let too_big_vec = [1,2,3,4,5,6,7,8,9,10,11,12,13,14,1,2,3,4,5,6,7,8,9,10,11,12,13,14,1,2,3,4,5,6,7,8,9,10,11,12,13,14,];

    //     let expected_error = VendorId::deserialize(&too_big_vec, Endianness::LittleEndian);
    //     match expected_error {
    //         Err(RtpsSerdesError::WrongSize) => assert!(true),
    //         _ => assert!(false),
    //     };

    //     let too_small_vec = [1];

    //     let expected_error = VendorId::deserialize(&too_small_vec, Endianness::BigEndian);
    //     match expected_error {
    //         Err(RtpsSerdesError::WrongSize) => assert!(true),
    //         _ => assert!(false),
    //     };
    // }

}