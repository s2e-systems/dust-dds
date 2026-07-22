#include <stdio.h>
#include <assert.h>
#include <unistd.h>
#include "../include/dust_dds.h"
#include "build/hello_world.h"

int main(void) {
    DustDdsDomainParticipantFactory* factory = dds_domain_participant_factory_get_instance();
    assert(factory != NULL);

    DustDdsDomainParticipant* participant = dds_domain_participant_factory_create_participant(
        factory,
        0,
        DUST_DDS_PARTICIPANT_QOS_DEFAULT
    );
    assert(participant != NULL);

    // Create topic
    DustDdsTopic* topic = dds_domain_participant_create_topic(
        participant,
        "HelloWorldTopic",
        "HelloWorld",
        DUST_DDS_TOPIC_QOS_DEFAULT,
        (DustDdsDynamicType*)HelloWorld_get_type()
    );
    assert(topic != NULL);

    // Create publisher
    DustDdsPublisher* publisher = dds_domain_participant_create_publisher(
        participant,
        DUST_DDS_PUBLISHER_QOS_DEFAULT
    );
    assert(publisher != NULL);

    // Create data writer
    DustDdsDataWriter* writer = dds_publisher_create_datawriter(
        publisher,
        topic,
        DUST_DDS_DATAWRITER_QOS_DEFAULT
    );
    assert(writer != NULL);

    DustDdsStatusCondition* writer_cond = dds_datawriter_get_statuscondition(writer);
    assert(writer_cond != NULL);
    ReturnCode result = dds_status_condition_set_enabled_statuses(writer_cond, DUST_DDS_STATUS_PUBLICATION_MATCHED_STATUS);
    assert(result == RETCODE_OK);

    DustDdsWaitSet* wait_set = dds_wait_set_new();
    assert(wait_set != NULL);
    result = dds_wait_set_attach_condition(wait_set, writer_cond);
    assert(result == RETCODE_OK);

    printf("Publisher waiting for subscriber discovery...\n");
    DustDdsDuration wait_timeout = { 60, 0 };
    result = dds_wait_set_wait(wait_set, wait_timeout);
    assert(result == RETCODE_OK);

    struct HelloWorld sample;
    sample.msg = "Hello from C bindings!";
    sample.count = 42;

    printf("Writing HelloWorld sample...\n");
    result = dds_datawriter_write_HelloWorld(writer, &sample);
    assert(result == RETCODE_OK);

    DustDdsDuration ack_timeout = { 30, 0 };
    result = dds_datawriter_wait_for_acknowledgments(writer, ack_timeout);
    assert(result == RETCODE_OK);

    sleep(2);

    // Clean up
    result = dds_wait_set_free(wait_set);
    assert(result == RETCODE_OK);

    result = dds_status_condition_free(writer_cond);
    assert(result == RETCODE_OK);

    result = dds_publisher_delete_datawriter(publisher, writer);
    assert(result == RETCODE_OK);

    result = dds_domain_participant_delete_publisher(participant, publisher);
    assert(result == RETCODE_OK);

    result = dds_domain_participant_delete_topic(participant, topic);
    assert(result == RETCODE_OK);

    result = dds_domain_participant_factory_delete_participant(factory, participant);
    assert(result == RETCODE_OK);

    printf("Publisher completed successfully.\n");
    return 0;
}
