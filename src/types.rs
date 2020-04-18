use crate::parser::InlineQosParameter;
use serde_derive::{Deserialize, Serialize};
use std::{i32, u32};
use std::collections::BTreeMap;

#[derive(Serialize, Hash, Deserialize, Eq, PartialEq, Default, Debug, Clone, Copy)]
pub struct EntityId {
    entity_key: [u8; 3],
    entity_kind: EntityKind,
}

impl EntityId {
    pub fn new(entity_key: [u8; 3], entity_kind: u8) -> EntityId {
        EntityId {
            entity_key,
            entity_kind,
        }
    }
}

pub const ENTITY_KIND_BUILT_IN_UNKNOWN: EntityKind = 0xc0;
pub const ENTITY_KIND_BUILT_IN_PARTICIPANT: EntityKind = 0xc1;
pub const ENTITY_KIND_WRITER_WITH_KEY: EntityKind = 0x02;
pub const ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY: EntityKind = 0xc2;
pub const ENTITY_KIND_WRITER_NO_KEY: EntityKind = 0x03;
pub const ENTITY_KIND_BUILT_IN_WRITER_NO_KEY: EntityKind = 0xc3;
pub const ENTITY_KIND_READER_NO_KEY: EntityKind = 0x04;
pub const ENTITY_KIND_BUILT_IN_READER_NO_KEY: EntityKind = 0xc4;
pub const ENTITY_KIND_READER_WITH_KEY: EntityKind = 0x07;
pub const ENTITY_KIND_BUILT_IN_READER_WITH_KEY: EntityKind = 0xc7;
pub const ENTITY_KIND_WRITER_GROUP: EntityKind = 0x08;
pub const ENTITY_KIND_BUILT_IN_WRITER_GROUP: EntityKind = 0xc8;
pub const ENTITY_KIND_READER_GROUP: EntityKind = 0x09;
pub const ENTITY_KIND_BUILT_IN_READER_GROUP: EntityKind = 0xc9;

pub const ENTITYID_UNKNOWN: EntityId = EntityId {
    entity_key: [0, 0, 0x00],
    entity_kind: 0,
};

pub const ENTITYID_PARTICIPANT: EntityId = EntityId {
    entity_key: [0, 0, 0x01],
    entity_kind: ENTITY_KIND_BUILT_IN_PARTICIPANT,
};

pub const ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER: EntityId = EntityId {
    entity_key: [0, 0, 0x02],
    entity_kind: ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY,
};

pub const ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR: EntityId = EntityId {
    entity_key: [0, 0, 0x02],
    entity_kind: ENTITY_KIND_BUILT_IN_READER_WITH_KEY,
};

pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER: EntityId = EntityId {
    entity_key: [0, 0, 0x03],
    entity_kind: ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY,
};

pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR: EntityId = EntityId {
    entity_key: [0, 0, 0x03],
    entity_kind: ENTITY_KIND_BUILT_IN_READER_WITH_KEY,
};

pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER: EntityId = EntityId {
    entity_key: [0, 0, 0x04],
    entity_kind: ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY,
};

pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR: EntityId = EntityId {
    entity_key: [0, 0, 0x04],
    entity_kind: ENTITY_KIND_BUILT_IN_READER_WITH_KEY,
};

pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER: EntityId = EntityId {
    entity_key: [0, 0x01, 0x00],
    entity_kind: ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY,
};

pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR: EntityId = EntityId {
    entity_key: [0, 0x01, 0x00],
    entity_kind: ENTITY_KIND_BUILT_IN_READER_WITH_KEY,
};

pub const ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER: EntityId = EntityId {
    entity_key: [0, 0x02, 0x00],
    entity_kind: ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY,
};

pub const ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER: EntityId = EntityId {
    entity_key: [0, 0x02, 0x00],
    entity_kind: ENTITY_KIND_BUILT_IN_READER_WITH_KEY,
};

pub enum TopicKind {
    NoKey,
    WithKey,
}

