#include "unity.h"
#include <stdio.h>
#include <unistd.h>
#include "../include/dust_dds.h"
#include "hello_world.h"

void setUp(void) {}
void tearDown(void) {}

void test_hello_world_write_read(void) {
    DDS_DomainParticipantFactory* factory = (DDS_DomainParticipantFactory*)DDS_domain_participant_factory_get_instance();
    TEST_ASSERT_NOT_NULL(factory);

    // Create Publisher Participant
    DDS_DomainParticipant* participant_pub = DDS_domain_participant_factory_create_participant(
        factory,
        0,
        DUST_DDS_PARTICIPANT_QOS_DEFAULT,
        NULL,
        0
    );
    TEST_ASSERT_NOT_NULL(participant_pub);

    // Create Subscriber Participant
    DDS_DomainParticipant* participant_sub = DDS_domain_participant_factory_create_participant(
        factory,
        0,
        DUST_DDS_PARTICIPANT_QOS_DEFAULT,
        NULL,
        0
    );
    TEST_ASSERT_NOT_NULL(participant_sub);

    // Create Topic for Publisher
    DDS_Topic* topic_pub = DDS_domain_participant_create_topic(
        participant_pub,
        "HelloWorldTopic",
        "HelloWorld",
        DUST_DDS_TOPIC_QOS_DEFAULT,
        NULL,
        0,
        (DDS_DynamicType*)HelloWorld_get_type()
    );
    TEST_ASSERT_NOT_NULL(topic_pub);

    // Create Topic for Subscriber
    DDS_Topic* topic_sub = DDS_domain_participant_create_topic(
        participant_sub,
        "HelloWorldTopic",
        "HelloWorld",
        DUST_DDS_TOPIC_QOS_DEFAULT,
        NULL,
        0,
        (DDS_DynamicType*)HelloWorld_get_type()
    );
    TEST_ASSERT_NOT_NULL(topic_sub);

    // Create Publisher and Writer
    DDS_Publisher* publisher = DDS_domain_participant_create_publisher(
        participant_pub,
        DUST_DDS_PUBLISHER_QOS_DEFAULT,
        NULL,
        0
    );
    TEST_ASSERT_NOT_NULL(publisher);

    DDS_DataWriter* writer = DDS_publisher_create_datawriter(
        publisher,
        topic_pub,
        DUST_DDS_DATAWRITER_QOS_DEFAULT,
        NULL,
        0
    );
    TEST_ASSERT_NOT_NULL(writer);

    // Create Subscriber and Reader
    DDS_Subscriber* subscriber = DDS_domain_participant_create_subscriber(
        participant_sub,
        DUST_DDS_SUBSCRIBER_QOS_DEFAULT,
        NULL,
        0
    );
    TEST_ASSERT_NOT_NULL(subscriber);

    DDS_DataReader* reader = DDS_subscriber_create_datareader(
        subscriber,
        topic_sub,
        DUST_DDS_DATAREADER_QOS_DEFAULT,
        NULL,
        0
    );
    TEST_ASSERT_NOT_NULL(reader);

    // Set up Status Conditions and Wait Sets for Discovery
    DDS_StatusCondition* writer_cond = DDS_datawriter_get_statuscondition(writer);
    TEST_ASSERT_NOT_NULL(writer_cond);
    DDS_ReturnCode result = DDS_status_condition_set_enabled_statuses(writer_cond, DDS_DUST_DDS_STATUS_PUBLICATION_MATCHED_STATUS);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    DDS_WaitSet* wait_set_pub = DDS_wait_set_new();
    TEST_ASSERT_NOT_NULL(wait_set_pub);
    result = DDS_wait_set_attach_condition(wait_set_pub, writer_cond);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    DDS_StatusCondition* reader_cond = DDS_datareader_get_statuscondition(reader);
    TEST_ASSERT_NOT_NULL(reader_cond);
    result = DDS_status_condition_set_enabled_statuses(reader_cond, DDS_DUST_DDS_STATUS_SUBSCRIPTION_MATCHED_STATUS);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    DDS_WaitSet* wait_set_sub = DDS_wait_set_new();
    TEST_ASSERT_NOT_NULL(wait_set_sub);
    result = DDS_wait_set_attach_condition(wait_set_sub, reader_cond);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    // Wait for Discovery (matching) on both ends
    printf("Waiting for discovery...\n");
    DDS_Duration wait_timeout = { 10, 0 }; // 10 seconds timeout
    result = DDS_wait_set_wait(wait_set_pub, wait_timeout);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    result = DDS_wait_set_wait(wait_set_sub, wait_timeout);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    // Write a sample
    struct HelloWorld sample;
    sample.msg = "Hello from C bindings!";
    sample.count = 42;
    printf("Writing HelloWorld sample...\n");

    // Test register_instance and lookup_instance
    DDS_InstanceHandle_t handle;
    result = HelloWorld_dds_datawriter_register_instance(writer, &sample, &handle);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    DDS_InstanceHandle_t lookup_handle;
    result = HelloWorld_dds_datawriter_lookup_instance(writer, &sample, &lookup_handle);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);
    TEST_ASSERT_EQUAL_MEMORY(handle, lookup_handle, sizeof(DDS_InstanceHandle_t));

    result = HelloWorld_dds_datawriter_write(writer, &sample, &handle);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    // Wait for data to be available on reader
    result = DDS_status_condition_set_enabled_statuses(reader_cond, DDS_DUST_DDS_STATUS_DATA_AVAILABLE_STATUS);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    printf("Waiting for data...\n");
    result = DDS_wait_set_wait(wait_set_sub, wait_timeout);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    // Read the sample
    struct HelloWorld data_values[1];
    int32_t received_samples = 0;
    result = HelloWorld_dds_datareader_read(reader, data_values, NULL, 1, DDS_ANY_SAMPLE_STATE, DDS_ANY_VIEW_STATE, DDS_ANY_INSTANCE_STATE, &received_samples);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);
    TEST_ASSERT_EQUAL_INT(1, received_samples);
    TEST_ASSERT_EQUAL_STRING("Hello from C bindings!", data_values[0].msg);
    TEST_ASSERT_EQUAL_INT(42, data_values[0].count);

    // Free the string allocated by the Rust bindings
    HelloWorld_free_sample(&data_values[0]);

    // Test unregister_instance now that reading is done
    result = HelloWorld_dds_datawriter_unregister_instance(writer, &sample, &handle);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);


    // Wait for acknowledgment on writer
    DDS_Duration ack_timeout = { 10, 0 };
    result = DDS_datawriter_wait_for_acknowledgments(writer, ack_timeout);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    // Clean up
    result = DDS_wait_set_free(wait_set_pub);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    result = DDS_wait_set_free(wait_set_sub);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    result = DDS_status_condition_free(writer_cond);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    result = DDS_status_condition_free(reader_cond);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    result = DDS_publisher_delete_datawriter(publisher, writer);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    result = DDS_domain_participant_delete_publisher(participant_pub, publisher);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    result = DDS_subscriber_delete_datareader(subscriber, reader);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    result = DDS_domain_participant_delete_subscriber(participant_sub, subscriber);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    result = DDS_domain_participant_delete_topic(participant_pub, topic_pub);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    result = DDS_domain_participant_delete_topic(participant_sub, topic_sub);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    result = DDS_domain_participant_factory_delete_participant(factory, participant_pub);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    result = DDS_domain_participant_factory_delete_participant(factory, participant_sub);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);
}

int main(void) {
    UNITY_BEGIN();
    RUN_TEST(test_hello_world_write_read);
    return UNITY_END();
}
