use dust_dds::domain::domain_participant_factory::DomainParticipantFactory;

#[test]
fn get_subscriber_parent_participant() {
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(0, None, None, &[])
        .unwrap();

    let subscriber = participant.create_subscriber(None, None, &[]).unwrap();

    let subscriber_parent_participant = subscriber.get_participant().unwrap();

    assert_eq!(participant, subscriber_parent_participant);
}