pub enum ReliabilityKind {
    BestEffort,
    Reliable,
}

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub enum ChangeKind {
    Alive,
    AliveFiltered,
    NotAliveDisposed,
    NotAliveUnregistered,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct Time {
    pub seconds: u32,
    pub fraction: u32,
}

const TIME_ZERO: Time = Time {
    seconds: 0,
    fraction: 0,
};
const TIME_INFINITE: Time = Time {
    seconds: u32::MAX,
    fraction: u32::MAX - 1,
};
const TIME_INVALID: Time = Time {
    seconds: u32::MAX,
    fraction: u32::MAX,
};

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct Duration {
    pub seconds: i32,
    pub fraction: u32,
}

pub const DURATION_ZERO: Duration = Duration {
    seconds: 0,
    fraction: 0,
};
pub const DURATION_INFINITE: Duration = Duration {
    seconds: i32::MAX,
    fraction: u32::MAX,
};

pub type InlineQosParameterList = Vec<InlineQosParameter>;

pub type ParameterList = Vec<Parameter>;

#[derive(Hash, Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct Parameter {
    pub parameter_id: u16,
    pub value: Vec<u8>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy, Hash, Eq)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
}

#[derive(Serialize, Deserialize, PartialEq, Hash, Eq, Debug, Copy, Clone)]
pub struct Locator {
    pub kind: i32,
    pub port: u32,
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

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Default, Debug, Clone, Copy)]
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

#[derive(PartialEq, Deserialize, Debug, Eq, Hash)]
pub struct BuiltInEndPointSet {
    value: u32,
}

pub enum BuiltInEndPoints {
    ParticipantAnnouncer = 0,
    ParticipantDetector = 1,
    PublicationsAnnouncer = 2,
    PublicationsDetector = 3,
    SubscriptionsAnnouncer = 4,
    SubscriptionsDetector = 5,

    /*
    The following have been deprecated in version 2.4 of the
    specification. These bits should not be used by versions of the
    protocol equal to or newer than the deprecated version unless
    they are used with the same meaning as in versions prior to the
    deprecated version.
    @position(6) DISC_BUILTIN_ENDPOINT_PARTICIPANT_PROXY_ANNOUNCER,
    @position(7) DISC_BUILTIN_ENDPOINT_PARTICIPANT_PROXY_DETECTOR,
    @position(8) DISC_BUILTIN_ENDPOINT_PARTICIPANT_STATE_ANNOUNCER,
    @position(9) DISC_BUILTIN_ENDPOINT_PARTICIPANT_STATE_DETECTOR,
    */
    ParticipantMessageDataWriter = 10,
    ParticipantMessageDataReader = 11,

    /*
    Bits 12-15 have been reserved by the DDS-Xtypes 1.2 Specification
    and future revisions thereof.
    Bits 16-27 have been reserved by the DDS-Security 1.1 Specification
    and future revisions thereof.
    */
    TopicsAnnouncer = 28,
    TopicsDetector = 29,
}

impl BuiltInEndPointSet {
    pub fn new(value: u32) -> Self {
        BuiltInEndPointSet { value }
    }

    pub fn has(&self, endpoint: BuiltInEndPoints) -> bool {
        let bit_position = endpoint as u8;
        let bitmask = 1 << bit_position;
        (self.value & bitmask) >> bit_position == 1
    }
}

