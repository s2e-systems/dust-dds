/// 
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// in the sub clauses of 9.6.2 Data representation for the built-in Endpoints
/// 

use serde::{Serialize, Deserialize};

use crate::types::{ProtocolVersion, VendorId, Locator, GUID};
use crate::messages::{Pid, };
use crate::messages::types::{ParameterId, Count,};
use crate::behavior::types::Duration;

pub type DomainId = u32;

#[derive(PartialEq, Debug, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct BuiltInEndpointSet {
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

impl BuiltInEndpointSet {
    pub fn new(value: u32) -> Self {
        BuiltInEndpointSet { value }
    }

    pub fn has(&self, endpoint: BuiltInEndPoints) -> bool {
        let bit_position = endpoint as u8;
        let bitmask = 1 << bit_position;
        (self.value & bitmask) >> bit_position == 1
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
            BuiltInEndpointSet::new(0).has(BuiltInEndPoints::ParticipantAnnouncer),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(1).has(BuiltInEndPoints::ParticipantAnnouncer),
            true
        );
        assert_eq!(
            BuiltInEndpointSet::new(16).has(BuiltInEndPoints::ParticipantAnnouncer),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(15).has(BuiltInEndPoints::ParticipantAnnouncer),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_set_participant_detector() {
        assert_eq!(
            BuiltInEndpointSet::new(0).has(BuiltInEndPoints::ParticipantDetector),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(2).has(BuiltInEndPoints::ParticipantDetector),
            true
        );
        assert_eq!(
            BuiltInEndpointSet::new(16).has(BuiltInEndPoints::ParticipantDetector),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(15).has(BuiltInEndPoints::ParticipantDetector),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_set_publications_announcer() {
        assert_eq!(
            BuiltInEndpointSet::new(0).has(BuiltInEndPoints::PublicationsAnnouncer),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(4).has(BuiltInEndPoints::PublicationsAnnouncer),
            true
        );
        assert_eq!(
            BuiltInEndpointSet::new(16).has(BuiltInEndPoints::PublicationsAnnouncer),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(15).has(BuiltInEndPoints::PublicationsAnnouncer),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_set_publications_detector() {
        assert_eq!(
            BuiltInEndpointSet::new(0).has(BuiltInEndPoints::PublicationsDetector),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(8).has(BuiltInEndPoints::PublicationsDetector),
            true
        );
        assert_eq!(
            BuiltInEndpointSet::new(16).has(BuiltInEndPoints::PublicationsDetector),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(15).has(BuiltInEndPoints::PublicationsDetector),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_set_subscriptions_announcer() {
        assert_eq!(
            BuiltInEndpointSet::new(0).has(BuiltInEndPoints::SubscriptionsAnnouncer),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(16).has(BuiltInEndPoints::SubscriptionsAnnouncer),
            true
        );
        assert_eq!(
            BuiltInEndpointSet::new(32).has(BuiltInEndPoints::SubscriptionsAnnouncer),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(31).has(BuiltInEndPoints::SubscriptionsAnnouncer),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_set_subscriptions_detector() {
        assert_eq!(
            BuiltInEndpointSet::new(0).has(BuiltInEndPoints::SubscriptionsDetector),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(32).has(BuiltInEndPoints::SubscriptionsDetector),
            true
        );
        assert_eq!(
            BuiltInEndpointSet::new(31).has(BuiltInEndPoints::SubscriptionsDetector),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(63).has(BuiltInEndPoints::SubscriptionsDetector),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_participant_message_data_writer() {
        assert_eq!(
            BuiltInEndpointSet::new(0).has(BuiltInEndPoints::ParticipantMessageDataWriter),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(1024).has(BuiltInEndPoints::ParticipantMessageDataWriter),
            true
        );
        assert_eq!(
            BuiltInEndpointSet::new(1023).has(BuiltInEndPoints::ParticipantMessageDataWriter),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(2047).has(BuiltInEndPoints::ParticipantMessageDataWriter),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_participant_message_data_reader() {
        assert_eq!(
            BuiltInEndpointSet::new(0).has(BuiltInEndPoints::ParticipantMessageDataReader),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(2048).has(BuiltInEndPoints::ParticipantMessageDataReader),
            true
        );
        assert_eq!(
            BuiltInEndpointSet::new(2047).has(BuiltInEndPoints::ParticipantMessageDataReader),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(4095).has(BuiltInEndPoints::ParticipantMessageDataReader),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_topics_announcer() {
        assert_eq!(
            BuiltInEndpointSet::new(0).has(BuiltInEndPoints::TopicsAnnouncer),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(268435456).has(BuiltInEndPoints::TopicsAnnouncer),
            true
        );
        assert_eq!(
            BuiltInEndpointSet::new(268435455).has(BuiltInEndPoints::TopicsAnnouncer),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(536870911).has(BuiltInEndPoints::TopicsAnnouncer),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_topics_detector() {
        assert_eq!(
            BuiltInEndpointSet::new(0).has(BuiltInEndPoints::TopicsDetector),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(536870912).has(BuiltInEndPoints::TopicsDetector),
            true
        );
        assert_eq!(
            BuiltInEndpointSet::new(536870911).has(BuiltInEndPoints::TopicsDetector),
            false
        );
        assert_eq!(
            BuiltInEndpointSet::new(1073741823).has(BuiltInEndPoints::TopicsDetector),
            true
        );
    }
    
}