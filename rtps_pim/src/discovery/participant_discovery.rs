use crate::{
    behavior::{
        reader::{
            stateful_reader::{RtpsStatefulReader, RtpsStatefulReaderOperations},
            writer_proxy::{RtpsWriterProxy, RtpsWriterProxyOperations},
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
    sedp::builtin_endpoints::{
        ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
        ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
    },
    spdp::participant_proxy::ParticipantProxy,
    types::{BuiltinEndpointSet, DomainId},
};

// pub trait ParticipantDiscovery<S, L> {
//     fn discovered_participant_add(&mut self, participant_data: &ParticipantProxy<S, L>);
//     fn discovered_participant_remove(&mut self, a_guid: &Guid);
// }

// impl<Participant, S, L> ParticipantDiscovery<S, L> for Participant
// where
//     for<'a> &'a L: IntoIterator<Item = &'a Locator>,
//     S: for<'a> PartialEq<&'a str>,
//     Participant: SedpParticipant,
//     Participant::BuiltinPublicationsWriter: RtpsStatefulWriterOperations + RtpsStatefulWriter,
//     <Participant::BuiltinPublicationsWriter as RtpsStatefulWriter>::ReaderProxyType:
//         RtpsReaderProxyOperations,
//     Participant::BuiltinPublicationsReader: RtpsStatefulReaderOperations + RtpsStatefulReader,
//     <Participant::BuiltinPublicationsReader as RtpsStatefulReader>::WriterProxyType:
//         RtpsWriterProxyOperations,
//     Participant::BuiltinSubscriptionsWriter: RtpsStatefulWriterOperations + RtpsStatefulWriter,
//     <Participant::BuiltinSubscriptionsWriter as RtpsStatefulWriter>::ReaderProxyType:
//         RtpsReaderProxyOperations,
//     Participant::BuiltinSubscriptionsReader: RtpsStatefulReaderOperations + RtpsStatefulReader,
//     <Participant::BuiltinSubscriptionsReader as RtpsStatefulReader>::WriterProxyType:
//         RtpsWriterProxyOperations,
//     Participant::BuiltinTopicsWriter: RtpsStatefulWriterOperations + RtpsStatefulWriter,
//     <Participant::BuiltinTopicsWriter as RtpsStatefulWriter>::ReaderProxyType:
//         RtpsReaderProxyOperations,
//     Participant::BuiltinTopicsReader: RtpsStatefulReaderOperations + RtpsStatefulReader,
//     <Participant::BuiltinTopicsReader as RtpsStatefulReader>::WriterProxyType:
//         RtpsWriterProxyOperations,
// {
pub struct ParticipantDiscovery<'a, S, L> {
    participant_data: &'a ParticipantProxy<S, L>,
}

impl<'a, S, L> ParticipantDiscovery<'a, S, L> {
    pub fn new(
        participant_data: &'a ParticipantProxy<S, L>,
        local_participant_domain_id: DomainId,
        local_participant_domain_tag: &'a str,
    ) -> core::result::Result<Self, ()>
    where
        S: for<'b> PartialEq<&'b str>,
    {
        // Check that the domainId of the discovered participant equals the local one.
        // If it is not equal then there the local endpoints are not configured to
        // communicate with the discovered participant.
        // AND
        // Check that the domainTag of the discovered participant equals the local one.
        // If it is not equal then there the local endpoints are not configured to
        // communicate with the discovered participant.
        if participant_data.domain_id == local_participant_domain_id
            && participant_data.domain_tag == local_participant_domain_tag
        {
            Ok(Self { participant_data })
        } else {
            Err(())
        }
    }

    pub fn discovered_participant_add_publications_writer(
        &self,
        writer: &mut (impl RtpsStatefulWriterOperations
                  + RtpsStatefulWriter<ReaderProxyType = impl RtpsReaderProxyOperations>),
    ) where
        for<'b> &'b L: IntoIterator<Item = &'b Locator>,
    {
        if self
            .participant_data
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR)
        {
            let guid = Guid::new(
                self.participant_data.guid_prefix,
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let expects_inline_qos = false;
            let is_active = true;
            let proxy = RtpsReaderProxyOperations::new(
                guid,
                remote_group_entity_id,
                &self.participant_data.metatraffic_unicast_locator_list,
                &self.participant_data.metatraffic_multicast_locator_list,
                expects_inline_qos,
                is_active,
            );
            writer.matched_reader_add(proxy);
        }
    }

    pub fn discovered_participant_add_publications_reader(
        &self,
        reader: &mut (impl RtpsStatefulReaderOperations<L>
                  + RtpsStatefulReader<WriterProxyType = impl RtpsWriterProxyOperations>),
    ) where
        L: Clone,
        for<'b> &'b L: IntoIterator<Item = &'b Locator>,
    {
        if self
            .participant_data
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER)
        {
            let remote_writer_guid = Guid::new(
                self.participant_data.guid_prefix,
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let data_max_size_serialized = None;

            let proxy = RtpsWriterProxy {
                remote_writer_guid,
                unicast_locator_list: self
                    .participant_data
                    .metatraffic_unicast_locator_list
                    .clone(),
                multicast_locator_list: self
                    .participant_data
                    .metatraffic_unicast_locator_list
                    .clone(),
                data_max_size_serialized,
                remote_group_entity_id,
            };

            reader.matched_writer_add(proxy);
        }
    }

    pub fn discovered_participant_add_subscriptions_writer(
        &self,
        writer: &mut (impl RtpsStatefulWriterOperations
                  + RtpsStatefulWriter<ReaderProxyType = impl RtpsReaderProxyOperations>),
    ) where
        for<'b> &'b L: IntoIterator<Item = &'b Locator>,
    {
        if self
            .participant_data
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR)
        {
            let guid = Guid::new(
                self.participant_data.guid_prefix,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let expects_inline_qos = false;
            let is_active = true;
            let proxy = RtpsReaderProxyOperations::new(
                guid,
                remote_group_entity_id,
                &self.participant_data.metatraffic_unicast_locator_list,
                &self.participant_data.metatraffic_multicast_locator_list,
                expects_inline_qos,
                is_active,
            );
            writer.matched_reader_add(proxy);
        }
    }

    pub fn discovered_participant_add_subscriptions_reader(
        &self,
        reader: &mut (impl RtpsStatefulReaderOperations<L>
                  + RtpsStatefulReader<WriterProxyType = impl RtpsWriterProxyOperations>),
    ) where
        L: Clone,
        for<'b> &'b L: IntoIterator<Item = &'b Locator>,
    {
        if self
            .participant_data
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER)
        {
            let remote_writer_guid = Guid::new(
                self.participant_data.guid_prefix,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let data_max_size_serialized = None;

            let proxy = RtpsWriterProxy {
                remote_writer_guid,
                unicast_locator_list: self
                    .participant_data
                    .metatraffic_unicast_locator_list
                    .clone(),
                multicast_locator_list: self
                    .participant_data
                    .metatraffic_unicast_locator_list
                    .clone(),
                data_max_size_serialized,
                remote_group_entity_id,
            };
            reader.matched_writer_add(proxy);
        }
    }

    pub fn discovered_participant_add_topics_writer(
        &self,
        writer: &mut (impl RtpsStatefulWriterOperations
                  + RtpsStatefulWriter<ReaderProxyType = impl RtpsReaderProxyOperations>),
    ) where
        for<'b> &'b L: IntoIterator<Item = &'b Locator>,
    {
        if self
            .participant_data
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_TOPICS_DETECTOR)
        {
            let guid = Guid::new(
                self.participant_data.guid_prefix,
                ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let expects_inline_qos = false;
            let is_active = true;
            let proxy = RtpsReaderProxyOperations::new(
                guid,
                remote_group_entity_id,
                &self.participant_data.metatraffic_unicast_locator_list,
                &self.participant_data.metatraffic_multicast_locator_list,
                expects_inline_qos,
                is_active,
            );
            writer.matched_reader_add(proxy);
        }
    }

    pub fn discovered_participant_add_topics_reader(
        &self,
        reader: &mut (impl RtpsStatefulReaderOperations<L>
                  + RtpsStatefulReader<WriterProxyType = impl RtpsWriterProxyOperations>),
    ) where
        L: Clone,
        for<'b> &'b L: IntoIterator<Item = &'b Locator>,
    {
        if self
            .participant_data
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_TOPICS_ANNOUNCER)
        {
            let remote_writer_guid = Guid::new(
                self.participant_data.guid_prefix,
                ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let data_max_size_serialized = None;

            let proxy =  RtpsWriterProxy {
                remote_writer_guid,
                unicast_locator_list: self
                    .participant_data
                    .metatraffic_unicast_locator_list
                    .clone(),
                multicast_locator_list: self
                    .participant_data
                    .metatraffic_unicast_locator_list
                    .clone(),
                data_max_size_serialized,
                remote_group_entity_id,
            };
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
