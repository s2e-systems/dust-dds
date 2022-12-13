#include <stdexcept>
#include <string>
#include <chrono>
#include <thread>

#include "dds/dds.hpp"
#include "HelloWorld.hpp"

using namespace org::eclipse::cyclonedds;

int main(int argc, char *argv[])
{
	try
	{
		const std::string topicName = "HelloWorld";

		const dds::domain::DomainParticipant dp{domain::default_id()};
		const dds::topic::qos::TopicQos topicQos{dp.default_topic_qos()};
		const dds::topic::Topic<HelloWorldType> topic{dp, topicName, topicQos};
		const dds::pub::qos::PublisherQos publisherQos{dp.default_publisher_qos()};
		const dds::pub::Publisher publisher{dp, publisherQos};
		dds::pub::qos::DataWriterQos dataWriterQos{topic.qos()};
		dataWriterQos << dds::core::policy::Reliability::Reliable();
		dds::pub::DataWriter<HelloWorldType> dataWriter{publisher, topic, dataWriterQos};

		for (uint8_t id = 0;; id++)
		{
			dataWriter << HelloWorldType{id, "Hello world"};
			std::cout << "Published sample with id: " << int(id) << std::endl;
			std::this_thread::sleep_for(std::chrono::milliseconds(100));
		}
	}
	catch (const dds::core::Error &e)
	{
		std::cerr << "DDS Error: " << e.what() << std::endl;
	}
	catch (...)
	{
		std::cerr << "Initialization failed with unhandled exception" << std::endl;
	}
}
