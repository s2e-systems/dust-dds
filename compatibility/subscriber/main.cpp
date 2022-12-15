#include <stdexcept>
#include <string>

#include "dds/dds.hpp"
#include "HelloWorld.hpp"

using namespace org::eclipse::cyclonedds;

int main(int argc, char *argv[])
{
	try
	{
		const std::string topicName = "HelloWorld";

		dds::domain::DomainParticipant dp{domain::default_id()};
		const dds::topic::qos::TopicQos topicQos{dp.default_topic_qos()};
		const dds::topic::Topic<HelloWorldType> topic{dp, topicName, topicQos};
		const dds::sub::qos::SubscriberQos subscriberQos{dp.default_subscriber_qos()};
		const dds::sub::Subscriber subscriber{dp, subscriberQos};
		dds::sub::qos::DataReaderQos dataReaderQos{topic.qos()};
		dataReaderQos << dds::core::policy::Reliability::Reliable();

		dds::sub::DataReader<HelloWorldType> dataReader{subscriber, topic, dataReaderQos};

		while (true)
		{
			const auto samples = dataReader.take();
			for (const auto &sample : samples)
			{
				if (sample.info().valid())
				{
					std::cout << "Received sample with id: " << int(sample.data().id()) << " and msg: " << sample.data().msg() << std::endl;
				}
			}
			std::this_thread::sleep_for(std::chrono::milliseconds(10));
		}
	}
	catch (const dds::core::Error &e)
	{
		std::cerr << "DDS Error: " << e.what() << std::endl;
	}
	catch (...)
	{
		std::cerr << "DDS initialization failed with unhandled exception" << std::endl;
	}
}
