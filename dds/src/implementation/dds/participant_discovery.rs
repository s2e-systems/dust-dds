use crate::{
    domain::domain_participant_factory::DomainId,
    implementation::{
        data_representation_builtin_endpoints::spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
        rtps::{
            discovery_types::BuiltinEndpointSet,
            stateful_reader::RtpsStatefulReader,
            types::{Guid, ENTITYID_UNKNOWN},
            writer_proxy::RtpsWriterProxy,
        },
    },
};

use super::dds_domain_participant::{
    ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
    ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
};

pub struct _ParticipantDiscovery<'a> {
    _participant_data: &'a SpdpDiscoveredParticipantData,
}

pub struct MismatchedDomain;

impl core::fmt::Display for MismatchedDomain {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "The domain of the discovered participant doesn't match the local one"
        )
    }
}

impl<'a> _ParticipantDiscovery<'a> {
    pub fn _new(
        participant_data: &'a SpdpDiscoveredParticipantData,
        local_participant_domain_id: DomainId,
        local_participant_domain_tag: &'a str,
    ) -> core::result::Result<Self, MismatchedDomain> {
        // Check that the domainId of the discovered participant equals the local one.
        // If it is not equal then there the local endpoints are not configured to
        // communicate with the discovered participant.
        // AND
        // Check that the domainTag of the discovered participant equals the local one.
        // If it is not equal then there the local endpoints are not configured to
        // communicate with the discovered participant.
        if participant_data.participant_proxy().domain_id() == local_participant_domain_id
            && participant_data.participant_proxy().domain_tag() == local_participant_domain_tag
        {
            Ok(Self {
                _participant_data: participant_data,
            })
        } else {
            Err(MismatchedDomain)
        }
    }

    pub fn _discovered_participant_add_publications_reader(&self, reader: &mut RtpsStatefulReader) {
        if self
            ._participant_data
            .participant_proxy()
            .available_builtin_endpoints()
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER)
        {
            let remote_writer_guid = Guid::new(
                self._participant_data.participant_proxy().guid_prefix(),
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let data_max_size_serialized = None;

            let proxy = RtpsWriterProxy::new(
                remote_writer_guid,
                self._participant_data
                    .participant_proxy()
                    .metatraffic_unicast_locator_list(),
                self._participant_data
                    .participant_proxy()
                    .metatraffic_multicast_locator_list(),
                data_max_size_serialized,
                remote_group_entity_id,
            );

            reader.matched_writer_add(proxy);
        }
    }

    pub fn _discovered_participant_add_subscriptions_reader(
        &self,
        reader: &mut RtpsStatefulReader,
    ) {
        if self
            ._participant_data
            .participant_proxy()
            .available_builtin_endpoints()
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER)
        {
            let remote_writer_guid = Guid::new(
                self._participant_data.participant_proxy().guid_prefix(),
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let data_max_size_serialized = None;

            let proxy = RtpsWriterProxy::new(
                remote_writer_guid,
                self._participant_data
                    .participant_proxy()
                    .metatraffic_unicast_locator_list(),
                self._participant_data
                    .participant_proxy()
                    .metatraffic_multicast_locator_list(),
                data_max_size_serialized,
                remote_group_entity_id,
            );
            reader.matched_writer_add(proxy);
        }
    }

    pub fn _discovered_participant_add_topics_reader(&self, reader: &mut RtpsStatefulReader) {
        if self
            ._participant_data
            .participant_proxy()
            .available_builtin_endpoints()
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_TOPICS_ANNOUNCER)
        {
            let remote_writer_guid = Guid::new(
                self._participant_data.participant_proxy().guid_prefix(),
                ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let data_max_size_serialized = None;

            let proxy = RtpsWriterProxy::new(
                remote_writer_guid,
                self._participant_data
                    .participant_proxy()
                    .metatraffic_unicast_locator_list(),
                self._participant_data
                    .participant_proxy()
                    .metatraffic_multicast_locator_list(),
                data_max_size_serialized,
                remote_group_entity_id,
            );
            reader.matched_writer_add(proxy);
        }
    }
}

// fn discovered_participant_remove(&mut self, a_guid: &Guid) {
//     if let Some(sedp_builtin_publications_writer) = self.sedp_builtin_publications_writer() {
//         let guid = Guid::new(
//             *a_guid.prefix(),
//             ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
//         );
//         sedp_builtin_publications_writer.matched_reader_remove(&guid);
//     }

//     if let Some(sedp_builtin_publications_reader) = self.sedp_builtin_publications_reader() {
//         let guid = Guid::new(
//             *a_guid.prefix(),
//             ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
//         );
//         sedp_builtin_publications_reader.matched_writer_remove(&guid)
//     }

//     if let Some(sedp_builtin_subscriptions_writer) = self.sedp_builtin_subscriptions_writer() {
//         let guid = Guid::new(
//             *a_guid.prefix(),
//             ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
//         );
//         sedp_builtin_subscriptions_writer.matched_reader_remove(&guid);
//     }

//     if let Some(sedp_builtin_subscriptions_reader) = self.sedp_builtin_subscriptions_reader() {
//         let guid = Guid::new(
//             *a_guid.prefix(),
//             ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
//         );
//         sedp_builtin_subscriptions_reader.matched_writer_remove(&guid)
//     }

//     if let Some(sedp_builtin_topics_writer) = self.sedp_builtin_topics_writer() {
//         let guid = Guid::new(*a_guid.prefix(), ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR);
//         sedp_builtin_topics_writer.matched_reader_remove(&guid);
//     }

//     if let Some(sedp_builtin_topics_reader) = self.sedp_builtin_topics_reader() {
//         let guid = Guid::new(*a_guid.prefix(), ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER);
//         sedp_builtin_topics_reader.matched_writer_remove(&guid)
//     }
// }
// // }
