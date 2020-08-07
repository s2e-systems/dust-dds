use crate::stateless_writer::StatelessWriter;
use crate::stateless_reader::StatelessReader;
use crate::stateful_writer::{StatefulWriter, ReaderProxy};
use crate::stateful_reader::{StatefulReader, WriterProxy};
use crate::types::{GUID, GuidPrefix, Locator, ProtocolVersion, VendorId, TopicKind, ChangeKind, ReliabilityKind};
use crate::types::constants::{
    ENTITYID_PARTICIPANT,
    ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
    ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR,
    ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
    ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
    ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
    ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
    ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
    ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
    LOCATOR_KIND_UDPv4};
use crate::endpoint_types::BuiltInEndpointSet;
use crate::messages::Endianness;
use crate::behavior::types::Duration;
use crate::behavior::types::constants::DURATION_ZERO;
use crate::spdp::SPDPdiscoveredParticipantData;
use crate::transport::{Transport, UdpTransport};
use crate::messages::message_sender::rtps_message_sender;
use crate::messages::message_receiver::rtps_message_receiver;
use crate::endpoint_types::DomainId;


pub struct Participant<T: Transport = UdpTransport> {
    guid: GUID,
    domain_id: DomainId,
    default_unicast_locator_list: Vec<Locator>,
    default_multicast_locator_list: Vec<Locator>,
    metatraffic_unicast_locator_list: Vec<Locator>,
    metatraffic_multicast_locator_list: Vec<Locator>,
    protocol_version: ProtocolVersion,
    vendor_id: VendorId,
    domain_tag: String,
    metatraffic_transport: T,
    spdp_builtin_participant_reader: StatelessReader,
    spdp_builtin_participant_writer: StatelessWriter,
    builtin_endpoint_set: BuiltInEndpointSet,
    sedp_builtin_publications_reader: StatefulReader,
    sedp_builtin_publications_writer: StatefulWriter,
    sedp_builtin_subscriptions_reader: StatefulReader,
    sedp_builtin_subscriptions_writer: StatefulWriter,
    sedp_builtin_topics_reader: StatefulReader,
    sedp_builtin_topics_writer: StatefulWriter,
}

impl<T: Transport> Participant<T> {
    fn new(
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        protocol_version: ProtocolVersion,
        vendor_id: VendorId,
    ) -> Self {
        let domain_id = 0; // TODO: Should be configurable
        let lease_duration = Duration::from_secs(100); // TODO: Should be configurable
        let endianness = Endianness::LittleEndian; // TODO: Should be configurable
        let expects_inline_qos = false;
        const PB : u32 = 7400;  // TODO: Should be configurable
        const DG : u32 = 250;   // TODO: Should be configurable
        const PG : u32 = 2; // TODO: Should be configurable
        const D0 : u32 = 0; // TODO: Should be configurable
        const D1 : u32 = 10;    // TODO: Should be configurable
        const D2 : u32 = 1; // TODO: Should be configurable
        const D3 : u32 = 11;    // TODO: Should be configurable

        let guid_prefix = [5, 6, 7, 8, 9, 5, 1, 2, 3, 4, 10, 11];   // TODO: Should be uniquely generated

        let spdp_well_known_multicast_port = PB + DG * domain_id + D0;

        let metatraffic_unicast_locator = Locator::new(
            LOCATOR_KIND_UDPv4,
            spdp_well_known_multicast_port,
            crate::transport::get_interface_address(&"Ethernet").unwrap(),
        );

        let metatraffic_multicast_locator = Locator::new(
            LOCATOR_KIND_UDPv4,
            spdp_well_known_multicast_port,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1],
        );

        let metatraffic_transport = T::new(metatraffic_unicast_locator, Some(metatraffic_multicast_locator)).unwrap();

