pub mod submessages;
pub mod psm;
pub mod submessage_elements;
pub mod message;


// impl EntityId {
//     pub const ENTITY_KIND_USER_DEFINED_UNKNOWN: u8 = 0x00;
//     pub const ENTITY_KIND_USER_DEFINED_WRITER_WITH_KEY: u8 = 0x02;
//     pub const ENTITY_KIND_USER_DEFINED_WRITER_NO_KEY: u8 = 0x03;
//     pub const ENTITY_KIND_USER_DEFINED_READER_WITH_KEY: u8 = 0x04;
//     pub const ENTITY_KIND_USER_DEFINED_READER_NO_KEY: u8 = 0x07;
//     pub const ENTITY_KIND_USER_DEFINED_WRITER_GROUP: u8 = 0x08;
//     pub const ENTITY_KIND_USER_DEFINED_READER_GROUP: u8 = 0x09;
//     pub const ENTITY_KIND_BUILT_IN_UNKNOWN: u8 = 0xc0;
//     pub const ENTITY_KIND_BUILT_IN_PARTICIPANT: u8 = 0xc1;
//     pub const ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY: u8 = 0xc2;
//     pub const ENTITY_KIND_BUILT_IN_WRITER_NO_KEY: u8 = 0xc3;
//     pub const ENTITY_KIND_BUILT_IN_READER_WITH_KEY: u8 = 0xc4;
//     pub const ENTITY_KIND_BUILT_IN_READER_NO_KEY: u8 = 0xc7;
//     pub const ENTITY_KIND_BUILT_IN_WRITER_GROUP: u8 = 0xc8;
//     pub const ENTITY_KIND_BUILT_IN_READER_GROUP: u8 = 0xc9;

//     pub const ENTITYID_PARTICIPANT: EntityId = EntityId {
//         entity_key: [0, 0, 0x01],
//         entity_kind: 0xc1,
//     };

//     pub const ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER: EntityId = EntityId {
//         entity_key: [0, 0, 0x02],
//         entity_kind: 0xc2,
//     };
//     pub const ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR: EntityId = EntityId {
//         entity_key: [0, 0, 0x02],
//         entity_kind: 0xc7,
//     };

//     pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER: EntityId = EntityId {
//         entity_key: [0, 0, 0x03],
//         entity_kind: 0xc2,
//     };
//     pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR: EntityId = EntityId {
//         entity_key: [0, 0, 0x03],
//         entity_kind: 0xc7,
//     };

//     pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER: EntityId = EntityId {
//         entity_key: [0, 0, 0x04],
//         entity_kind: 0xc2,
//     };
//     pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR: EntityId = EntityId {
//         entity_key: [0, 0, 0x04],
//         entity_kind: 0xc7,
//     };

//     pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER: EntityId = EntityId {
//         entity_key: [0, 0x01, 0x00],
//         entity_kind: 0xc2,
//     };

//     pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR: EntityId = EntityId {
//         entity_key: [0, 0x01, 0x00],
//         entity_kind: 0xc7,
//     };

//     pub const ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER: EntityId = EntityId {
//         entity_key: [0, 0x02, 0x00],
//         entity_kind: 0xc2,
//     };
//     pub const ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER: EntityId = EntityId {
//         entity_key: [0, 0x02, 0x00],
//         entity_kind: 0xc7,
//     };
// }

// impl rust_rtps_pim::types::EntityId for EntityId {
//     const ENTITYID_UNKNOWN: Self = Self {
//         entity_key: [0; 3],
//         entity_kind: 0,
//     };
// }
