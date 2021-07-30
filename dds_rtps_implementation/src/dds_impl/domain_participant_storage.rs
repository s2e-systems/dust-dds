use rust_rtps_pim::discovery::spdp::spdp_discovered_participant_data::SPDPdiscoveredParticipantData;

use crate::{
    rtps_impl::rtps_participant_impl::RTPSParticipantImpl, utils::shared_object::RtpsShared,
};

use super::{subscriber_storage::SubscriberStorage, writer_group_factory::WriterGroupFactory};

pub struct DomainParticipantStorage {
    rtps_participant: RTPSParticipantImpl,
    builtin_subscriber_storage: RtpsShared<SubscriberStorage>,
    user_defined_reader_groups: Vec<RtpsShared<SubscriberStorage>>,
    writer_group_factory: WriterGroupFactory,
}

impl DomainParticipantStorage {
    pub fn new(
        rtps_participant: RTPSParticipantImpl,
        builtin_reader_group: RtpsShared<SubscriberStorage>,
    ) -> Self {
        let writer_group_factory = WriterGroupFactory::new(rtps_participant.guid_prefix());
        Self {
            rtps_participant,
            builtin_subscriber_storage: builtin_reader_group,
            user_defined_reader_groups: Vec::new(),
            writer_group_factory,
        }
    }
}
