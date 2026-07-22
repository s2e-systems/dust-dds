#include <stdio.h>
#include <assert.h>
#include <string.h>
#include "../include/dust_dds.h"

int main(void) {
    DustDdsDomainParticipantFactory* factory = (DustDdsDomainParticipantFactory*)dds_domain_participant_factory_get_instance();
    assert(factory != NULL);

    // Test creating participant with NULL QoS (default QoS)
    DustDdsDomainParticipant* participant = dds_domain_participant_factory_create_participant(
        factory,
        0,
        DUST_DDS_PARTICIPANT_QOS_DEFAULT
    );
    assert(participant != NULL);

    ReturnCode result = dds_domain_participant_factory_delete_participant(
        factory,
        participant
    );
    assert(result == RETCODE_OK);

    // Test creating participant with specific DomainParticipantQos object and custom UserDataQosPolicy
    DustDdsDomainParticipantQos* qos = dds_domain_participant_qos_default();
    assert(qos != NULL);

    const uint8_t custom_user_data[] = {0x01, 0x02, 0x03, 0x04, 0x05};
    DustDdsUserDataQosPolicy* user_data = dds_user_data_qos_policy_new(custom_user_data, sizeof(custom_user_data));
    assert(user_data != NULL);

    dds_domain_participant_qos_set_user_data(qos, user_data);

    // Verify getting user_data back from qos
    DustDdsUserDataQosPolicy* retrieved_user_data = dds_domain_participant_qos_get_user_data(qos);
    assert(retrieved_user_data != NULL);
    assert(dds_user_data_qos_policy_get_value_length(retrieved_user_data) == sizeof(custom_user_data));

    uint8_t buffer[10] = {0};
    size_t copied = dds_user_data_qos_policy_get_value(retrieved_user_data, buffer, sizeof(buffer));
    assert(copied == sizeof(custom_user_data));
    assert(memcmp(buffer, custom_user_data, sizeof(custom_user_data)) == 0);

    dds_user_data_qos_policy_free(retrieved_user_data);
    dds_user_data_qos_policy_free(user_data);

    participant = dds_domain_participant_factory_create_participant(
        factory,
        0,
        qos
    );
    assert(participant != NULL);

    // Test creating publisher and subscriber on participant with default QoS macros and NULL
    DustDdsPublisher* publisher = dds_domain_participant_create_publisher(participant, DUST_DDS_PUBLISHER_QOS_DEFAULT);
    assert(publisher != NULL);
    result = dds_domain_participant_delete_publisher(participant, publisher);
    assert(result == RETCODE_OK);

    DustDdsPublisherQos* pub_qos = dds_publisher_qos_default();
    assert(pub_qos != NULL);
    publisher = dds_domain_participant_create_publisher(participant, pub_qos);
    assert(publisher != NULL);
    dds_publisher_qos_free(pub_qos);
    result = dds_domain_participant_delete_publisher(participant, publisher);
    assert(result == RETCODE_OK);

    DustDdsSubscriber* subscriber = dds_domain_participant_create_subscriber(participant, DUST_DDS_SUBSCRIBER_QOS_DEFAULT);
    assert(subscriber != NULL);
    result = dds_domain_participant_delete_subscriber(participant, subscriber);
    assert(result == RETCODE_OK);

    DustDdsSubscriberQos* sub_qos = dds_subscriber_qos_default();
    assert(sub_qos != NULL);
    subscriber = dds_domain_participant_create_subscriber(participant, sub_qos);
    assert(subscriber != NULL);
    dds_subscriber_qos_free(sub_qos);
    result = dds_domain_participant_delete_subscriber(participant, subscriber);
    assert(result == RETCODE_OK);

    result = dds_domain_participant_factory_delete_participant(
        factory,
        participant
    );
    assert(result == RETCODE_OK);

    dds_domain_participant_qos_free(qos);

    // Test lookup_participant
    participant = dds_domain_participant_factory_create_participant(
        factory,
        1, // domain_id = 1
        DUST_DDS_PARTICIPANT_QOS_DEFAULT
    );
    assert(participant != NULL);

    DustDdsDomainParticipant* looked_up = dds_domain_participant_factory_lookup_participant(factory, 1);
    assert(looked_up != NULL);

    result = dds_domain_participant_factory_delete_participant(factory, looked_up);
    assert(result == RETCODE_OK);

    // After deleting, lookup should return NULL
    looked_up = dds_domain_participant_factory_lookup_participant(factory, 1);
    assert(looked_up == NULL);

    // Manually delete/free the other wrapper. Since it's already deleted in DDS, calling delete_participant will fail, but we don't have a direct free function in the C API.
    // That's ok for integration tests to just let it leak or we can test delete failing.
    result = dds_domain_participant_factory_delete_participant(factory, participant);
    assert(result == RETCODE_ALREADY_DELETED);

    // Test get/set default participant QoS
    DustDdsDomainParticipantQos* default_part_qos = dds_domain_participant_factory_get_default_participant_qos(factory);
    assert(default_part_qos != NULL);

    // Verify entity factory get/set on participant QoS
    DustDdsEntityFactoryQosPolicy* entity_factory = dds_domain_participant_qos_get_entity_factory(default_part_qos);
    assert(entity_factory != NULL);
    assert(dds_entity_factory_qos_policy_get_autoenable_created_entities(entity_factory) == true);

    dds_entity_factory_qos_policy_set_autoenable_created_entities(entity_factory, false);
    assert(dds_entity_factory_qos_policy_get_autoenable_created_entities(entity_factory) == false);

    dds_domain_participant_qos_set_entity_factory(default_part_qos, entity_factory);
    dds_entity_factory_qos_policy_free(entity_factory);

    // Set as default participant QoS
    result = dds_domain_participant_factory_set_default_participant_qos(factory, default_part_qos);
    assert(result == RETCODE_OK);
    dds_domain_participant_qos_free(default_part_qos);

    // Verify default participant QoS was updated
    default_part_qos = dds_domain_participant_factory_get_default_participant_qos(factory);
    assert(default_part_qos != NULL);
    entity_factory = dds_domain_participant_qos_get_entity_factory(default_part_qos);
    assert(entity_factory != NULL);
    assert(dds_entity_factory_qos_policy_get_autoenable_created_entities(entity_factory) == false);
    dds_entity_factory_qos_policy_free(entity_factory);
    dds_domain_participant_qos_free(default_part_qos);

    // Restore to standard default QoS
    result = dds_domain_participant_factory_set_default_participant_qos(factory, DUST_DDS_PARTICIPANT_QOS_DEFAULT);
    assert(result == RETCODE_OK);

    // Test factory QoS
    DustDdsDomainParticipantFactoryQos* factory_qos = dds_domain_participant_factory_get_qos(factory);
    assert(factory_qos != NULL);

    DustDdsEntityFactoryQosPolicy* factory_entity_factory = dds_domain_participant_factory_qos_get_entity_factory(factory_qos);
    assert(factory_entity_factory != NULL);
    assert(dds_entity_factory_qos_policy_get_autoenable_created_entities(factory_entity_factory) == true);

    dds_entity_factory_qos_policy_set_autoenable_created_entities(factory_entity_factory, false);
    dds_domain_participant_factory_qos_set_entity_factory(factory_qos, factory_entity_factory);
    dds_entity_factory_qos_policy_free(factory_entity_factory);

    result = dds_domain_participant_factory_set_qos(factory, factory_qos);
    assert(result == RETCODE_OK);
    dds_domain_participant_factory_qos_free(factory_qos);

    // Verify factory QoS updated
    factory_qos = dds_domain_participant_factory_get_qos(factory);
    assert(factory_qos != NULL);
    factory_entity_factory = dds_domain_participant_factory_qos_get_entity_factory(factory_qos);
    assert(factory_entity_factory != NULL);
    assert(dds_entity_factory_qos_policy_get_autoenable_created_entities(factory_entity_factory) == false);
    dds_entity_factory_qos_policy_free(factory_entity_factory);
    dds_domain_participant_factory_qos_free(factory_qos);

    // Restore factory QoS to default
    DustDdsDomainParticipantFactoryQos* default_factory_qos = dds_domain_participant_factory_qos_default();
    assert(default_factory_qos != NULL);
    result = dds_domain_participant_factory_set_qos(factory, default_factory_qos);
    assert(result == RETCODE_OK);
    dds_domain_participant_factory_qos_free(default_factory_qos);

    printf("C test passed: create_participant, create_publisher, create_subscriber, and participant factory methods succeeded!\n");
    return 0;
}



