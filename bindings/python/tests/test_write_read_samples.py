import dust_dds_python

def test_write_read():
    participant_factory = dust_dds_python.participant.DomainParticipantFactory.get_instance()
    participant = participant_factory.create_participant(100)
    publisher = participant.create_publisher()
    topic = participant.create_topic("TestTopic", "TestType")
    data_writer = publisher.create_datawriter(topic)
