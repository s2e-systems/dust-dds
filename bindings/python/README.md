# Python bindings for Dust DDS.

Dust DDS is a native [Rust](https://www.rust-lang.org/) implementation of the OMG [Data Distribution Services (DDS)](https://www.omg.org/omg-dds-portal/) and [Real-time Publisher-Subscriber (RTPS)](https://www.omg.org/spec/DDSI-RTPS/About-DDSI-RTPS/) developed by [S2E Software Systems](https://www.s2e-systems.com).

These bindings are a feature complete thin wrapper of the original Dust DDS implementation.

## A brief introduction to DDS

DDS is a middleware protocol and API standard designed for data-centric connectivity. At its core, DDS aims to facilitate the seamless sharing of pertinent data precisely where and when it's needed, even across publishers and subscribers operating asynchronously in time. With DDS, applications can exchange information through the reading and writing of data-objects identified by user-defined names (Topics) and keys. One of its defining features is the robust control it offers over Quality-of-Service (QoS) parameters, encompassing reliability, bandwidth, delivery deadlines, and resource allocations.

The [DDS standard](https://www.omg.org/spec/DDS/1.4/PDF) "defines both the Application Interfaces (APIs) and the Communication Semantics (behavior and quality of service) that enable the efficient delivery of information from information producers to matching consumer". Complementing this standard is the [DDSI-RTPS specification](https://www.omg.org/spec/DDSI-RTPS/2.5/PDF), which defines an interoperability wire protocol for DDS. Its primary aim is to ensure that applications based on different vendors' implementations of DDS can interoperate. The implementation of Dust DDS primarily centers around the DDS and DDSI-RTPS standards.

## Example

A basic example on how to use Dust DDS:

```python
from dataclasses import dataclass
import dust_dds

@dataclass
class MyDataType:
    data: bytes

class MyReaderListener:
    def on_data_available(reader):
        received_data = reader.read(max_samples = 1)
        print(f"On data available, data: {received_data[0].get_data()}")

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
print(f"Received data {received_data[0].get_data()}")

```

## DDS REST API

If you want to interact with your DDS data using a REST API you can use our [Nebula DDS WebLink](https://www.s2e-systems.com/products/nebula-dds-weblink/) software. Nebula DDS WebLink provides a server implementing the Object Management Group (OMG) Web-Enabled DDS v1.0 standard.

## Shapes demo

DDS interoperability is typically tested using a shapes demo. The Dust DDS Shapes Demo is available on our repository.

![Dust DDS Shapes demo screenshot](https://raw.githubusercontent.com/s2e-systems/dust-dds/master/dds/docs/shapes_demo_screenshot.png)

## Release schedule

Dust DDS doesn't follow a fixed release schedule but we will make releases as new features are implemented.

## License

This project is licensed under the Apache License Version 2.0.
