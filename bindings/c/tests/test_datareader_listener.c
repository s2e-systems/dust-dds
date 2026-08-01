#include <stdio.h>
#include <assert.h>
#include <unistd.h>
#include "../include/dust_dds.h"
#include "build/hello_world.h"

#define dds_datawriter_write_HelloWorld dust_dds_datawriter_write_HelloWorld

struct MyListenerData {
    int counter;
};

static void on_data_available(DustDdsDataReader* reader, void* listener_data) {
    printf("C on_data_available callback triggered!\n");
    struct MyListenerData* data = (struct MyListenerData*)listener_data;
    data->counter++;
}

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

    // Create subscriber
    DustDdsSubscriber* subscriber = dds_domain_participant_create_subscriber(
        participant,
        DUST_DDS_SUBSCRIBER_QOS_DEFAULT
    );
    assert(subscriber != NULL);

    // Set up DataReaderListener
    struct MyListenerData my_data = { 0 };
    DustDdsDataReaderListener listener = {
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
    DustDdsDataReader* reader = dds_subscriber_create_datareader(
        subscriber,
        topic,
        DUST_DDS_DATAREADER_QOS_DEFAULT,
        &listener,
        DUST_DDS_STATUS_DATA_AVAILABLE_STATUS
    );
    assert(reader != NULL);

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

    // Wait for match / discovery
    printf("Waiting for discovery...\n");
    sleep(1);

    struct HelloWorld sample;
    sample.msg = "Hello Listener!";
    sample.count = 1;
    ReturnCode result = dds_datawriter_write_HelloWorld(writer, &sample);
    assert(result == RETCODE_OK);

    // Wait for listener to trigger
    printf("Waiting for listener to trigger...\n");
    for (int i = 0; i < 50; i++) {
        if (my_data.counter > 0) {
            break;
        }
        usleep(100000); // 100ms
    }

    assert(my_data.counter > 0);
    printf("Callback successfully triggered! Counter: %d\n", my_data.counter);

    // Clean up
    result = dds_publisher_delete_datawriter(publisher, writer);
    assert(result == RETCODE_OK);

    result = dds_domain_participant_delete_publisher(participant, publisher);
    assert(result == RETCODE_OK);

    result = dds_subscriber_delete_datareader(subscriber, reader);
    assert(result == RETCODE_OK);

    result = dds_domain_participant_delete_subscriber(participant, subscriber);
    assert(result == RETCODE_OK);

    result = dds_domain_participant_delete_topic(participant, topic);
    assert(result == RETCODE_OK);

    result = dds_domain_participant_factory_delete_participant(factory, participant);
    assert(result == RETCODE_OK);

    printf("Listener test successfully completed execution.\n");
    return 0;
}