pub type EntityKind = u8;
pub type InstanceHandle = [u8; 16];
pub type VendorId = [u8; 2];
pub type LocatorList = Vec<Locator>;
pub type GuidPrefix = [u8; 12];
pub type Count = i32;
pub type SequenceNumber = i64;
pub type SequenceNumberSet = BTreeMap<SequenceNumber, bool>;
pub type FragmentNumber = u32;
pub type FragmentNumberSet = Vec<(FragmentNumber, bool)>;
pub type KeyHash = [u8; 16];
pub type StatusInfo = [u8; 4];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_endpoint_set_participant_announcer() {
        assert_eq!(
            BuiltInEndPointSet::new(0).has(BuiltInEndPoints::ParticipantAnnouncer),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(1).has(BuiltInEndPoints::ParticipantAnnouncer),
            true
        );
        assert_eq!(
            BuiltInEndPointSet::new(16).has(BuiltInEndPoints::ParticipantAnnouncer),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(15).has(BuiltInEndPoints::ParticipantAnnouncer),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_set_participant_detector() {
        assert_eq!(
            BuiltInEndPointSet::new(0).has(BuiltInEndPoints::ParticipantDetector),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(2).has(BuiltInEndPoints::ParticipantDetector),
            true
        );
        assert_eq!(
            BuiltInEndPointSet::new(16).has(BuiltInEndPoints::ParticipantDetector),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(15).has(BuiltInEndPoints::ParticipantDetector),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_set_publications_announcer() {
        assert_eq!(
            BuiltInEndPointSet::new(0).has(BuiltInEndPoints::PublicationsAnnouncer),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(4).has(BuiltInEndPoints::PublicationsAnnouncer),
            true
        );
        assert_eq!(
            BuiltInEndPointSet::new(16).has(BuiltInEndPoints::PublicationsAnnouncer),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(15).has(BuiltInEndPoints::PublicationsAnnouncer),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_set_publications_detector() {
        assert_eq!(
            BuiltInEndPointSet::new(0).has(BuiltInEndPoints::PublicationsDetector),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(8).has(BuiltInEndPoints::PublicationsDetector),
            true
        );
        assert_eq!(
            BuiltInEndPointSet::new(16).has(BuiltInEndPoints::PublicationsDetector),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(15).has(BuiltInEndPoints::PublicationsDetector),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_set_subscriptions_announcer() {
        assert_eq!(
            BuiltInEndPointSet::new(0).has(BuiltInEndPoints::SubscriptionsAnnouncer),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(16).has(BuiltInEndPoints::SubscriptionsAnnouncer),
            true
        );
        assert_eq!(
            BuiltInEndPointSet::new(32).has(BuiltInEndPoints::SubscriptionsAnnouncer),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(31).has(BuiltInEndPoints::SubscriptionsAnnouncer),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_set_subscriptions_detector() {
        assert_eq!(
            BuiltInEndPointSet::new(0).has(BuiltInEndPoints::SubscriptionsDetector),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(32).has(BuiltInEndPoints::SubscriptionsDetector),
            true
        );
        assert_eq!(
            BuiltInEndPointSet::new(31).has(BuiltInEndPoints::SubscriptionsDetector),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(63).has(BuiltInEndPoints::SubscriptionsDetector),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_participant_message_data_writer() {
        assert_eq!(
            BuiltInEndPointSet::new(0).has(BuiltInEndPoints::ParticipantMessageDataWriter),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(1024).has(BuiltInEndPoints::ParticipantMessageDataWriter),
            true
        );
        assert_eq!(
            BuiltInEndPointSet::new(1023).has(BuiltInEndPoints::ParticipantMessageDataWriter),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(2047).has(BuiltInEndPoints::ParticipantMessageDataWriter),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_participant_message_data_reader() {
        assert_eq!(
            BuiltInEndPointSet::new(0).has(BuiltInEndPoints::ParticipantMessageDataReader),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(2048).has(BuiltInEndPoints::ParticipantMessageDataReader),
            true
        );
        assert_eq!(
            BuiltInEndPointSet::new(2047).has(BuiltInEndPoints::ParticipantMessageDataReader),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(4095).has(BuiltInEndPoints::ParticipantMessageDataReader),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_topics_announcer() {
        assert_eq!(
            BuiltInEndPointSet::new(0).has(BuiltInEndPoints::TopicsAnnouncer),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(268435456).has(BuiltInEndPoints::TopicsAnnouncer),
            true
        );
        assert_eq!(
            BuiltInEndPointSet::new(268435455).has(BuiltInEndPoints::TopicsAnnouncer),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(536870911).has(BuiltInEndPoints::TopicsAnnouncer),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_topics_detector() {
        assert_eq!(
            BuiltInEndPointSet::new(0).has(BuiltInEndPoints::TopicsDetector),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(536870912).has(BuiltInEndPoints::TopicsDetector),
            true
        );
        assert_eq!(
            BuiltInEndPointSet::new(536870911).has(BuiltInEndPoints::TopicsDetector),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(1073741823).has(BuiltInEndPoints::TopicsDetector),
            true
        );
    }
}
