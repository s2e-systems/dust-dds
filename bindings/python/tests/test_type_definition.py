from dataclasses import dataclass
import dust_dds
import time

@dataclass
class MyDataType:
    id : dust_dds.TypeKind.uint32
    state : dust_dds.TypeKind.boolean
    def _key():
        return ["id"]


def test_write_read_my_data_type():
    participant_factory = dust_dds.DomainParticipantFactory()
    participant = participant_factory.create_participant(domain_id = 100)
    topic = participant.create_topic(topic_name = "TestTopic", type_ = MyDataType)

    publisher = participant.create_publisher()
    data_writer = publisher.create_datawriter(topic)

    subscriber = participant.create_subscriber()
    data_reader = subscriber.create_datareader(topic)

    # Wait for discovery
    time.sleep(2)

    data = MyDataType(231, True)
    data_writer.write(data)

    # Wait for data to be received
    time.sleep(2)

    received_data = data_reader.read(max_samples = 1)

    print(f"Received data {received_data[0].data}")
    assert data == received_data[0].data