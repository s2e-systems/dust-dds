/// 
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// in the sub clauses of 9.6.2 Data representation for the built-in Endpoints
/// 

use serde::{Serialize, Deserialize};

use crate::rtps::types::{ProtocolVersion, VendorId, Locator, GUID};
use crate::rtps::serialized_payload::{Pid, ParameterId };
use crate::rtps::messages::types::Count;
use crate::rtps::behavior::types::Duration;

pub type DomainId = u32;

#[derive(PartialEq, Debug, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct BuiltInEndpointSet {
    value: u32,
}

impl BuiltInEndpointSet {
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

    pub const BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_WRITER: u32 = 1 << 10;
    pub const BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_READER: u32 = 1 << 11;

    /*
    Bits 12-15 have been reserved by the DDS-Xtypes 1.2 Specification
    and future revisions thereof.
    Bits 16-27 have been reserved by the DDS-Security 1.1 Specification
    and future revisions thereof.
    */

    pub const BUILTIN_ENDPOINT_TOPICS_ANNOUNCER: u32 = 1 << 28;
    pub const BUILTIN_ENDPOINT_TOPICS_DETECTOR: u32 = 1 << 29;

    pub fn new(value: u32) -> Self {
        BuiltInEndpointSet { value }
    }

    pub fn has(&self, endpoint: u32) -> bool {
        (self.value & endpoint)  == endpoint
    }
}


// ///////////////// PID_DOMAIN_ID
#[derive(Serialize, Deserialize, Debug)]
pub struct ParameterDomainId(pub DomainId);
impl Pid for ParameterDomainId {
    fn pid() -> ParameterId {
        0x000f       
    }
}

// ///////////////// PID_DOMAIN_TAG
#[derive(Serialize, Deserialize, Debug)]
pub struct ParameterDomainTag(pub String);
impl Pid for ParameterDomainTag {
    fn pid() -> ParameterId {
        0x4014       
    }
}

impl Default for ParameterDomainTag {
    fn default() -> Self {
        Self("".to_string())
    }
}

impl PartialEq<ParameterDomainTag> for String {
    fn eq(&self, other: &ParameterDomainTag) -> bool {
        self == &other.0
    }
}

// ///////////////// PID_PROTOCOL_VERSION
#[derive(Serialize, Deserialize, Debug)]
pub struct ParameterProtocolVersion(pub ProtocolVersion);
impl Pid for ParameterProtocolVersion {
    fn pid() -> ParameterId {
        0x0015       
    }
}

// ///////////////// PID_VENDORID
#[derive(Serialize, Deserialize, Debug)]
pub struct ParameterVendorId(pub VendorId);
impl Pid for ParameterVendorId {
    fn pid() -> ParameterId {
        0x0016
    }
}

// ///////////////// PID_UNICAST_LOCATOR
#[derive(Serialize, Deserialize, Debug)]
pub struct ParameterUnicastLocator(pub Locator);
impl Pid for ParameterUnicastLocator {
    fn pid() -> ParameterId {
        0x002f
    }
}

// ///////////////// PID_MULTICAST_LOCATOR
#[derive(Serialize, Deserialize, Debug)]
pub struct ParameterMulticastLocator(pub Locator);
impl Pid for ParameterMulticastLocator {
    fn pid() -> ParameterId {
        0x0030
    }
}

// ///////////////// PID_DEFAULT_UNICAST_LOCATOR
#[derive(Serialize, Deserialize, Debug)]
pub struct ParameterDefaultUnicastLocator(pub Locator);
impl Pid for ParameterDefaultUnicastLocator {
    fn pid() -> ParameterId {
        0x0031
    }
}

// ///////////////// PID_DEFAULT_MULTICAST_LOCATOR
#[derive(Serialize, Deserialize, Debug)]
pub struct ParameterDefaultMulticastLocator(pub Locator);
impl Pid for ParameterDefaultMulticastLocator {
    fn pid() -> ParameterId {
        0x0048
    }
}

// ///////////////// PID_METATRAFFIC_UNICAST_LOCATOR
#[derive(Serialize, Deserialize, Debug)]
pub struct ParameterMetatrafficUnicastLocator(pub Locator);
impl Pid for ParameterMetatrafficUnicastLocator {
    fn pid() -> ParameterId {
        0x0032
    }
}

// ///////////////// PID_METATRAFFIC_MULTICAST_LOCATOR
#[derive(Serialize, Deserialize, Debug)]
pub struct ParameterMetatrafficMulticastLocator(pub Locator);
impl Pid for ParameterMetatrafficMulticastLocator {
    fn pid() -> ParameterId {
        0x0033
    }
}

// ///////////////// PID_EXPECTS_INLINE_QOS
#[derive(Serialize, Deserialize, Debug)]
pub struct ParameterExpectsInlineQoS(pub bool);
impl Pid for ParameterExpectsInlineQoS {
    fn pid() -> ParameterId {
        0x0043
    }
}

impl Default for ParameterExpectsInlineQoS {
    fn default() -> Self {
        Self(false)
    }
}

impl PartialEq<ParameterExpectsInlineQoS> for bool {
    fn eq(&self, other: &ParameterExpectsInlineQoS) -> bool {
        self == &other.0
    }
}

