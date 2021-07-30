use rust_rtps_pim::discovery::spdp::spdp_discovered_participant_data::SPDPdiscoveredParticipantData;

use crate::{
    rtps_impl::rtps_participant_impl::RTPSParticipantImpl, utils::shared_object::RtpsShared,
};

use super::{
    publisher_storage::PublisherStorage, subscriber_storage::SubscriberStorage,
    writer_group_factory::WriterGroupFactory,
};

pub struct DomainParticipantStorage {
    rtps_participant: RTPSParticipantImpl,
    builtin_subscriber_storage: RtpsShared<SubscriberStorage>,
    builtin_publisher_storage: RtpsShared<PublisherStorage>,
    user_defined_subscriber_storage: Vec<RtpsShared<SubscriberStorage>>,
    user_defined_publisher_storage: Vec<RtpsShared<PublisherStorage>>,
    writer_group_factory: WriterGroupFactory,
}

impl DomainParticipantStorage {
    pub fn new(
        rtps_participant: RTPSParticipantImpl,
        builtin_subscriber_storage: RtpsShared<SubscriberStorage>,
        builtin_publisher_storage: RtpsShared<PublisherStorage>,
    ) -> Self {
        let writer_group_factory = WriterGroupFactory::new(rtps_participant.guid_prefix());
        Self {
            rtps_participant,
            builtin_subscriber_storage,
            builtin_publisher_storage,
            user_defined_subscriber_storage: Vec::new(),
            user_defined_publisher_storage: Vec::new(),
            writer_group_factory,
        }
    }
}
