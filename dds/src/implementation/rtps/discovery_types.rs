// This file implements the types that appear in the built-in topic messages
// using the mapping described in 9.3.2 Mapping of the Types that Appear Within Submessages or Built-in Topic Data

use crate::serialized_payload::cdr::{deserialize::CdrDeserialize, serialize::CdrSerialize};

use super::types::{EntityId, BUILT_IN_READER_WITH_KEY, BUILT_IN_WRITER_WITH_KEY};

pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER: EntityId =
    EntityId::new([0x00, 0x01, 0x00], BUILT_IN_WRITER_WITH_KEY);

pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER: EntityId =
    EntityId::new([0x00, 0x01, 0x00], BUILT_IN_READER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER: EntityId =
    EntityId::new([0, 0, 0x02], BUILT_IN_WRITER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR: EntityId =
    EntityId::new([0, 0, 0x02], BUILT_IN_READER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER: EntityId =
    EntityId::new([0, 0, 0x03], BUILT_IN_WRITER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR: EntityId =
    EntityId::new([0, 0, 0x03], BUILT_IN_READER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER: EntityId =
    EntityId::new([0, 0, 0x04], BUILT_IN_WRITER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR: EntityId =
    EntityId::new([0, 0, 0x04], BUILT_IN_READER_WITH_KEY);

#[derive(PartialEq, Eq, Debug, Clone, Copy, CdrSerialize, CdrDeserialize)]
pub struct BuiltinEndpointSet(pub u32);

impl Default for BuiltinEndpointSet {
    fn default() -> Self {
        Self(
            Self::BUILTIN_ENDPOINT_PARTICIPANT_ANNOUNCER
                | Self::BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR
                | Self::BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER
                | Self::BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR
                | Self::BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER
                | Self::BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR
                | Self::BUILTIN_ENDPOINT_TOPICS_ANNOUNCER
                | Self::BUILTIN_ENDPOINT_TOPICS_DETECTOR,
        )
    }
}

impl BuiltinEndpointSet {
    pub const BUILTIN_ENDPOINT_PARTICIPANT_ANNOUNCER: u32 = 1 << 0;
    pub const BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR: u32 = 1 << 1;
    pub const BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER: u32 = 1 << 2;
    pub const BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR: u32 = 1 << 3;
    pub const BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER: u32 = 1 << 4;
    pub const BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR: u32 = 1 << 5;

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

    pub const _BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_WRITER: u32 = 1 << 10;
    pub const _BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_READER: u32 = 1 << 11;

    /*
    Bits 12-15 have been reserved by the DDS-Xtypes 1.2 Specification
    and future revisions thereof.
    Bits 16-27 have been reserved by the DDS-Security 1.1 Specification
    and future revisions thereof.
    */

    pub const BUILTIN_ENDPOINT_TOPICS_ANNOUNCER: u32 = 1 << 28;
    pub const BUILTIN_ENDPOINT_TOPICS_DETECTOR: u32 = 1 << 29;

    #[allow(dead_code)]
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn has(&self, endpoint: u32) -> bool {
        (self.0 & endpoint) == endpoint
    }
}

#[derive(PartialEq, Eq, Debug, Default, Clone, Copy, CdrSerialize, CdrDeserialize)]
pub struct BuiltinEndpointQos(pub u32);

impl BuiltinEndpointQos {
    #[allow(dead_code)]
    pub const BEST_EFFORT_PARTICIPANT_MESSAGE_DATA_READER: u32 = 1 << 29;

    #[allow(dead_code)]
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    #[allow(dead_code)]
    pub fn has(&self, endpoint: u32) -> bool {
        (self.0 & endpoint) == endpoint
    }
}
