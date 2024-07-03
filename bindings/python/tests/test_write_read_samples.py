from dataclasses import dataclass
import dust_dds
import time

@dataclass
class MyDataType:
    data: bytes

class MyReaderListener:
    def on_data_available(reader):
        received_data = reader.read(max_samples = 1)
        print(f"On data available, data: {received_data[0].get_data()}")

def test_write_read():
    participant_factory = dust_dds.DomainParticipantFactory.get_instance()
    participant = participant_factory.create_participant(domain_id = 100)
    topic = participant.create_topic(topic_name = "TestTopic", type_ = MyDataType)

    publisher = participant.create_publisher()
    data_writer = publisher.create_datawriter(topic)

    subscriber = participant.create_subscriber()
    data_reader = subscriber.create_datareader(topic, a_listener = MyReaderListener, mask=[dust_dds.StatusKind.DataAvailable] )

    # Wait for discovery
    ws = dust_dds.WaitSet()
    cond = data_writer.get_statuscondition()
    cond.set_enabled_statuses([dust_dds.StatusKind.PublicationMatched])
    ws.attach_condition(dust_dds.Condition.StatusCondition(cond))

    data = MyDataType(bytes([0,1,2,3,4]))
    data_writer.write(data)

    # Wait for data to be received
    ws_data_available = dust_dds.WaitSet()
    cond = data_reader.get_statuscondition()
    cond.set_enabled_statuses([dust_dds.StatusKind.DataAvailable])
    ws_data_available.attach_condition(dust_dds.Condition.StatusCondition(cond))

    ws_data_available.wait(dust_dds.Duration(sec=2, nanosec=0))

    received_data = data_reader.read(max_samples = 1)

    assert data == received_data[0].get_data()