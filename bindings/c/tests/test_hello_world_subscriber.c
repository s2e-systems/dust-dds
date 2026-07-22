#include <stdio.h>
#include <assert.h>
#include <unistd.h>
#include "../include/dust_dds.h"
#include "build/hello_world.h"

int main(void) {
    DustDdsDomainParticipantFactory* factory = dust_dds_domain_participant_factory_get_instance();
    assert(factory != NULL);

    DustDdsDomainParticipant* participant = dust_dds_domain_participant_factory_create_participant(
        factory,
        0,
        DUST_DDS_PARTICIPANT_QOS_DEFAULT
    );
    assert(participant != NULL);

    // Create topic
    DustDdsTopic* topic = dust_dds_domain_participant_create_topic(
        participant,
        "HelloWorldTopic",
        "HelloWorld",
        DUST_DDS_TOPIC_QOS_DEFAULT,
        (DustDdsDynamicType*)HelloWorld_get_type()
    );
    assert(topic != NULL);

    // Create subscriber
    DustDdsSubscriber* subscriber = dust_dds_domain_participant_create_subscriber(
        participant,
        DUST_DDS_SUBSCRIBER_QOS_DEFAULT
    );
    assert(subscriber != NULL);

    // Create data reader
    DustDdsDataReader* reader = dust_dds_subscriber_create_datareader(
        subscriber,
        topic,
        DUST_DDS_DATAREADER_QOS_DEFAULT
    );
    assert(reader != NULL);

    DustDdsStatusCondition* reader_cond = dust_dds_datareader_get_statuscondition(reader);
    assert(reader_cond != NULL);
    ReturnCode result = dust_dds_status_condition_set_enabled_statuses(reader_cond, DUST_DDS_STATUS_SUBSCRIPTION_MATCHED_STATUS);
    assert(result == RETCODE_OK);

    DustDdsWaitSet* wait_set = dust_dds_wait_set_new();
    assert(wait_set != NULL);
    result = dust_dds_wait_set_attach_condition(wait_set, reader_cond);
    assert(result == RETCODE_OK);

    printf("Subscriber waiting for discovery...\n");
    DustDdsDuration wait_timeout1 = { 60, 0 };
    result = dust_dds_wait_set_wait(wait_set, wait_timeout1);
    assert(result == RETCODE_OK);

    result = dust_dds_status_condition_set_enabled_statuses(reader_cond, DUST_DDS_STATUS_DATA_AVAILABLE_STATUS);
    assert(result == RETCODE_OK);

    printf("Subscriber waiting for data...\n");
    DustDdsDuration wait_timeout2 = { 30, 0 };
    result = dust_dds_wait_set_wait(wait_set, wait_timeout2);
    assert(result == RETCODE_OK);

    struct HelloWorld samples[1];
    int32_t received = 0;
    result = dust_dds_datareader_read_HelloWorld(reader, samples, 1, &received);
    assert(result == RETCODE_OK && received > 0);
    printf("Received message: %s (count: %u)\n", samples[0].msg, samples[0].count);
    HelloWorld_free_sample(&samples[0]);

    // Clean up
    result = dust_dds_wait_set_free(wait_set);
    assert(result == RETCODE_OK);

    result = dust_dds_status_condition_free(reader_cond);
    assert(result == RETCODE_OK);

    result = dust_dds_subscriber_delete_datareader(subscriber, reader);
    assert(result == RETCODE_OK);

    result = dust_dds_domain_participant_delete_subscriber(participant, subscriber);
    assert(result == RETCODE_OK);

    result = dust_dds_domain_participant_delete_topic(participant, topic);
    assert(result == RETCODE_OK);

    result = dust_dds_domain_participant_factory_delete_participant(factory, participant);
    assert(result == RETCODE_OK);

    printf("Subscriber completed successfully.\n");
    return 0;
}
