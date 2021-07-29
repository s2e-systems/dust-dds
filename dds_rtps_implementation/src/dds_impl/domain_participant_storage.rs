use crate::{rtps_impl::rtps_participant_impl::RTPSParticipantImpl, utils::shared_object::RtpsShared};

use super::subscriber_storage::SubscriberStorage;

pub struct DomainParticipantStorage{
    rtps_participant: RTPSParticipantImpl,
    builtin_reader_group: RtpsShared<SubscriberStorage>,
    user_defined_reader_groups: Vec<RtpsShared<SubscriberStorage>>,
}