use crate::{
    behavior::{
        reader::{
            stateful_reader::{RtpsStatefulReader, RtpsStatefulReaderOperations},
            writer_proxy::RtpsWriterProxyOperations,
        },
        writer::{
            reader_proxy::RtpsReaderProxyOperations,
            stateful_writer::{RtpsStatefulWriter, RtpsStatefulWriterOperations},
        },
    },
    discovery::sedp::builtin_endpoints::ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
    structure::types::{Guid, Locator, ENTITYID_UNKNOWN},
};

use super::{
    sedp::{
        builtin_endpoints::{
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
        },
        sedp_participant::SedpParticipant,
    },
    spdp::participant_proxy::ParticipantProxy,
    types::BuiltinEndpointSet,
};

pub trait ParticipantDiscovery<L> {
    fn discovered_participant_add(&mut self, participant_data: &ParticipantProxy<L>);
    fn discovered_participant_remove(&mut self, a_guid: &Guid);
}

impl<Participant, L> ParticipantDiscovery<L> for Participant
where
    for<'a> &'a L: IntoIterator<Item = &'a Locator>,
    Participant: SedpParticipant,
    Participant::BuiltinPublicationsWriter: RtpsStatefulWriterOperations + RtpsStatefulWriter,
    <Participant::BuiltinPublicationsWriter as RtpsStatefulWriter>::ReaderProxyType:
        RtpsReaderProxyOperations,
    Participant::BuiltinPublicationsReader: RtpsStatefulReaderOperations + RtpsStatefulReader,
    <Participant::BuiltinPublicationsReader as RtpsStatefulReader>::WriterProxyType:
        RtpsWriterProxyOperations,
    Participant::BuiltinSubscriptionsWriter: RtpsStatefulWriterOperations + RtpsStatefulWriter,
    <Participant::BuiltinSubscriptionsWriter as RtpsStatefulWriter>::ReaderProxyType:
        RtpsReaderProxyOperations,
    Participant::BuiltinSubscriptionsReader: RtpsStatefulReaderOperations + RtpsStatefulReader,
    <Participant::BuiltinSubscriptionsReader as RtpsStatefulReader>::WriterProxyType:
        RtpsWriterProxyOperations,
    Participant::BuiltinTopicsWriter: RtpsStatefulWriterOperations + RtpsStatefulWriter,
    <Participant::BuiltinTopicsWriter as RtpsStatefulWriter>::ReaderProxyType:
        RtpsReaderProxyOperations,
    Participant::BuiltinTopicsReader: RtpsStatefulReaderOperations + RtpsStatefulReader,
    <Participant::BuiltinTopicsReader as RtpsStatefulReader>::WriterProxyType:
        RtpsWriterProxyOperations,
{
    fn discovered_participant_add(&mut self, participant_data: &ParticipantProxy<L>) {
        // Check that the domainId of the discovered participant equals the local one.
        // If it is not equal then there the local endpoints are not configured to
        // communicate with the discovered participant.
        if participant_data.domain_id != self.domain_id() {
            return;
        }

        // Check that the domainTag of the discovered participant equals the local one.
        // If it is not equal then there the local endpoints are not configured to
        // communicate with the discovered participant.
        if participant_data.domain_tag != self.domain_tag() {
            return;
        }

        if participant_data
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR)
        {
            if let Some(sedp_builtin_publications_writer) = self.sedp_builtin_publications_writer()
            {
                let guid = Guid::new(
                    participant_data.guid_prefix,
                    ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
                );
                let remote_group_entity_id = ENTITYID_UNKNOWN;
                let expects_inline_qos = false;
                let is_active = true;
                let proxy = RtpsReaderProxyOperations::new(
                    guid,
                    remote_group_entity_id,
                    &participant_data.metatraffic_unicast_locator_list,
                    &participant_data.metatraffic_multicast_locator_list,
                    expects_inline_qos,
                    is_active,
                );
                sedp_builtin_publications_writer.matched_reader_add(proxy);
            }
        }

        if participant_data
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER)
        {
            if let Some(sedp_builtin_publications_reader) = self.sedp_builtin_publications_reader()
            {
                let guid = Guid::new(
                    participant_data.guid_prefix,
                    ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
                );
                let remote_group_entity_id = ENTITYID_UNKNOWN;
                let data_max_size_serialized = None;

                let proxy = RtpsWriterProxyOperations::new(
                    guid,
                    remote_group_entity_id,
                    &participant_data.metatraffic_unicast_locator_list,
                    &participant_data.metatraffic_multicast_locator_list,
                    data_max_size_serialized,
                );
                sedp_builtin_publications_reader.matched_writer_add(proxy);
            }
        }

        if participant_data
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR)
        {
            if let Some(sedp_builtin_subscriptions_writer) =
                self.sedp_builtin_subscriptions_writer()
            {
                let guid = Guid::new(
                    participant_data.guid_prefix,
                    ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
                );
                let remote_group_entity_id = ENTITYID_UNKNOWN;
                let expects_inline_qos = false;
                let is_active = true;
                let proxy = RtpsReaderProxyOperations::new(
                    guid,
                    remote_group_entity_id,
                    &participant_data.metatraffic_unicast_locator_list,
                    &participant_data.metatraffic_multicast_locator_list,
                    expects_inline_qos,
                    is_active,
                );
                sedp_builtin_subscriptions_writer.matched_reader_add(proxy);
            }
        }

        if participant_data
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER)
        {
            if let Some(sedp_builtin_subscriptions_reader) =
                self.sedp_builtin_subscriptions_reader()
            {
                let guid = Guid::new(
                    participant_data.guid_prefix,
                    ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
                );
                let remote_group_entity_id = ENTITYID_UNKNOWN;
                let data_max_size_serialized = None;

                let proxy = RtpsWriterProxyOperations::new(
                    guid,
                    remote_group_entity_id,
                    &participant_data.metatraffic_unicast_locator_list,
                    &participant_data.metatraffic_multicast_locator_list,
                    data_max_size_serialized,
                );
                sedp_builtin_subscriptions_reader.matched_writer_add(proxy);
            }
        }

        if participant_data
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_TOPICS_DETECTOR)
        {
            if let Some(sedp_builtin_topics_writer) = self.sedp_builtin_topics_writer() {
                let guid = Guid::new(
                    participant_data.guid_prefix,
                    ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
                );
                let remote_group_entity_id = ENTITYID_UNKNOWN;
                let expects_inline_qos = false;
                let is_active = true;
                let proxy = RtpsReaderProxyOperations::new(
                    guid,
                    remote_group_entity_id,
                    &participant_data.metatraffic_unicast_locator_list,
                    &participant_data.metatraffic_multicast_locator_list,
                    expects_inline_qos,
                    is_active,
                );
                sedp_builtin_topics_writer.matched_reader_add(proxy);
            }
        }

        if participant_data
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_TOPICS_ANNOUNCER)
        {
            if let Some(sedp_builtin_topics_reader) = self.sedp_builtin_topics_reader() {
                let guid = Guid::new(
                    participant_data.guid_prefix,
                    ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
                );
                let remote_group_entity_id = ENTITYID_UNKNOWN;
                let data_max_size_serialized = None;

                let proxy = RtpsWriterProxyOperations::new(
                    guid,
                    remote_group_entity_id,
                    &participant_data.metatraffic_unicast_locator_list,
                    &participant_data.metatraffic_multicast_locator_list,
                    data_max_size_serialized,
                );
                sedp_builtin_topics_reader.matched_writer_add(proxy);
            }
        }
    }

    fn discovered_participant_remove(&mut self, a_guid: &Guid) {
        if let Some(sedp_builtin_publications_writer) = self.sedp_builtin_publications_writer() {
            let guid = Guid::new(
                *a_guid.prefix(),
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            );
            sedp_builtin_publications_writer.matched_reader_remove(&guid);
        }

        if let Some(sedp_builtin_publications_reader) = self.sedp_builtin_publications_reader() {
            let guid = Guid::new(
                *a_guid.prefix(),
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
            );
            sedp_builtin_publications_reader.matched_writer_remove(&guid)
        }

        if let Some(sedp_builtin_subscriptions_writer) = self.sedp_builtin_subscriptions_writer() {
            let guid = Guid::new(
                *a_guid.prefix(),
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
            );
            sedp_builtin_subscriptions_writer.matched_reader_remove(&guid);
        }

        if let Some(sedp_builtin_subscriptions_reader) = self.sedp_builtin_subscriptions_reader() {
            let guid = Guid::new(
                *a_guid.prefix(),
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            );
            sedp_builtin_subscriptions_reader.matched_writer_remove(&guid)
        }

        if let Some(sedp_builtin_topics_writer) = self.sedp_builtin_topics_writer() {
            let guid = Guid::new(*a_guid.prefix(), ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR);
            sedp_builtin_topics_writer.matched_reader_remove(&guid);
        }

        if let Some(sedp_builtin_topics_reader) = self.sedp_builtin_topics_reader() {
            let guid = Guid::new(*a_guid.prefix(), ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER);
            sedp_builtin_topics_reader.matched_writer_remove(&guid)
        }
    }
}
