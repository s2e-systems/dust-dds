#include <stdio.h>
#include <assert.h>
#include <string.h>
#include <stdlib.h>
#include "../include/dust_dds.h"

int main(void) {
    DustDdsDomainParticipantFactory* factory = (DustDdsDomainParticipantFactory*)dds_domain_participant_factory_get_instance();
    assert(factory != NULL);

    // Test creating participant with NULL QoS (default QoS)
    DustDdsDomainParticipant* participant = dds_domain_participant_factory_create_participant(
        factory,
        0,
        NULL
    );
    assert(participant != NULL);

    ReturnCode result = dds_domain_participant_factory_delete_participant(
        factory,
        participant
    );
    assert(result == RETCODE_OK);

    // Test creating participant with specific DomainParticipantQos object and custom UserDataQosPolicy
    DomainParticipantQos qos = dds_domain_participant_qos_default();

    const uint8_t custom_user_data[] = {0x01, 0x02, 0x03, 0x04, 0x05};
    qos.user_data.value.length = sizeof(custom_user_data);
    qos.user_data.value.buffer = malloc(sizeof(custom_user_data));
    memcpy(qos.user_data.value.buffer, custom_user_data, sizeof(custom_user_data));

    // Verify user data matches
    assert(qos.user_data.value.length == sizeof(custom_user_data));
    assert(memcmp(qos.user_data.value.buffer, custom_user_data, sizeof(custom_user_data)) == 0);

    participant = dds_domain_participant_factory_create_participant(
        factory,
        0,
        &qos
    );
    assert(participant != NULL);

    // Clean up our allocated buffer (the Rust function will make a copy internal to DDS)
    free(qos.user_data.value.buffer);
    qos.user_data.value.buffer = NULL;
    qos.user_data.value.length = 0;

    // Test creating publisher and subscriber on participant with default QoS and NULL
    DustDdsPublisher* publisher = dds_domain_participant_create_publisher(participant, NULL);
    assert(publisher != NULL);
    result = dds_domain_participant_delete_publisher(participant, publisher);
    assert(result == RETCODE_OK);

    PublisherQos pub_qos = dds_publisher_qos_default();
    publisher = dds_domain_participant_create_publisher(participant, &pub_qos);
    assert(publisher != NULL);
    result = dds_domain_participant_delete_publisher(participant, publisher);
    assert(result == RETCODE_OK);

    DustDdsSubscriber* subscriber = dds_domain_participant_create_subscriber(participant, NULL);
    assert(subscriber != NULL);
    result = dds_domain_participant_delete_subscriber(participant, subscriber);
    assert(result == RETCODE_OK);

    SubscriberQos sub_qos = dds_subscriber_qos_default();
    subscriber = dds_domain_participant_create_subscriber(participant, &sub_qos);
    assert(subscriber != NULL);
    result = dds_domain_participant_delete_subscriber(participant, subscriber);
    assert(result == RETCODE_OK);

    result = dds_domain_participant_factory_delete_participant(
        factory,
        participant
    );
    assert(result == RETCODE_OK);

    // Test lookup_participant
    participant = dds_domain_participant_factory_create_participant(
        factory,
        1, // domain_id = 1
        NULL
    );
    assert(participant != NULL);

    DustDdsDomainParticipant* looked_up = dds_domain_participant_factory_lookup_participant(factory, 1);
    assert(looked_up != NULL);

    result = dds_domain_participant_factory_delete_participant(factory, looked_up);
    assert(result == RETCODE_OK);

    // After deleting, lookup should return NULL
    looked_up = dds_domain_participant_factory_lookup_participant(factory, 1);
    assert(looked_up == NULL);

    result = dds_domain_participant_factory_delete_participant(factory, participant);
    assert(result == RETCODE_ALREADY_DELETED);

    // Test get/set default participant QoS
    DomainParticipantQos default_part_qos;
    result = dds_domain_participant_factory_get_default_participant_qos(factory, &default_part_qos);
    assert(result == RETCODE_OK);

    // Verify entity factory default
    assert(default_part_qos.entity_factory.autoenable_created_entities == true);

    default_part_qos.entity_factory.autoenable_created_entities = false;

    // Set as default participant QoS
    result = dds_domain_participant_factory_set_default_participant_qos(factory, &default_part_qos);
    assert(result == RETCODE_OK);

    // Verify default participant QoS was updated
    DomainParticipantQos default_part_qos2;
    result = dds_domain_participant_factory_get_default_participant_qos(factory, &default_part_qos2);
    assert(result == RETCODE_OK);
    assert(default_part_qos2.entity_factory.autoenable_created_entities == false);

    // Restore to standard default QoS
    result = dds_domain_participant_factory_set_default_participant_qos(factory, NULL);
    assert(result == RETCODE_OK);

    // Test factory QoS
    DomainParticipantFactoryQos factory_qos;
    result = dds_domain_participant_factory_get_qos(factory, &factory_qos);
    assert(result == RETCODE_OK);
    assert(factory_qos.entity_factory.autoenable_created_entities == true);

    factory_qos.entity_factory.autoenable_created_entities = false;
    result = dds_domain_participant_factory_set_qos(factory, &factory_qos);
    assert(result == RETCODE_OK);

    // Verify factory QoS updated
    DomainParticipantFactoryQos factory_qos2;
    result = dds_domain_participant_factory_get_qos(factory, &factory_qos2);
    assert(result == RETCODE_OK);
    assert(factory_qos2.entity_factory.autoenable_created_entities == false);

    // Restore factory QoS to default
    DomainParticipantFactoryQos default_factory_qos = dds_domain_participant_factory_qos_default();
    result = dds_domain_participant_factory_set_qos(factory, &default_factory_qos);
    assert(result == RETCODE_OK);

    printf("C test passed: create_participant, create_publisher, create_subscriber, and participant factory methods succeeded!\n");
    return 0;
}
