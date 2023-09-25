#include "HelloWorldPubSubTypes.h"
#include <fastdds/dds/domain/DomainParticipantFactory.hpp>
#include <fastdds/dds/domain/DomainParticipant.hpp>
#include <fastdds/dds/topic/TypeSupport.hpp>
#include <fastdds/dds/subscriber/Subscriber.hpp>
#include <fastdds/dds/publisher/Publisher.hpp>
#include <fastdds/dds/subscriber/DataReader.hpp>
#include <fastdds/dds/publisher/DataWriter.hpp>

#include <string>
#include <thread>

using namespace eprosima::fastdds::dds;

int main(int argc, char *argv[])
{
	const std::string topic_name = "HelloWorld";

	auto participant = DomainParticipantFactory::get_instance()->create_participant(0, PARTICIPANT_QOS_DEFAULT);
	TypeSupport hello_world_type{new HelloWorldTypePubSubType()};
	hello_world_type.register_type(participant);
	auto topic = participant->create_topic(topic_name, "HelloWorldType", TOPIC_QOS_DEFAULT);
	auto subscriber = participant->create_subscriber(SUBSCRIBER_QOS_DEFAULT, nullptr);
	auto reader = subscriber->create_datareader(topic, DATAREADER_QOS_DEFAULT);

	auto publisher = participant->create_publisher(PUBLISHER_QOS_DEFAULT, nullptr);
	auto writer = publisher->create_datawriter(topic, DATAWRITER_QOS_DEFAULT);

	HelloWorldType hello;
	hello.id(3);
	hello.msg('h');
	writer->write(&hello);

	std::this_thread::sleep_for(std::chrono::milliseconds(1000));

	HelloWorldType sample;
	SampleInfo info;
	if (reader->take_next_sample(&sample, &info) == ReturnCode_t::RETCODE_OK)
	{
	    if (info.valid_data)
	    {
			printf("Received: HelloWorldType { id: %d, msg: \"%c\" }\n", sample.id(), sample.msg());
	    }
	}
}
