use crate::types::{GuidPrefix, GUID, ReliabilityKind};

use crate::discovery::spdp_data::SPDPdiscoveredParticipantData;
use crate::endpoint_types::BuiltInEndpointSet;
use crate::behavior::{StatelessWriter, StatelessReader};
use crate::behavior::{ReaderProxy, WriterProxy};
use crate::discovery::sedp::SimpleEndpointDiscoveryProtocol;

use crate::types::constants::{
    ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR,
    ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
    ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
    ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
};

use rust_dds_interface::types::{DomainId, ChangeKind, TopicKind, };
use rust_dds_interface::history_cache::HistoryCache;



pub struct SimpleParticipantDiscovery {
    spdp_builtin_participant_writer: StatelessWriter,
    spdp_builtin_participant_reader: StatelessReader,
    domain_id: DomainId,
    domain_tag: String,
}

impl SimpleParticipantDiscovery {
    pub fn new(
        spdp_data: SPDPdiscoveredParticipantData
    ) -> Self {
        let guid_prefix = spdp_data.guid_prefix();

        let guid = GUID::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER);
        
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::BestEffort;
        
        let push_mode = true;
        let writer_cache = HistoryCache::default();
        let data_max_sized_serialized = None;
        let mut spdp_builtin_participant_writer = StatelessWriter::new(guid, topic_kind, reliability_level, push_mode, writer_cache, data_max_sized_serialized);

        let change = spdp_builtin_participant_writer.writer.new_change(ChangeKind::Alive, Some(spdp_data.data()), None, spdp_data.key());
        spdp_builtin_participant_writer.writer.writer_cache.add_change(change).unwrap();

        for locator in spdp_data.metatraffic_multicast_locator_list() {
            spdp_builtin_participant_writer.reader_locator_add(locator.clone());
        }


        let guid = GUID::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR);
        
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::BestEffort;
        let expects_inline_qos = false;

        let reader_cache = HistoryCache::default();
        let spdp_builtin_participant_reader = StatelessReader::new(guid, topic_kind, reliability_level, reader_cache, expects_inline_qos);

        Self {
            spdp_builtin_participant_writer,
            spdp_builtin_participant_reader,
            domain_id: spdp_data.domain_id(),
            domain_tag: spdp_data.domain_tag().clone(),
        }
    }

    fn add_discovered_participant(discovered_participant: &SPDPdiscoveredParticipantData, sedp: &mut SimpleEndpointDiscoveryProtocol) {
        // Implements the process described in 8.5.5.1 - Discovery of a new remote Participant
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
            sedp.sedp_builtin_publications_writer().matched_reader_add(proxy);
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
            sedp.sedp_builtin_publications_reader().matched_writer_add(proxy);
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
            sedp.sedp_builtin_subscriptions_writer().matched_reader_add(proxy);
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
            sedp.sedp_builtin_subscriptions_reader().matched_writer_add(proxy);
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
            sedp.sedp_builtin_topics_writer().matched_reader_add(proxy);
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
            sedp.sedp_builtin_topics_reader().matched_writer_add(proxy);
        }
    }

    fn remove_discovered_participant(remote_participant_guid_prefix: GuidPrefix, sedp: &mut SimpleEndpointDiscoveryProtocol) {
        // Implements the process described in 8.5.5.2 Removal of a previously discovered Participant
        let guid = GUID::new(
            remote_participant_guid_prefix,
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
        );
        sedp
            .sedp_builtin_publications_writer()
            .matched_reader_remove(&guid);

        let guid = GUID::new(
            remote_participant_guid_prefix,
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
        );
        sedp
            .sedp_builtin_publications_reader()
            .matched_writer_remove(&guid);

        let guid = GUID::new(
            remote_participant_guid_prefix,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
        );
        sedp
            .sedp_builtin_subscriptions_writer()
            .matched_reader_remove(&guid);

        let guid = GUID::new(
            remote_participant_guid_prefix,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
        );
        sedp
            .sedp_builtin_subscriptions_reader()
            .matched_writer_remove(&guid);

        let guid = GUID::new(
            remote_participant_guid_prefix,
            ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
        );
        sedp
            .sedp_builtin_topics_writer()
            .matched_reader_remove(&guid);

        let guid = GUID::new(
            remote_participant_guid_prefix,
            ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
        );
        sedp
            .sedp_builtin_topics_reader()
            .matched_writer_remove(&guid);
    }

    pub fn on_add_change(&mut self, sedp: &mut SimpleEndpointDiscoveryProtocol) {
        let seq_num_min = self.spdp_builtin_participant_reader.reader.reader_cache.get_seq_num_min().unwrap();
        let seq_num_max = self.spdp_builtin_participant_reader.reader.reader_cache.get_seq_num_max().unwrap();
        for seq_num in seq_num_min..=seq_num_max {   
            let cc = self.spdp_builtin_participant_reader.reader.reader_cache.get_change(seq_num).unwrap();              

            let discovered_participant = SPDPdiscoveredParticipantData::from_key_data(cc.instance_handle(), cc.data_value().as_ref().unwrap(), 0);

            if discovered_participant.domain_id() == self.domain_id && discovered_participant.domain_tag() == &self.domain_tag {      
                match cc.change_kind() {
                    ChangeKind::Alive => Self::add_discovered_participant(&discovered_participant, sedp),
                    ChangeKind::AliveFiltered => (),
                    ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered => {
                        Self::remove_discovered_participant(discovered_participant.guid_prefix(), sedp)
                    }
                }
                
                self.spdp_builtin_participant_reader.reader.reader_cache.remove_change(seq_num);      
            }      
        }  
    }


    pub fn spdp_builtin_participant_writer(&mut self) -> &mut StatelessWriter {
        &mut self.spdp_builtin_participant_writer
    }

    pub fn spdp_builtin_participant_reader(&mut self) -> &mut StatelessReader {
        &mut self.spdp_builtin_participant_reader
    }
}

