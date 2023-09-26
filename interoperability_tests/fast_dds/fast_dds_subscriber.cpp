#include "HelloWorldPubSubTypes.h"
#include <fastdds/dds/domain/DomainParticipantFactory.hpp>
#include <fastdds/dds/domain/DomainParticipant.hpp>
#include <fastdds/dds/topic/TypeSupport.hpp>
#include <fastdds/dds/subscriber/Subscriber.hpp>
#include <fastdds/dds/subscriber/DataReader.hpp>
#include <fastdds/dds/core/condition/WaitSet.hpp>
#include <fastdds/dds/core/condition/StatusCondition.hpp>

#include <string>
#include <thread>
#include <chrono>
#include <stdexcept>

using namespace eprosima::fastdds::dds;

int main(int argc, char *argv[])
{
	const std::string topic_name = "HelloWorld";

	auto participant = DomainParticipantFactory::get_instance()->create_participant(0, PARTICIPANT_QOS_DEFAULT);
	TypeSupport hello_world_type{new HelloWorldTypePubSubType()};
	hello_world_type.register_type(participant);
	auto topic = participant->create_topic(topic_name, "HelloWorldType", TOPIC_QOS_DEFAULT);
	auto subscriber = participant->create_subscriber(SUBSCRIBER_QOS_DEFAULT, nullptr);

	DataReaderQos qos;
	qos.reliability().kind = RELIABLE_RELIABILITY_QOS;
	qos.durability().kind = TRANSIENT_LOCAL_DURABILITY_QOS;
	auto reader = subscriber->create_datareader(topic, qos);

	auto& reader_condition = reader->get_statuscondition();
	reader_condition.set_enabled_statuses(StatusMask::subscription_matched());
	WaitSet wait_set_publication_matched;
	wait_set_publication_matched.attach_condition(reader_condition);
	ConditionSeq active_conditions;
	const auto ret_wait_publication = wait_set_publication_matched.wait(active_conditions, eprosima::fastrtps::Duration_t{60, 0});
	if (ret_wait_publication != ReturnCode_t::RETCODE_OK)
	{
		throw std::runtime_error{"Publication not matched"};
	}

	reader_condition.set_enabled_statuses(StatusMask::data_available());
	WaitSet wait_set_data_available;
	wait_set_data_available.attach_condition(reader_condition);
	const auto ret_wait_data = wait_set_data_available.wait(active_conditions, eprosima::fastrtps::Duration_t{30, 0});
	if (ret_wait_data != ReturnCode_t::RETCODE_OK)
	{
		throw std::runtime_error{"No data available on time"};
	}

	HelloWorldType sample;
	SampleInfo info;
	if (reader->take_next_sample(&sample, &info) != ReturnCode_t::RETCODE_OK)
	{
		throw std::runtime_error{"take_next_sample failed with"};
	}
	if (!info.valid_data)
	{
		throw std::runtime_error{"data not valid"};
	}

	printf("Received: HelloWorldType { id: %d, msg: \"%c\" }\n", sample.id(), sample.msg());

	// Sleep to allow sending acknowledgements
	std::this_thread::sleep_for(std::chrono::seconds(2));
}
