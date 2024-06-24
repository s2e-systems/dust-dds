import dust_dds_python
import time

def test_write_read():
    participant_factory = dust_dds_python.participant.DomainParticipantFactory.get_instance()
    participant = participant_factory.create_participant(domain_id = 100)
    topic = participant.create_topic(topic_name = "TestTopic", type_name = "TestType")

    publisher = participant.create_publisher()
    data_writer = publisher.create_datawriter(topic)

    subscriber = participant.create_subscriber()
    data_reader = subscriber.create_datareader(topic)

    # Wait for discovery
    time.sleep(2)

    data = dust_dds_python.topic_definition.MyDdsData([0,1,2,3,4])
    data_writer.write(data)

    # Wait for data to be received
    time.sleep(2)

    received_data = data_reader.read(max_samples = 1)

    print(f"Received data {received_data[0].data.value}")
    assert data.value == received_data[0].data.value