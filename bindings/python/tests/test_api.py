import dust_dds

def test_set_default_participant_qos():
    participant_factory = dust_dds.DomainParticipantFactory()
    participant_qos = dust_dds.DomainParticipantQos(user_data=dust_dds.UserDataQosPolicy([0,1,2,3]))
    participant_factory.set_default_participant_qos(participant_qos)

def test_create_delete_publisher():
    participant_factory = dust_dds.DomainParticipantFactory()
    participant = participant_factory.create_participant(100)
    publisher = participant.create_publisher()
    participant.delete_publisher(publisher)

def test_create_delete_subscriber():
    participant_factory = dust_dds.DomainParticipantFactory()
    participant = participant_factory.create_participant(101)
    subscriber = participant.create_subscriber()
    participant.delete_subscriber(subscriber)

def test_create_delete_topic():
    participant_factory = dust_dds.DomainParticipantFactory()
    participant = participant_factory.create_participant(102)
    topic = participant.create_topic("MyTopicName", "MyTopicType")
    participant.delete_topic(topic)