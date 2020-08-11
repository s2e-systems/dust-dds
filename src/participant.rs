use crate::stateless_writer::StatelessWriter;
use crate::stateless_reader::StatelessReader;
use crate::stateful_writer::{StatefulWriter, };
use crate::stateful_reader::{StatefulReader, };
use crate::types::{GUID, Locator, ProtocolVersion, VendorId, TopicKind, ChangeKind, ReliabilityKind};
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
    PROTOCOL_VERSION_2_4,};
use crate::endpoint_types::BuiltInEndpointSet;
use crate::messages::Endianness;
use crate::behavior::types::Duration;
use crate::behavior::types::constants::DURATION_ZERO;
use crate::spdp::SPDPdiscoveredParticipantData;
use crate::transport::{Transport, UdpTransport};
use crate::messages::message_sender::rtps_message_sender;
use crate::messages::message_receiver::rtps_message_receiver;
use crate::endpoint_types::DomainId;
use crate::spdp;


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
    userdata_transport: T,
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
        userdata_transport: T,
        metatraffic_transport: T,
    ) -> Self {
        let domain_id = 0; // TODO: Should be configurable
        let protocol_version = PROTOCOL_VERSION_2_4;
        let vendor_id = [99,99];
        let lease_duration = Duration::from_secs(100); // TODO: Should be configurable
        let endianness = Endianness::LittleEndian; // TODO: Should be configurable
        let expects_inline_qos = false;
        let guid_prefix = [5, 6, 7, 8, 9, 5, 1, 2, 3, 4, 10, 11];   // TODO: Should be uniquely generated

        let spdp_builtin_participant_writer = StatelessWriter::new(
            GUID::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER),
            TopicKind::WithKey);

        for metatraffic_multicast_locator in metatraffic_transport.multicast_locator_list() {
            spdp_builtin_participant_writer.reader_locator_add(metatraffic_multicast_locator);
        }

        let spdp_builtin_participant_reader = StatelessReader::new(
            GUID::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR),
            TopicKind::WithKey,
            vec![],
            metatraffic_transport.multicast_locator_list(),
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

        let default_unicast_locator_list = userdata_transport.unicast_locator_list().clone();
        let default_multicast_locator_list = userdata_transport.multicast_locator_list().clone();

        // Fill up the metatraffic locator lists. By default only the SPDP will
        // use the multicast and the remaining built-in endpoints will communicate
        // over unicast.
        let metatraffic_unicast_locator_list = metatraffic_transport.unicast_locator_list().clone();
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
            userdata_transport,
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

    pub fn sedp_builtin_publications_reader(&self) -> &StatefulReader {
        &self.sedp_builtin_publications_reader
    }

    pub fn sedp_builtin_publications_writer(&self) -> &StatefulWriter {
        &self.sedp_builtin_publications_writer
    }

    pub fn sedp_builtin_subscriptions_reader(&self) -> &StatefulReader {
        &self.sedp_builtin_publications_reader
    }

    pub fn sedp_builtin_subscriptions_writer(&self) -> &StatefulWriter {
        &self.sedp_builtin_publications_writer
    }

    pub fn sedp_builtin_topics_reader(&self) -> &StatefulReader {
        &self.sedp_builtin_topics_reader
    }

    pub fn sedp_builtin_topics_writer(&self) -> &StatefulWriter {
        &self.sedp_builtin_topics_writer
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

        for spdp_data in self.spdp_builtin_participant_reader.reader_cache().changes().iter() {
            let discovered_participant = SPDPdiscoveredParticipantData::from_key_data(*spdp_data.instance_handle(), spdp_data.data_value().unwrap(), self.domain_id);
            spdp::add_discovered_participant(&self, &discovered_participant);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport_stub::StubTransport;

    // #[test]
    // fn participant_with_default_transport() {
    //     // The weird syntax is needed to use the default transport without
    //     // infering anything for the type. See: https://github.com/rust-lang/rust/issues/36980#issuecomment-251726254 
    //     // and https://users.rust-lang.org/t/default-trait-type-not-working-for-type-inference/33905
    //     let participant = <Participant>::new(
    //         vec![],
    //         vec![]);

    //     participant.run();
    // }


    #[test]
    fn participant() {
        let userdata_transport1 = StubTransport::new(
            Locator::new_udpv4(7410, [192,168,0,5]), 
            Some(Locator::new_udpv4(7410, [239,255,0,1]))).unwrap();
        let metatraffic_transport1 = StubTransport::new(
            Locator::new_udpv4(7400, [192,168,0,5]), 
            Some(Locator::new_udpv4(7400, [239,255,0,1]))).unwrap();

        
        let participant_1 = Participant::new(userdata_transport1,metatraffic_transport1);


        let userdata_transport2 = StubTransport::new(
            Locator::new_udpv4(7410, [192,168,0,10]), 
            Some(Locator::new_udpv4(7410, [239,255,0,1]))).unwrap();
        let metatraffic_transport2 = StubTransport::new(
            Locator::new_udpv4(7400, [192,168,0,10]), 
            Some(Locator::new_udpv4(7400, [239,255,0,1]))).unwrap();

        let participant_2 = Participant::<StubTransport>::new(
            userdata_transport2,
            metatraffic_transport2);

        participant_1.run();

        participant_2.metatraffic_transport.receive_from(&participant_1.metatraffic_transport);

        participant_2.run();
        participant_1.metatraffic_transport.receive_from(&participant_2.metatraffic_transport);

        // For now just check that a cache change is added to the receiver. TODO: Check that the discovery
        // worked properly
        assert_eq!(participant_2.spdp_builtin_participant_reader.reader_cache().changes().len(), 1);

        assert_eq!(participant_1.spdp_builtin_participant_reader.reader_cache().changes().len(), 0);
        participant_1.run();
        assert_eq!(participant_1.spdp_builtin_participant_reader.reader_cache().changes().len(), 1);
        
    }
}
