from dataclasses import dataclass
import dust_dds
import threading
import time

@dataclass
class MyDataType:
    data: bytes

class MyReaderListener:
    def __init__(self, semaphore):
        self.semaphore = semaphore

    def on_data_available(self, reader):
        print("On listener")
        self.semaphore.release()

def test_write_read():
    listener_semaphore = threading.Semaphore()
    listener_semaphore.acquire()

    participant_factory = dust_dds.DomainParticipantFactory.get_instance()
    participant = participant_factory.create_participant(domain_id = 100)
    topic = participant.create_topic(topic_name = "TestTopic", type_ = MyDataType)

    publisher = participant.create_publisher()
    data_writer = publisher.create_datawriter(topic)

    reader_listener = MyReaderListener(listener_semaphore)
    subscriber = participant.create_subscriber()
    data_reader = subscriber.create_datareader(topic, a_listener = reader_listener, mask=[dust_dds.StatusKind.DataAvailable] ) #

    # Wait for discovery
    ws = dust_dds.WaitSet()
    cond = data_writer.get_statuscondition()
    cond.set_enabled_statuses([dust_dds.StatusKind.PublicationMatched])
    ws.attach_condition(dust_dds.Condition.StatusCondition(cond))
    
    data = MyDataType(bytes([0,1,2,3,4]))
    data_writer.write(data)
    
    # Wait for data to be received on the listener using semaphore
    listener_semaphore.acquire(timeout=10)

    received_data: MyDataType = data_reader.read(max_samples = 1)

    assert data.data == received_data[0].get_data().data