use rtps_pim::{
    behavior::types::Duration,
    messages::types::Count,
    structure::types::{Guid, GuidPrefix, Locator, ProtocolVersion, VendorId, ENTITYID_UNKNOWN},
};

use crate::{
    dcps_psm::DomainId,
    implementation::{
        data_representation_builtin_endpoints::spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
        rtps_impl::{
            discovery_types::{BuiltinEndpointQos, BuiltinEndpointSet},
            rtps_stateful_reader_impl::RtpsStatefulReaderImpl,
            rtps_stateful_writer_impl::{RtpsReaderProxyImpl, RtpsStatefulWriterImpl},
            rtps_writer_proxy_impl::RtpsWriterProxyImpl,
            utils::clock::StdTimer,
        },
    },
};

use super::domain_participant_impl::{
    ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
    ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
    ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
};

pub struct ParticipantDiscovery<'a> {
    participant_data: &'a SpdpDiscoveredParticipantData,
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

impl<'a> ParticipantDiscovery<'a> {
    pub fn new(
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
        if participant_data.domain_id() == local_participant_domain_id
            && participant_data.domain_tag() == local_participant_domain_tag
        {
            Ok(Self { participant_data })
        } else {
            Err(MismatchedDomain)
        }
    }

    pub fn discovered_participant_add_publications_writer(
        &self,
        writer: &mut RtpsStatefulWriterImpl<StdTimer>,
    ) {
        if self
            .participant_data
            .available_builtin_endpoints()
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR)
        {
            let remote_reader_guid = Guid::new(
                self.participant_data.guid_prefix(),
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let expects_inline_qos = false;
            let proxy = RtpsReaderProxyImpl::new(
                remote_reader_guid,
                remote_group_entity_id,
                self.participant_data.metatraffic_unicast_locator_list(),
                self.participant_data.metatraffic_multicast_locator_list(),
                expects_inline_qos,
                true,
            );
            writer.matched_reader_add(proxy);
        }
    }

    pub fn discovered_participant_add_publications_reader(
        &self,
        reader: &mut RtpsStatefulReaderImpl,
    ) {
        if self
            .participant_data
            .available_builtin_endpoints()
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER)
        {
            let remote_writer_guid = Guid::new(
                self.participant_data.guid_prefix(),
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let data_max_size_serialized = None;

            let proxy = RtpsWriterProxyImpl::new(
                remote_writer_guid,
                self.participant_data.metatraffic_unicast_locator_list(),
                self.participant_data.metatraffic_multicast_locator_list(),
                data_max_size_serialized,
                remote_group_entity_id,
            );

            reader.matched_writer_add(proxy);
        }
    }

    pub fn discovered_participant_add_subscriptions_writer(
        &self,
        writer: &mut RtpsStatefulWriterImpl<StdTimer>,
    ) {
        if self
            .participant_data
            .available_builtin_endpoints()
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR)
        {
            let remote_reader_guid = Guid::new(
                self.participant_data.guid_prefix(),
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let expects_inline_qos = false;
            let proxy = RtpsReaderProxyImpl::new(
                remote_reader_guid,
                remote_group_entity_id,
                self.participant_data.metatraffic_unicast_locator_list(),
                self.participant_data.metatraffic_multicast_locator_list(),
                expects_inline_qos,
                true,
            );
            writer.matched_reader_add(proxy);
        }
    }

    pub fn discovered_participant_add_subscriptions_reader(
        &self,
        reader: &mut RtpsStatefulReaderImpl,
    ) {
        if self
            .participant_data
            .available_builtin_endpoints()
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER)
        {
            let remote_writer_guid = Guid::new(
                self.participant_data.guid_prefix(),
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let data_max_size_serialized = None;

            let proxy = RtpsWriterProxyImpl::new(
                remote_writer_guid,
                self.participant_data.metatraffic_unicast_locator_list(),
                self.participant_data.metatraffic_multicast_locator_list(),
                data_max_size_serialized,
                remote_group_entity_id,
            );
            reader.matched_writer_add(proxy);
        }
    }

    pub fn discovered_participant_add_topics_writer(
        &self,
        writer: &mut RtpsStatefulWriterImpl<StdTimer>,
    ) {
        if self
            .participant_data
            .available_builtin_endpoints()
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_TOPICS_DETECTOR)
        {
            let remote_reader_guid = Guid::new(
                self.participant_data.guid_prefix(),
                ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let expects_inline_qos = false;
            let proxy = RtpsReaderProxyImpl::new(
                remote_reader_guid,
                remote_group_entity_id,
                self.participant_data.metatraffic_unicast_locator_list(),
                self.participant_data.metatraffic_multicast_locator_list(),
                expects_inline_qos,
                true,
            );
            writer.matched_reader_add(proxy);
        }
    }

    pub fn discovered_participant_add_topics_reader(&self, reader: &mut RtpsStatefulReaderImpl) {
        if self
            .participant_data
            .available_builtin_endpoints()
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_TOPICS_ANNOUNCER)
        {
            let remote_writer_guid = Guid::new(
                self.participant_data.guid_prefix(),
                ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let data_max_size_serialized = None;

            let proxy = RtpsWriterProxyImpl::new(
                remote_writer_guid,
                self.participant_data.metatraffic_unicast_locator_list(),
                self.participant_data.metatraffic_multicast_locator_list(),
                data_max_size_serialized,
                remote_group_entity_id,
            );
            reader.matched_writer_add(proxy);
        }
    }
}

impl<'a> ParticipantDiscovery<'a> {
    pub fn domain_id(&self) -> DomainId {
        self.participant_data.domain_id() as i32
    }

    pub fn domain_tag(&self) -> &str {
        self.participant_data.domain_tag()
    }

    pub fn protocol_version(&self) -> ProtocolVersion {
        self.participant_data.protocol_version()
    }

    pub fn guid_prefix(&self) -> GuidPrefix {
        self.participant_data.guid_prefix()
    }

    pub fn vendor_id(&self) -> VendorId {
        self.participant_data.vendor_id()
    }

    pub fn expects_inline_qos(&self) -> bool {
        self.participant_data.expects_inline_qos()
    }

    pub fn metatraffic_unicast_locator_list(&self) -> &[Locator] {
        self.participant_data.metatraffic_unicast_locator_list()
    }

    pub fn metatraffic_multicast_locator_list(&self) -> &[Locator] {
        self.participant_data.metatraffic_multicast_locator_list()
    }

    pub fn default_unicast_locator_list(&self) -> &[Locator] {
        self.participant_data.default_unicast_locator_list()
    }

    pub fn default_multicast_locator_list(&self) -> &[Locator] {
        self.participant_data.default_multicast_locator_list()
    }

    pub fn available_builtin_endpoints(&self) -> BuiltinEndpointSet {
        self.participant_data.available_builtin_endpoints()
    }

    pub fn lease_duration(&self) -> Duration {
        self.participant_data.lease_duration()
    }

    pub fn manual_liveliness_count(&self) -> Count {
        self.participant_data.manual_liveliness_count()
    }

    pub fn builtin_endpoint_qos(&self) -> BuiltinEndpointQos {
        self.participant_data.builtin_endpoint_qos()
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
