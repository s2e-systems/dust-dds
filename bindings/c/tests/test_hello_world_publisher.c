#include <stdio.h>
#include <assert.h>
#include <unistd.h>
#include "../include/dust_dds.h"
#include "build/hello_world.h"

int main(void) {
    DustDdsDomainParticipantFactory* factory = (DustDdsDomainParticipantFactory*)dust_dds_domain_participant_factory_get_instance();
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

    // Create publisher
    DustDdsPublisher* publisher = dust_dds_domain_participant_create_publisher(
        participant,
        DUST_DDS_PUBLISHER_QOS_DEFAULT
    );
    assert(publisher != NULL);

    // Create data writer
    DustDdsDataWriter* writer = dust_dds_publisher_create_datawriter(
        publisher,
        topic,
        DUST_DDS_DATAWRITER_QOS_DEFAULT
    );
    assert(writer != NULL);

    printf("Publisher waiting for subscriber discovery...\n");
    sleep(5);

    struct HelloWorld sample;
    sample.msg = "Hello from C bindings!";
    sample.count = 42;

    printf("Writing HelloWorld sample...\n");
    ReturnCode result = dust_dds_datawriter_write_HelloWorld(writer, &sample);
    assert(result == RETCODE_OK);

    sleep(1);

    // Clean up
    result = dust_dds_publisher_delete_datawriter(publisher, writer);
    assert(result == RETCODE_OK);

    result = dust_dds_domain_participant_delete_publisher(participant, publisher);
    assert(result == RETCODE_OK);

    result = dust_dds_domain_participant_delete_topic(participant, topic);
    assert(result == RETCODE_OK);

    result = dust_dds_domain_participant_factory_delete_participant(factory, participant);
    assert(result == RETCODE_OK);

    printf("Publisher completed successfully.\n");
    return 0;
}
