/// 
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// in the sub clauses of 9.6.2 Data representation for the built-in Endpoints
/// 



#[derive(PartialEq, Debug, Eq, Hash)]
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






#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;


    ///////////////////////// BuiltInEndPointSet Tests ////////////////////////

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