        let spdp_builtin_participant_writer = StatelessWriter::new(
            GUID::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER),
            TopicKind::WithKey);
        spdp_builtin_participant_writer.reader_locator_add(metatraffic_multicast_locator);

        let spdp_builtin_participant_reader = StatelessReader::new(
            GUID::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR),
            TopicKind::WithKey,
            vec![],
            vec![metatraffic_multicast_locator],
            expects_inline_qos,
        );
        
        

        let expects_inline_qos = false;
        let heartbeat_period = Duration::from_secs(5);
        let heartbeat_response_delay = Duration::from_millis(500);
        let nack_response_delay = DURATION_ZERO;
        let nack_supression_duration = DURATION_ZERO;


        let sedp_builtin_publications_reader = StatefulReader::new(
            GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR),
            TopicKind::WithKey,
            ReliabilityKind::Reliable,
            expects_inline_qos,
            heartbeat_response_delay,
        );

        let sedp_builtin_publications_writer = StatefulWriter::new(
            GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER),
            TopicKind::WithKey,
            ReliabilityKind::Reliable,
            true,
            heartbeat_period,
            nack_response_delay,
            nack_supression_duration
        );

        let sedp_builtin_subscriptions_reader = StatefulReader::new(
            GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR),
            TopicKind::WithKey,
            ReliabilityKind::Reliable,
            expects_inline_qos,
            heartbeat_response_delay,
        );

        let sedp_builtin_subscriptions_writer = StatefulWriter::new(
            GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER),
            TopicKind::WithKey,
            ReliabilityKind::Reliable,
            true,
            heartbeat_period,
            nack_response_delay,
            nack_supression_duration
        );
        
        let sedp_builtin_topics_reader = StatefulReader::new(
            GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR),
            TopicKind::WithKey,
            ReliabilityKind::Reliable,
            expects_inline_qos,
            heartbeat_response_delay,
        );

        let sedp_builtin_topics_writer = StatefulWriter::new(
            GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER),
            TopicKind::WithKey,
            ReliabilityKind::Reliable,
            true,
            heartbeat_period,
            nack_response_delay,
            nack_supression_duration
        );

        let builtin_endpoint_set = BuiltInEndpointSet::new(
            BuiltInEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_ANNOUNCER |
            BuiltInEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR |
            BuiltInEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER |
            BuiltInEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR |
            BuiltInEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER |
            BuiltInEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR |
            BuiltInEndpointSet::BUILTIN_ENDPOINT_TOPICS_ANNOUNCER |
            BuiltInEndpointSet::BUILTIN_ENDPOINT_TOPICS_DETECTOR
        );

        // Fill up the metatraffic locator lists. By default only the SPDP will
        // use the multicast and the remaining built-in endpoints will communicate
        // over unicast.
        let metatraffic_unicast_locator_list = vec![metatraffic_unicast_locator];
        let metatraffic_multicast_locator_list = vec![];

        let participant = Self {
            guid: GUID::new(guid_prefix,ENTITYID_PARTICIPANT ),
            domain_id,
            default_unicast_locator_list,
            default_multicast_locator_list,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            protocol_version,
            vendor_id,
            domain_tag: "".to_string(),
            metatraffic_transport,
            builtin_endpoint_set,
            spdp_builtin_participant_reader,
            spdp_builtin_participant_writer,
            sedp_builtin_publications_reader,
            sedp_builtin_publications_writer,
            sedp_builtin_subscriptions_reader,
            sedp_builtin_subscriptions_writer,
            sedp_builtin_topics_reader,
            sedp_builtin_topics_writer,
        };

        let spdp_discovered_data = SPDPdiscoveredParticipantData::new_from_participant(&participant, lease_duration);
        let spdp_change = participant.spdp_builtin_participant_writer.new_change(ChangeKind::Alive,Some(spdp_discovered_data.data(endianness)) , None, spdp_discovered_data.key());
        participant.spdp_builtin_participant_writer.writer_cache().add_change(spdp_change);
        
        participant
    }

    pub fn guid(&self) -> GUID {
        self.guid
    }

    pub fn domain_id(&self) -> DomainId {
        self.domain_id
    }

    pub fn protocol_version(&self) -> ProtocolVersion {
        self.protocol_version
    }

    pub fn vendor_id(&self) -> VendorId {
        self.vendor_id
    }

    pub fn default_unicast_locator_list(&self) -> &Vec<Locator> {
        &self.default_unicast_locator_list
    }

    pub fn default_multicast_locator_list(&self) -> &Vec<Locator> {
        &self.default_multicast_locator_list
    }

    pub fn metatraffic_unicast_locator_list(&self) -> &Vec<Locator> {
        &self.metatraffic_unicast_locator_list
    }

    pub fn metatraffic_multicast_locator_list(&self) -> &Vec<Locator> {
        &self.metatraffic_multicast_locator_list
    }

    pub fn builtin_endpoint_set(&self) -> BuiltInEndpointSet {
        self.builtin_endpoint_set
    }

    pub fn domain_tag(&self) -> &String {
        &self.domain_tag
    }

    fn run(&self) {
        rtps_message_receiver(
            &self.metatraffic_transport, 
            self.guid.prefix(), 
            &[&self.spdp_builtin_participant_reader],
        &[&self.sedp_builtin_publications_reader, &self.sedp_builtin_subscriptions_reader, &self.sedp_builtin_topics_reader]);
        self.spdp_builtin_participant_reader.run();
        self.sedp_builtin_publications_reader.run();
        self.sedp_builtin_subscriptions_reader.run();
        self.sedp_builtin_topics_reader.run();

        self.spdp_builtin_participant_writer.run();
        self.sedp_builtin_publications_writer.run();
        self.sedp_builtin_subscriptions_writer.run();
        self.sedp_builtin_topics_writer.run();
        rtps_message_sender(&self.metatraffic_transport, self.guid.prefix(), &[&self.spdp_builtin_participant_writer],
    &[&self.sedp_builtin_publications_writer, &self.sedp_builtin_subscriptions_writer, &self.sedp_builtin_topics_writer]);
    }

    
    fn add_discovered_participant(&self, discovered_participant: &SPDPdiscoveredParticipantData) {
        // Implements the process described in
        // 8.5.5.1 Discovery of a new remote Participant

        if discovered_participant.domain_id() != self.domain_id {
            return;
        }

        if discovered_participant.domain_tag() != &self.domain_tag {
            return;
        }

        if discovered_participant.available_built_in_endpoints().has(BuiltInEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR) {
            let guid = GUID::new(discovered_participant.guid_prefix(), ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);
            let proxy = ReaderProxy::new(
                guid,
                discovered_participant.metatraffic_unicast_locator_list().clone(),
            discovered_participant.metatraffic_multicast_locator_list().clone(),
        discovered_participant.expects_inline_qos(),
    true );
            self.sedp_builtin_publications_writer.matched_reader_add(proxy);
        }

        if discovered_participant.available_built_in_endpoints().has(BuiltInEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER) {
            let guid = GUID::new(discovered_participant.guid_prefix(), ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER);
            let proxy = WriterProxy::new(
                guid,
                discovered_participant.metatraffic_unicast_locator_list().clone(), 
                discovered_participant.metatraffic_multicast_locator_list().clone());
            self.sedp_builtin_publications_reader.matched_writer_add(proxy);
        }

        if discovered_participant.available_built_in_endpoints().has(BuiltInEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR) {
            let guid = GUID::new(discovered_participant.guid_prefix(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            let proxy = ReaderProxy::new(
                guid,
                discovered_participant.metatraffic_unicast_locator_list().clone(),
            discovered_participant.metatraffic_multicast_locator_list().clone(),
        discovered_participant.expects_inline_qos(),
    true );
            self.sedp_builtin_subscriptions_writer.matched_reader_add(proxy);
        }
        
        if discovered_participant.available_built_in_endpoints().has(BuiltInEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER) {
            let guid = GUID::new(discovered_participant.guid_prefix(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            let proxy = WriterProxy::new(
                guid,
                discovered_participant.metatraffic_unicast_locator_list().clone(), 
                discovered_participant.metatraffic_multicast_locator_list().clone());
            self.sedp_builtin_subscriptions_reader.matched_writer_add(proxy);
        }

        if discovered_participant.available_built_in_endpoints().has(BuiltInEndpointSet::BUILTIN_ENDPOINT_TOPICS_DETECTOR) {
            let guid = GUID::new(discovered_participant.guid_prefix(), ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR);
            let proxy = ReaderProxy::new(
                guid,
                discovered_participant.metatraffic_unicast_locator_list().clone(),
            discovered_participant.metatraffic_multicast_locator_list().clone(),
        discovered_participant.expects_inline_qos(),
    true );
            self.sedp_builtin_topics_writer.matched_reader_add(proxy);
        }

        if discovered_participant.available_built_in_endpoints().has(BuiltInEndpointSet::BUILTIN_ENDPOINT_TOPICS_ANNOUNCER) {
            let guid = GUID::new(discovered_participant.guid_prefix(), ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER);
            let proxy = WriterProxy::new(
                guid,
                discovered_participant.metatraffic_unicast_locator_list().clone(), 
                discovered_participant.metatraffic_multicast_locator_list().clone());
            self.sedp_builtin_topics_reader.matched_writer_add(proxy);
        }           
    }

    fn remove_discovered_participant(&self, remote_participant_guid_prefix: GuidPrefix) {
        // Implements the process described in
        // 8.5.5.2 Removal of a previously discovered Participant
        let guid = GUID::new(remote_participant_guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);
        self.sedp_builtin_publications_writer.matched_reader_remove(&guid);

        let guid = GUID::new(remote_participant_guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER);
        self.sedp_builtin_publications_reader.matched_writer_remove(&guid);

        let guid = GUID::new(remote_participant_guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        self.sedp_builtin_subscriptions_writer.matched_reader_remove(&guid);

        let guid = GUID::new(remote_participant_guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        self.sedp_builtin_subscriptions_reader.matched_writer_remove(&guid);

        let guid = GUID::new(remote_participant_guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR);
        self.sedp_builtin_topics_writer.matched_reader_remove(&guid);

        let guid = GUID::new(remote_participant_guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER);
        self.sedp_builtin_topics_reader.matched_writer_remove(&guid);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::constants::{PROTOCOL_VERSION_2_4};
    use crate::transport_stub::StubTransport;

    #[test]
    fn participant_with_default_transport() {
        // The weird syntax is needed to use the default transport without
        // infering anything. See: https://github.com/rust-lang/rust/issues/36980#issuecomment-251726254 
        // and https://users.rust-lang.org/t/default-trait-type-not-working-for-type-inference/33905
        let participant = <Participant>::new(
            vec![],
            vec![],
            PROTOCOL_VERSION_2_4,
            [99,99]);

        participant.run();
    }


    #[test]
    fn participant() {
        let participant = Participant::<StubTransport>::new(
            vec![],
            vec![],
            PROTOCOL_VERSION_2_4,
            [99,99]);
    
        participant.run();

        println!("Message: {:?}",participant.metatraffic_transport.pop_write());
    }
}
