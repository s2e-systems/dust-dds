use crate::{
    behavior::writer::{
        reader_proxy::RTPSReaderProxyOperations,
        stateful_writer::{RTPSStatefulWriter, RTPSStatefulWriterOperations},
    },
    discovery::sedp::builtin_endpoints::ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
    structure::types::{Locator, ENTITYID_UNKNOWN, GUID},
};

use super::{
    sedp::sedp_participant::SedpParticipant,
    spdp::spdp_discovered_participant_data::SPDPdiscoveredParticipantData,
    types::BuiltinEndpointSet,
};

pub fn discover_new_remote_participant<Participant, ParticipantData>(
    participant_data: &ParticipantData,
    local_participant: &mut Participant,
) where
    ParticipantData: SPDPdiscoveredParticipantData,
    ParticipantData::LocatorListType: IntoIterator<Item = Locator>,
    Participant: SedpParticipant + SPDPdiscoveredParticipantData,
    Participant::BuiltinPublicationsWriter: RTPSStatefulWriterOperations + RTPSStatefulWriter,
    <Participant::BuiltinPublicationsWriter as RTPSStatefulWriter>::ReaderProxyType:
        RTPSReaderProxyOperations,
{
    // Check that the domainId of the discovered participant equals the local one.
    // If it is not equal then there the local endpoints are not configured to
    // communicate with the discovered participant.
    if participant_data.domain_id() != local_participant.domain_id() {
        return;
    }

    // Check that the domainTag of the discovered participant equals the local one.
    // If it is not equal then there the local endpoints are not configured to
    // communicate with the discovered participant.
    if participant_data.domain_tag() != local_participant.domain_tag() {
        return;
    }

    if participant_data
        .available_builtin_endpoints()
        .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR)
    {
        let guid = GUID::new(
            participant_data.guid_prefix(),
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
        );
        let remote_group_entity_id = ENTITYID_UNKNOWN;
        let expects_inline_qos = false;
        let is_active = true;
        let proxy = RTPSReaderProxyOperations::new(
            guid,
            remote_group_entity_id,
            participant_data.metatraffic_unicast_locator_list(),
            participant_data.metatraffic_multicast_locator_list(),
            expects_inline_qos,
            is_active,
        );
        local_participant
            .sedp_builtin_publications_writer()
            .matched_reader_add(proxy);
    }
}

pub fn remove_previously_discovered_participant() {}
