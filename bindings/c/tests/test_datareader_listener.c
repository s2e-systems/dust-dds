#include "unity.h"
#include <stdio.h>
#include <unistd.h>
#include "../include/dust_dds.h"
#include "hello_world.h"

struct MyListenerData {
    int counter;
};

static void on_data_available(DDS_DataReader* reader, void* listener_data) {
    printf("C on_data_available callback triggered!\n");
    struct MyListenerData* data = (struct MyListenerData*)listener_data;
    data->counter++;
}

void setUp(void) {}
void tearDown(void) {}

void test_datareader_listener_callback(void) {
    DDS_DomainParticipantFactory* factory = (DDS_DomainParticipantFactory*)DDS_DomainParticipantFactory_get_instance();
    TEST_ASSERT_NOT_NULL(factory);

    DDS_DomainParticipant* participant = DDS_DomainParticipantFactory_create_participant(
        factory,
        0,
        DUST_DDS_PARTICIPANT_QOS_DEFAULT,
        NULL,
        0
    );
    TEST_ASSERT_NOT_NULL(participant);

    // Create topic
    DDS_Topic* topic = DDS_DomainParticipant_create_topic(
        participant,
        "HelloWorldTopic",
        "HelloWorld",
        DUST_DDS_TOPIC_QOS_DEFAULT,
        NULL,
        0,
        (DDS_DynamicType*)HelloWorld_get_type()
    );
    TEST_ASSERT_NOT_NULL(topic);

    // Create subscriber
    DDS_Subscriber* subscriber = DDS_DomainParticipant_create_subscriber(
        participant,
        DUST_DDS_SUBSCRIBER_QOS_DEFAULT,
        NULL,
        0
    );
    TEST_ASSERT_NOT_NULL(subscriber);

    // Set up DataReaderListener
    struct MyListenerData my_data = { 0 };
    DDS_DataReaderListener listener = {
        .listener_data = &my_data,
        .on_data_available = on_data_available,
        .on_sample_rejected = NULL,
        .on_liveliness_changed = NULL,
        .on_requested_deadline_missed = NULL,
        .on_requested_incompatible_qos = NULL,
        .on_subscription_matched = NULL,
        .on_sample_lost = NULL
    };

    // Create data reader with the listener
    DDS_DataReader* reader = DDS_Subscriber_create_datareader(
        subscriber,
        topic,
        DUST_DDS_DATAREADER_QOS_DEFAULT,
        &listener,
        DDS_DUST_DDS_STATUS_DATA_AVAILABLE_STATUS
    );
    TEST_ASSERT_NOT_NULL(reader);

    // Create publisher
    DDS_Publisher* publisher = DDS_DomainParticipant_create_publisher(
        participant,
        DUST_DDS_PUBLISHER_QOS_DEFAULT,
        NULL,
        0
    );
    TEST_ASSERT_NOT_NULL(publisher);

    // Create data writer
    DDS_DataWriter* writer = DDS_Publisher_create_datawriter(
        publisher,
        topic,
        DUST_DDS_DATAWRITER_QOS_DEFAULT,
        NULL,
        0
    );
    TEST_ASSERT_NOT_NULL(writer);

    // Wait for match / discovery
    printf("Waiting for discovery...\n");
    sleep(1);

    struct HelloWorld sample;
    sample.msg = "Hello Listener!";
    sample.count = 1;
    DDS_ReturnCode result = HelloWorldDataWriter_write(writer, &sample, NULL);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    // Wait for listener to trigger
    printf("Waiting for listener to trigger...\n");
    for (int i = 0; i < 50; i++) {
        if (my_data.counter > 0) {
            break;
        }
        usleep(100000); // 100ms
    }

    TEST_ASSERT_TRUE(my_data.counter > 0);
    printf("Callback successfully triggered! Counter: %d\n", my_data.counter);

    // Clean up
    result = DDS_Publisher_delete_datawriter(publisher, writer);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    result = DDS_DomainParticipant_delete_publisher(participant, publisher);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    result = DDS_Subscriber_delete_datareader(subscriber, reader);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    result = DDS_DomainParticipant_delete_subscriber(participant, subscriber);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    result = DDS_DomainParticipant_delete_topic(participant, topic);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    result = DDS_DomainParticipantFactory_delete_participant(factory, participant);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);
}

int main(void) {
    UNITY_BEGIN();
    RUN_TEST(test_datareader_listener_callback);
    return UNITY_END();
}
