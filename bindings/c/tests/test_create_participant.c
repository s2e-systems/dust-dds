#include "unity.h"
#include <string.h>
#include <stdlib.h>
#include "../include/dust_dds.h"

void setUp(void) {}
void tearDown(void) {}

void test_participant_lifecycle(void)
{
    DustDdsDomainParticipantFactory *factory = (DustDdsDomainParticipantFactory *)dds_domain_participant_factory_get_instance();
    TEST_ASSERT_NOT_NULL(factory);

    // Test creating participant with NULL QoS (default QoS)
    DustDdsDomainParticipant *participant = dds_domain_participant_factory_create_participant(
        factory,
        0,
        NULL,
        NULL,
        0);
    TEST_ASSERT_NOT_NULL(participant);

    ReturnCode result = dds_domain_participant_factory_delete_participant(
        factory,
        participant);
    TEST_ASSERT_EQUAL_INT(RETCODE_OK, result);

    // Test creating participant with specific DomainParticipantQos object and custom UserDataQosPolicy
    DomainParticipantQos qos = dds_domain_participant_qos_default();

    const uint8_t custom_user_data[] = {0x01, 0x02, 0x03, 0x04, 0x05};
    qos.user_data.value.length = sizeof(custom_user_data);
    qos.user_data.value.buffer = malloc(sizeof(custom_user_data));
    memcpy(qos.user_data.value.buffer, custom_user_data, sizeof(custom_user_data));

    // Verify user data matches
    TEST_ASSERT_EQUAL_UINT(sizeof(custom_user_data), qos.user_data.value.length);
    TEST_ASSERT_EQUAL_HEX8_ARRAY(custom_user_data, qos.user_data.value.buffer, sizeof(custom_user_data));

    participant = dds_domain_participant_factory_create_participant(
        factory,
        0,
        &qos,
        NULL,
        0);
    TEST_ASSERT_NOT_NULL(participant);

    // Clean up our allocated buffer (the Rust function will make a copy internal to DDS)
    free(qos.user_data.value.buffer);
    qos.user_data.value.buffer = NULL;
    qos.user_data.value.length = 0;

    // Test creating publisher and subscriber on participant with default QoS and NULL
    DustDdsPublisher *publisher = dds_domain_participant_create_publisher(participant, NULL, NULL, 0);
    TEST_ASSERT_NOT_NULL(publisher);
    result = dds_domain_participant_delete_publisher(participant, publisher);
    TEST_ASSERT_EQUAL_INT(RETCODE_OK, result);

    PublisherQos pub_qos = dds_publisher_qos_default();
    publisher = dds_domain_participant_create_publisher(participant, &pub_qos, NULL, 0);
    TEST_ASSERT_NOT_NULL(publisher);
    result = dds_domain_participant_delete_publisher(participant, publisher);
    TEST_ASSERT_EQUAL_INT(RETCODE_OK, result);

    DustDdsSubscriber *subscriber = dds_domain_participant_create_subscriber(participant, NULL, NULL, 0);
    TEST_ASSERT_NOT_NULL(subscriber);
    result = dds_domain_participant_delete_subscriber(participant, subscriber);
    TEST_ASSERT_EQUAL_INT(RETCODE_OK, result);

    SubscriberQos sub_qos = dds_subscriber_qos_default();
    subscriber = dds_domain_participant_create_subscriber(participant, &sub_qos, NULL, 0);
    TEST_ASSERT_NOT_NULL(subscriber);
    result = dds_domain_participant_delete_subscriber(participant, subscriber);
    TEST_ASSERT_EQUAL_INT(RETCODE_OK, result);

    result = dds_domain_participant_factory_delete_participant(
        factory,
        participant);
    TEST_ASSERT_EQUAL_INT(RETCODE_OK, result);

    // Test lookup_participant
    participant = dds_domain_participant_factory_create_participant(
        factory,
        1, // domain_id = 1
        NULL,
        NULL,
        0);
    TEST_ASSERT_NOT_NULL(participant);

    DustDdsDomainParticipant *looked_up = dds_domain_participant_factory_lookup_participant(factory, 1);
    TEST_ASSERT_NOT_NULL(looked_up);

    result = dds_domain_participant_factory_delete_participant(factory, looked_up);
    TEST_ASSERT_EQUAL_INT(RETCODE_OK, result);

    // After deleting, lookup should return NULL
    looked_up = dds_domain_participant_factory_lookup_participant(factory, 1);
    TEST_ASSERT_NULL(looked_up);

    result = dds_domain_participant_factory_delete_participant(factory, participant);
    TEST_ASSERT_EQUAL_INT(RETCODE_ALREADY_DELETED, result);

    // Test get/set default participant QoS
    DomainParticipantQos default_part_qos;
    result = dds_domain_participant_factory_get_default_participant_qos(factory, &default_part_qos);
    TEST_ASSERT_EQUAL_INT(RETCODE_OK, result);

    // Verify entity factory default
    TEST_ASSERT_TRUE(default_part_qos.entity_factory.autoenable_created_entities);

    default_part_qos.entity_factory.autoenable_created_entities = false;

    // Set as default participant QoS
    result = dds_domain_participant_factory_set_default_participant_qos(factory, &default_part_qos);
    TEST_ASSERT_EQUAL_INT(RETCODE_OK, result);

    // Verify default participant QoS was updated
    DomainParticipantQos default_part_qos2;
    result = dds_domain_participant_factory_get_default_participant_qos(factory, &default_part_qos2);
    TEST_ASSERT_EQUAL_INT(RETCODE_OK, result);
    TEST_ASSERT_FALSE(default_part_qos2.entity_factory.autoenable_created_entities);

    // Restore to standard default QoS
    result = dds_domain_participant_factory_set_default_participant_qos(factory, NULL);
    TEST_ASSERT_EQUAL_INT(RETCODE_OK, result);

    // Test factory QoS
    DomainParticipantFactoryQos factory_qos;
    result = dds_domain_participant_factory_get_qos(factory, &factory_qos);
    TEST_ASSERT_EQUAL_INT(RETCODE_OK, result);
    TEST_ASSERT_TRUE(factory_qos.entity_factory.autoenable_created_entities);

    factory_qos.entity_factory.autoenable_created_entities = false;
    result = dds_domain_participant_factory_set_qos(factory, &factory_qos);
    TEST_ASSERT_EQUAL_INT(RETCODE_OK, result);

    // Verify factory QoS updated
    DomainParticipantFactoryQos factory_qos2;
    result = dds_domain_participant_factory_get_qos(factory, &factory_qos2);
    TEST_ASSERT_EQUAL_INT(RETCODE_OK, result);
    TEST_ASSERT_FALSE(factory_qos2.entity_factory.autoenable_created_entities);

    // Restore factory QoS to default
    DomainParticipantFactoryQos default_factory_qos = dds_domain_participant_factory_qos_default();
    result = dds_domain_participant_factory_set_qos(factory, &default_factory_qos);
    TEST_ASSERT_EQUAL_INT(RETCODE_OK, result);
}

int main(void)
{
    UNITY_BEGIN();
    RUN_TEST(test_participant_lifecycle);
    return UNITY_END();
}
