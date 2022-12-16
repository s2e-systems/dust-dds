#include "ddsc/dds.h"
#include "HelloWorld.h"

int main(int argc, char *argv[])
{
	const char *topic_name = "HelloWorld";

	const dds_entity_t participant = dds_create_participant(DDS_DOMAIN_DEFAULT, NULL /*qos*/, NULL /*listener*/);
	if (participant < 0)
	{
		DDS_FATAL("dds_create_participant: %s\n", dds_strretcode(-participant));
	}
	const dds_entity_t topic = dds_create_topic(participant, &HelloWorldType_desc, topic_name, NULL /*qos*/, NULL /*listener*/);
	if (topic < 0)
	{
		DDS_FATAL("dds_create_topic: %s\n", dds_strretcode(-topic));
	}
	dds_qos_t *qos = dds_create_qos();
	dds_qset_reliability(qos, DDS_RELIABILITY_RELIABLE, DDS_SECS(1));
	dds_qset_durability(qos, DDS_DURABILITY_TRANSIENT_LOCAL);

	const dds_entity_t data_writer = dds_create_writer(participant, topic, qos, NULL /*listener*/);
	if (data_writer < 0)
	{
		DDS_FATAL("dds_create_writer: %s\n", dds_strretcode(-data_writer));
	}

	dds_return_t rc;

	rc = dds_set_status_mask(data_writer, DDS_PUBLICATION_MATCHED_STATUS);
	if (rc != DDS_RETCODE_OK)
	{
		DDS_FATAL("dds_set_status_mask: %s\n", dds_strretcode(-rc));
	}

	dds_entity_t waitset = dds_create_waitset(participant);

	rc = dds_waitset_attach(waitset, data_writer, data_writer);
	if (rc != DDS_RETCODE_OK)
	{
		DDS_FATAL("dds_waitset_attach: %s\n", dds_strretcode(-rc));
	}

	dds_attach_t wsresults[1];
	const size_t wsresultsize = 1U;
	rc = dds_waitset_wait(waitset, wsresults, wsresultsize, DDS_SECS(60));
	if (rc == 0) {
		DDS_FATAL("dds_waitset_wait: timeout");
	}
	if (rc != wsresultsize)
	{
		DDS_FATAL("dds_waitset_wait: %s\n", dds_strretcode(-rc));
	}


	HelloWorldType msg = {8, "Hello world!"};
	dds_write(data_writer, &msg);

	rc = dds_wait_for_acks(data_writer, DDS_SECS(30));
	if (rc != DDS_RETCODE_OK) {
		DDS_FATAL("dds_wait_for_acks: %s\n", dds_strretcode(-rc));
	}
}
