#include "DisposeDataPubSubTypes.h"
#include <fastdds/dds/domain/DomainParticipantFactory.hpp>
#include <fastdds/dds/domain/DomainParticipant.hpp>
#include <fastdds/dds/topic/TypeSupport.hpp>
#include <fastdds/dds/publisher/Publisher.hpp>
#include <fastdds/dds/publisher/DataWriter.hpp>
#include <fastdds/dds/core/condition/WaitSet.hpp>
#include <fastdds/dds/core/condition/StatusCondition.hpp>
#include <fastdds/dds/core/condition/StatusCondition.hpp>

#include <string>
#include <thread>
#include <stdexcept>

using namespace eprosima::fastdds::dds;

int main(int argc, char *argv[])
{
	const std::string topic_name = "DisposeData";

	auto participant = DomainParticipantFactory::get_instance()->create_participant(0, PARTICIPANT_QOS_DEFAULT);
	TypeSupport dispose_data_type{new interoperability::test::DisposeDataTypePubSubType()};
	dispose_data_type.register_type(participant);
	auto topic = participant->create_topic(topic_name, dispose_data_type.get_type_name(), TOPIC_QOS_DEFAULT);
	auto publisher = participant->create_publisher(PUBLISHER_QOS_DEFAULT, nullptr);

	DataWriterQos qos;
	qos.durability().kind = TRANSIENT_LOCAL_DURABILITY_QOS;
	auto writer = publisher->create_datawriter(topic, qos);

	auto &writer_condition = writer->get_statuscondition();
	writer_condition.set_enabled_statuses(StatusMask::publication_matched());
	WaitSet wait_set;
	wait_set.attach_condition(writer_condition);
	ConditionSeq active_conditions;
	auto ret_wait = wait_set.wait(active_conditions, eprosima::fastrtps::Duration_t{60, 0});
	if (ret_wait != ReturnCode_t::RETCODE_OK)
	{
		throw std::runtime_error{"Subscription not matched"};
	}

	interoperability::test::DisposeDataType dispose_msg;
	dispose_msg.name("Very Long Name");
	dispose_msg.value(1);
	auto handle = writer->register_instance(&dispose_msg);

	writer->write(&dispose_msg);
	auto ret_ack = writer->wait_for_acknowledgments(eprosima::fastrtps::Duration_t{30, 0});
	if (ret_ack != ReturnCode_t::RETCODE_OK)
	{
		throw std::runtime_error{"Acknowledgements for write did not arrive in time"};
	}

	writer->dispose(&dispose_msg, handle);
	ret_ack = writer->wait_for_acknowledgments(eprosima::fastrtps::Duration_t{30, 0});
	if (ret_ack != ReturnCode_t::RETCODE_OK)
	{
		throw std::runtime_error{"Acknowledgements for dispose did not arrive in time"};
	}
}
