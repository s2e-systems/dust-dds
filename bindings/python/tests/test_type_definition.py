from dataclasses import dataclass
import dust_dds

@dataclass
class MyDataType:
    id : dust_dds.TypeKind.uint32
    state : dust_dds.TypeKind.boolean
    data: bytes
    msg: str
    def _key():
        return ["id"]


def test_write_read_my_data_type():
    participant_factory = dust_dds.DomainParticipantFactory.get_instance()
    participant = participant_factory.create_participant(domain_id = 100)
    topic = participant.create_topic(topic_name = "TestTopic", type_ = MyDataType)

    publisher = participant.create_publisher()
    data_writer = publisher.create_datawriter(topic)

    subscriber = participant.create_subscriber()
    data_reader = subscriber.create_datareader(topic)

    # Wait for discovery
    ws = dust_dds.WaitSet()
    cond = data_writer.get_statuscondition()
    cond.set_enabled_statuses([dust_dds.StatusKind.PublicationMatched])
    ws.attach_condition(dust_dds.Condition.StatusCondition(cond))

    ws.wait(dust_dds.Duration(sec=2, nanosec=0))

    data = MyDataType(231, True, bytes([1,2,3]), "hello world")
    data_writer.write(data)

    # Wait for data to be received
    ws_data_available = dust_dds.WaitSet()
    cond = data_reader.get_statuscondition()
    cond.set_enabled_statuses([dust_dds.StatusKind.DataAvailable])
    ws_data_available.attach_condition(dust_dds.Condition.StatusCondition(cond))

    ws_data_available.wait(dust_dds.Duration(sec=2, nanosec=0))

    received_data = data_reader.read(max_samples = 1 )

    assert data == received_data[0].get_data()