use crate::behavior::stateless_reader::StatelessReaderListener;
use crate::behavior::{ReaderProxy, WriterProxy};
use crate::endpoint_types::BuiltInEndpointSet;
use crate::types::constants::{
    ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
    ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
    ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
};
use crate::types::{GuidPrefix, GUID};

use super::sedp::SimpleEndpointDiscoveryProtocol;
use super::spdp::SPDPdiscoveredParticipantData;

use rust_dds_interface::types::{DomainId, ChangeKind};
use rust_dds_interface::cache_change::CacheChange;

pub struct SimpleParticipantDiscoveryListener {
    domain_id: DomainId,
    domain_tag: String,
    sedp: SimpleEndpointDiscoveryProtocol,
}

impl SimpleParticipantDiscoveryListener {
    pub fn new(
        domain_id: DomainId,
        domain_tag: String,
        sedp: SimpleEndpointDiscoveryProtocol,
    ) -> Self {
        Self {
            domain_id,
            domain_tag,
            sedp,
        }
    }

    fn add_discovered_participant(&self, discovered_participant: &SPDPdiscoveredParticipantData) {
        // Implements the process described in 8.5.5.1 - Discovery of a new remote Participant

        if discovered_participant.domain_id() != self.domain_id {
            return;
        }

        if discovered_participant.domain_tag() != &self.domain_tag {
            return;
        }

        if discovered_participant
            .available_built_in_endpoints()
            .has(BuiltInEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR)
        {
            let guid = GUID::new(
                discovered_participant.guid_prefix(),
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            );
            let proxy = ReaderProxy::new(
                guid,
                discovered_participant
                    .metatraffic_unicast_locator_list()
                    .clone(),
                discovered_participant
                    .metatraffic_multicast_locator_list()
                    .clone(),
                discovered_participant.expects_inline_qos(),
                true,
            );
            self.sedp
                .sedp_builtin_publications_writer()
                .matched_reader_add(proxy);
        }

        if discovered_participant
            .available_built_in_endpoints()
            .has(BuiltInEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER)
        {
            let guid = GUID::new(
                discovered_participant.guid_prefix(),
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
            );
            let proxy = WriterProxy::new(
                guid,
                discovered_participant
                    .metatraffic_unicast_locator_list()
                    .clone(),
                discovered_participant
                    .metatraffic_multicast_locator_list()
                    .clone(),
            );
            self.sedp
                .sedp_builtin_publications_reader()
                .matched_writer_add(proxy);
        }

        if discovered_participant
            .available_built_in_endpoints()
            .has(BuiltInEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR)
        {
            let guid = GUID::new(
                discovered_participant.guid_prefix(),
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
            );
            let proxy = ReaderProxy::new(
                guid,
                discovered_participant
                    .metatraffic_unicast_locator_list()
                    .clone(),
                discovered_participant
                    .metatraffic_multicast_locator_list()
                    .clone(),
                discovered_participant.expects_inline_qos(),
                true,
            );
            self.sedp
                .sedp_builtin_subscriptions_writer()
                .matched_reader_add(proxy);
        }

        if discovered_participant
            .available_built_in_endpoints()
            .has(BuiltInEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER)
        {
            let guid = GUID::new(
                discovered_participant.guid_prefix(),
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            );
            let proxy = WriterProxy::new(
                guid,
                discovered_participant
                    .metatraffic_unicast_locator_list()
                    .clone(),
                discovered_participant
                    .metatraffic_multicast_locator_list()
                    .clone(),
            );
            self.sedp
                .sedp_builtin_subscriptions_reader()
                .matched_writer_add(proxy);
        }

        if discovered_participant
            .available_built_in_endpoints()
            .has(BuiltInEndpointSet::BUILTIN_ENDPOINT_TOPICS_DETECTOR)
        {
            let guid = GUID::new(
                discovered_participant.guid_prefix(),
                ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
            );
            let proxy = ReaderProxy::new(
                guid,
                discovered_participant
                    .metatraffic_unicast_locator_list()
                    .clone(),
                discovered_participant
                    .metatraffic_multicast_locator_list()
                    .clone(),
                discovered_participant.expects_inline_qos(),
                true,
            );
            self.sedp
                .sedp_builtin_topics_writer()
                .matched_reader_add(proxy);
        }

        if discovered_participant
            .available_built_in_endpoints()
            .has(BuiltInEndpointSet::BUILTIN_ENDPOINT_TOPICS_ANNOUNCER)
        {
            let guid = GUID::new(
                discovered_participant.guid_prefix(),
                ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
            );
            let proxy = WriterProxy::new(
                guid,
                discovered_participant
                    .metatraffic_unicast_locator_list()
                    .clone(),
                discovered_participant
                    .metatraffic_multicast_locator_list()
                    .clone(),
            );
            self.sedp
                .sedp_builtin_topics_reader()
                .matched_writer_add(proxy);
        }
    }

    fn remove_discovered_participant(&self, remote_participant_guid_prefix: GuidPrefix) {
        // Implements the process described in 8.5.5.2 Removal of a previously discovered Participant
        let guid = GUID::new(
            remote_participant_guid_prefix,
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
        );
        self.sedp
            .sedp_builtin_publications_writer()
            .matched_reader_remove(&guid);

        let guid = GUID::new(
            remote_participant_guid_prefix,
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
        );
        self.sedp
            .sedp_builtin_publications_reader()
            .matched_writer_remove(&guid);

        let guid = GUID::new(
            remote_participant_guid_prefix,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
        );
        self.sedp
            .sedp_builtin_subscriptions_writer()
            .matched_reader_remove(&guid);

        let guid = GUID::new(
            remote_participant_guid_prefix,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
        );
        self.sedp
            .sedp_builtin_subscriptions_reader()
            .matched_writer_remove(&guid);

        let guid = GUID::new(
            remote_participant_guid_prefix,
            ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
        );
        self.sedp
            .sedp_builtin_topics_writer()
            .matched_reader_remove(&guid);

        let guid = GUID::new(
            remote_participant_guid_prefix,
            ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
        );
        self.sedp
            .sedp_builtin_topics_reader()
            .matched_writer_remove(&guid);
    }
}

impl StatelessReaderListener for SimpleParticipantDiscoveryListener {
    fn on_add_change(&self, cc: &CacheChange) {
        let discovered_participant =
            SPDPdiscoveredParticipantData::from_key_data(cc.instance_handle(), cc.data_value().as_ref().unwrap(), 0);

        match cc.change_kind() {
            ChangeKind::Alive => self.add_discovered_participant(&discovered_participant),
            ChangeKind::AliveFiltered => (),
            ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered => {
                self.remove_discovered_participant(discovered_participant.guid_prefix())
            }
        }
    }
}
