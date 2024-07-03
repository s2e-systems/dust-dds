from dataclasses import dataclass
from dust_dds import DomainParticipantFactory, UserDataQosPolicy, DomainParticipantQos, TypeKind
import dust_dds

@dataclass
class MyTopicType:
    id: TypeKind.uint8

def test_set_default_participant_qos():
    participant_factory = DomainParticipantFactory.get_instance()
    participant_qos = DomainParticipantQos(user_data=UserDataQosPolicy([0,1,2,3]))
    participant_factory.set_default_participant_qos(participant_qos)

def test_create_delete_publisher():
    participant_factory = DomainParticipantFactory.get_instance()
    participant = participant_factory.create_participant(100, None)
    publisher = participant.create_publisher()
    participant.delete_publisher(publisher)

def test_create_delete_subscriber():
    participant_factory = DomainParticipantFactory.get_instance()
    participant = participant_factory.create_participant(101)
    subscriber = participant.create_subscriber()
    participant.delete_subscriber(subscriber)

def test_create_delete_topic():
    participant_factory = DomainParticipantFactory.get_instance()
    participant = participant_factory.create_participant(102)
    topic = participant.create_topic("MyTopicName", MyTopicType)
    participant.delete_topic(topic)