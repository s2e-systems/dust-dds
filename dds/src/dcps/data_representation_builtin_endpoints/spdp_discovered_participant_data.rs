use super::parameter_id_values::{
    PID_BUILTIN_ENDPOINT_QOS, PID_BUILTIN_ENDPOINT_SET, PID_DOMAIN_ID, PID_DOMAIN_TAG,
    PID_EXPECTS_INLINE_QOS, PID_METATRAFFIC_MULTICAST_LOCATOR, PID_METATRAFFIC_UNICAST_LOCATOR,
    PID_PARTICIPANT_GUID, PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, PID_PROTOCOL_VERSION,
    PID_VENDORID,
};
use crate::{
    builtin_topics::ParticipantBuiltinTopicData,
    infrastructure::{
        domain::DomainId, instance::InstanceHandle, time::Duration, type_support::TypeSupport,
    },
    transport::types::{GuidPrefix, Locator, Long, ProtocolVersion, VendorId},
};
use alloc::{string::String, vec::Vec};

pub type Count = Long;

#[derive(PartialEq, Eq, Debug, Clone, Copy, TypeSupport)]
#[dust_dds(extensibility = "Final", nested)]
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

#[derive(PartialEq, Eq, Debug, Default, Clone, Copy, TypeSupport)]
#[dust_dds(extensibility = "Final", nested)]
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

#[derive(Debug, PartialEq, Eq, Clone, TypeSupport)]
#[dust_dds(extensibility = "Mutable")]
pub struct ParticipantProxy {
    #[dust_dds(id=PID_DOMAIN_ID)]
    pub(crate) domain_id: Option<DomainId>,
    #[dust_dds(id=PID_DOMAIN_TAG)]
    pub(crate) domain_tag: String,
    #[dust_dds(id=PID_PROTOCOL_VERSION)]
    pub(crate) protocol_version: ProtocolVersion,
    #[dust_dds(id = PID_PARTICIPANT_GUID)]
    pub(crate) guid_prefix: GuidPrefix,
    #[dust_dds(id = PID_VENDORID)]
    pub(crate) vendor_id: VendorId,
    #[dust_dds(id = PID_EXPECTS_INLINE_QOS)]
    pub(crate) expects_inline_qos: bool,
    #[dust_dds(id = PID_METATRAFFIC_UNICAST_LOCATOR)]
    pub(crate) metatraffic_unicast_locator_list: Vec<Locator>,
    #[dust_dds(id = PID_METATRAFFIC_MULTICAST_LOCATOR)]
    pub(crate) metatraffic_multicast_locator_list: Vec<Locator>,
    #[dust_dds(id = PID_METATRAFFIC_UNICAST_LOCATOR)]
    pub(crate) default_unicast_locator_list: Vec<Locator>,
    #[dust_dds(id = PID_METATRAFFIC_MULTICAST_LOCATOR)]
    pub(crate) default_multicast_locator_list: Vec<Locator>,
    #[dust_dds(id = PID_BUILTIN_ENDPOINT_SET)]
    pub(crate) available_builtin_endpoints: BuiltinEndpointSet,
    #[dust_dds(id = PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT)]
    pub(crate) manual_liveliness_count: Count,
    #[dust_dds(id = PID_BUILTIN_ENDPOINT_QOS)]
    pub(crate) builtin_endpoint_qos: BuiltinEndpointQos,
}

#[derive(Debug, PartialEq, Eq, Clone, TypeSupport)]
pub struct SpdpDiscoveredParticipantData {
    pub(crate) dds_participant_data: ParticipantBuiltinTopicData,
    pub(crate) participant_proxy: ParticipantProxy,
    pub(crate) lease_duration: Duration,
    pub(crate) discovered_participant_list: Vec<InstanceHandle>,
}
