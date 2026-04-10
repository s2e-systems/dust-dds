use dust_dds::domain::domain_participant_factory::DomainParticipantFactory;

fn main() {
    let participant_factory = DomainParticipantFactory::get_instance();
    let mut transport = participant_factory.get_mut_transport();
    transport.set_interface_name(Some(String::from("Wi-Fi")));
    transport.set_fragment_size(500).unwrap();
    transport.set_udp_receive_buffer_size(Some(10000));
}