// ///////////////// PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT
#[derive(Serialize, Deserialize, Debug)]
pub struct ParameterParticipantManualLivelinessCount(pub Count);
impl Pid for ParameterParticipantManualLivelinessCount {
    fn pid() -> ParameterId {
        0x0034
    }
}

// ///////////////// PID_PARTICIPANT_LEASE_DURATION
#[derive(Serialize, Deserialize, Debug)]
pub struct ParameterParticipantLeaseDuration(pub Duration);
impl Pid for ParameterParticipantLeaseDuration {
    fn pid() -> ParameterId {
        0x0002
    }
}

impl Default for ParameterParticipantLeaseDuration {
    fn default() -> Self {
        Self(Duration::from_secs(100))
    }
}

impl PartialEq<ParameterParticipantLeaseDuration> for Duration {
    fn eq(&self, other: &ParameterParticipantLeaseDuration) -> bool {
        self == &other.0
    }
}

// ///////////////// PID_PARTICIPANT_GUID
#[derive(Serialize, Deserialize, Debug)]
pub struct ParameterParticipantGuid(pub GUID);
impl Pid for ParameterParticipantGuid {
    fn pid() -> ParameterId {
        0x0050
    }
}

// ///////////////// PID_GROUP_GUID
#[derive(Serialize, Deserialize, Debug)]
pub struct ParameterGroupGuid(pub GUID);
impl Pid for ParameterGroupGuid {
    fn pid() -> ParameterId {
        0x0052
    }
}

// ///////////////// PID_BUILTIN_ENDPOINT_SET
#[derive(Serialize, Deserialize, Debug)]
pub struct ParameterBuiltInEndpointSet(pub BuiltInEndpointSet);
impl Pid for ParameterBuiltInEndpointSet {
    fn pid() -> ParameterId {
        0x0058
    }
}

// ///////////////// PID_BUILTIN_ENDPOINT_QOS
//TODO


#[cfg(test)]
mod tests {
    use super::*;

    ///////////////////////// BuiltInEndpointSet Tests ////////////////////////

    #[test]
    fn test_builtin_endpoint_set_participant_announcer() {
        assert_eq!(
            BuiltInEndpointSet::new(0).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_ANNOUNCER),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(1).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_ANNOUNCER),
            true
        );
        assert_eq!(
            BuiltInEndpointSet::new(16).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_ANNOUNCER),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(15).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_ANNOUNCER),
            true
        );
        assert_eq!(
            BuiltInEndpointSet::new(BuiltInEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_ANNOUNCER |
                                    BuiltInEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR).
                                    has(BuiltInEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_ANNOUNCER),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_set_participant_detector() {
        assert_eq!(
            BuiltInEndpointSet::new(0).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(2).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR),
            true
        );
        assert_eq!(
            BuiltInEndpointSet::new(16).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(15).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_set_publications_announcer() {
        assert_eq!(
            BuiltInEndpointSet::new(0).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(4).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER),
            true
        );
        assert_eq!(
            BuiltInEndpointSet::new(16).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(15).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_set_publications_detector() {
        assert_eq!(
            BuiltInEndpointSet::new(0).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(8).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR),
            true
        );
        assert_eq!(
            BuiltInEndpointSet::new(16).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(15).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_set_subscriptions_announcer() {
        assert_eq!(
            BuiltInEndpointSet::new(0).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(16).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER),
            true
        );
        assert_eq!(
            BuiltInEndpointSet::new(32).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(31).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_set_subscriptions_detector() {
        assert_eq!(
            BuiltInEndpointSet::new(0).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(32).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR),
            true
        );
        assert_eq!(
            BuiltInEndpointSet::new(31).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(63).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_participant_message_data_writer() {
        assert_eq!(
            BuiltInEndpointSet::new(0).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_WRITER),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(1024).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_WRITER),
            true
        );
        assert_eq!(
            BuiltInEndpointSet::new(1023).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_WRITER),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(2047).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_WRITER),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_participant_message_data_reader() {
        assert_eq!(
            BuiltInEndpointSet::new(0).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_READER),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(2048).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_READER),
            true
        );
        assert_eq!(
            BuiltInEndpointSet::new(2047).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_READER),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(4095).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_READER),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_topics_announcer() {
        assert_eq!(
            BuiltInEndpointSet::new(0).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_TOPICS_ANNOUNCER),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(268435456).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_TOPICS_ANNOUNCER),
            true
        );
        assert_eq!(
            BuiltInEndpointSet::new(268435455).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_TOPICS_ANNOUNCER),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(536870911).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_TOPICS_ANNOUNCER),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_topics_detector() {
        assert_eq!(
            BuiltInEndpointSet::new(0).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_TOPICS_DETECTOR),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(536870912).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_TOPICS_DETECTOR),
            true
        );
        assert_eq!(
            BuiltInEndpointSet::new(536870911).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_TOPICS_DETECTOR),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(1073741823).has(BuiltInEndpointSet::BUILTIN_ENDPOINT_TOPICS_DETECTOR),
            true
        );
    }
    
}