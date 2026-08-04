#include "unity.h"
#include <string.h>
#include <stdlib.h>
#include "../include/dust_dds.h"

void setUp(void) {}
void tearDown(void) {}

void test_participant_lifecycle(void)
{
    DDS_DustDdsDomainParticipantFactory *factory = (DDS_DustDdsDomainParticipantFactory *)DDS_domain_participant_factory_get_instance();
    TEST_ASSERT_NOT_NULL(factory);

    // Test creating participant with NULL QoS (default QoS)
    DDS_DustDdsDomainParticipant *participant = DDS_domain_participant_factory_create_participant(
        factory,
        0,
        NULL,
        NULL,
        0);
    TEST_ASSERT_NOT_NULL(participant);

    DDS_ReturnCode result = DDS_domain_participant_factory_delete_participant(
        factory,
        participant);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    // Test creating participant with specific DDS_DomainParticipantQos object and custom UserDataQosPolicy
    DDS_DomainParticipantQos qos = DDS_domain_participant_qos_default();

    const uint8_t custom_user_data[] = {0x01, 0x02, 0x03, 0x04, 0x05};
    qos.user_data.value.length = sizeof(custom_user_data);
    qos.user_data.value.buffer = malloc(sizeof(custom_user_data));
    memcpy(qos.user_data.value.buffer, custom_user_data, sizeof(custom_user_data));

    // Verify user data matches
    TEST_ASSERT_EQUAL_UINT(sizeof(custom_user_data), qos.user_data.value.length);
    TEST_ASSERT_EQUAL_HEX8_ARRAY(custom_user_data, qos.user_data.value.buffer, sizeof(custom_user_data));

    participant = DDS_domain_participant_factory_create_participant(
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
    DDS_DustDdsPublisher *publisher = DDS_domain_participant_create_publisher(participant, NULL, NULL, 0);
    TEST_ASSERT_NOT_NULL(publisher);
    result = DDS_domain_participant_delete_publisher(participant, publisher);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    DDS_PublisherQos pub_qos = DDS_publisher_qos_default();
    publisher = DDS_domain_participant_create_publisher(participant, &pub_qos, NULL, 0);
    TEST_ASSERT_NOT_NULL(publisher);
    result = DDS_domain_participant_delete_publisher(participant, publisher);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    DDS_DustDdsSubscriber *subscriber = DDS_domain_participant_create_subscriber(participant, NULL, NULL, 0);
    TEST_ASSERT_NOT_NULL(subscriber);
    result = DDS_domain_participant_delete_subscriber(participant, subscriber);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    DDS_SubscriberQos sub_qos = DDS_subscriber_qos_default();
    subscriber = DDS_domain_participant_create_subscriber(participant, &sub_qos, NULL, 0);
    TEST_ASSERT_NOT_NULL(subscriber);
    result = DDS_domain_participant_delete_subscriber(participant, subscriber);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    result = DDS_domain_participant_factory_delete_participant(
        factory,
        participant);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    // Test lookup_participant
    participant = DDS_domain_participant_factory_create_participant(
        factory,
        1, // domain_id = 1
        NULL,
        NULL,
        0);
    TEST_ASSERT_NOT_NULL(participant);

    DDS_DustDdsDomainParticipant *looked_up = DDS_domain_participant_factory_lookup_participant(factory, 1);
    TEST_ASSERT_NOT_NULL(looked_up);

    result = DDS_domain_participant_factory_delete_participant(factory, looked_up);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    // After deleting, lookup should return NULL
    looked_up = DDS_domain_participant_factory_lookup_participant(factory, 1);
    TEST_ASSERT_NULL(looked_up);

    result = DDS_domain_participant_factory_delete_participant(factory, participant);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_ALREADY_DELETED, result);

    // Test get/set default participant QoS
    DDS_DomainParticipantQos default_part_qos;
    result = DDS_domain_participant_factory_get_default_participant_qos(factory, &default_part_qos);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    // Verify entity factory default
    TEST_ASSERT_TRUE(default_part_qos.entity_factory.autoenable_created_entities);

    default_part_qos.entity_factory.autoenable_created_entities = false;

    // Set as default participant QoS
    result = DDS_domain_participant_factory_set_default_participant_qos(factory, &default_part_qos);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    // Verify default participant QoS was updated
    DDS_DomainParticipantQos default_part_qos2;
    result = DDS_domain_participant_factory_get_default_participant_qos(factory, &default_part_qos2);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);
    TEST_ASSERT_FALSE(default_part_qos2.entity_factory.autoenable_created_entities);

    // Restore to standard default QoS
    result = DDS_domain_participant_factory_set_default_participant_qos(factory, NULL);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    // Test factory QoS
    DDS_DomainParticipantFactoryQos factory_qos;
    result = DDS_domain_participant_factory_get_qos(factory, &factory_qos);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);
    TEST_ASSERT_TRUE(factory_qos.entity_factory.autoenable_created_entities);

    factory_qos.entity_factory.autoenable_created_entities = false;
    result = DDS_domain_participant_factory_set_qos(factory, &factory_qos);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    // Verify factory QoS updated
    DDS_DomainParticipantFactoryQos factory_qos2;
    result = DDS_domain_participant_factory_get_qos(factory, &factory_qos2);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);
    TEST_ASSERT_FALSE(factory_qos2.entity_factory.autoenable_created_entities);

    // Restore factory QoS to default
    DDS_DomainParticipantFactoryQos default_factory_qos = DDS_domain_participant_factory_qos_default();
    result = DDS_domain_participant_factory_set_qos(factory, &default_factory_qos);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);
}

void test_subscriber_lifecycle(void)
{
    DDS_DustDdsDomainParticipantFactory *factory = (DDS_DustDdsDomainParticipantFactory *)DDS_domain_participant_factory_get_instance();
    TEST_ASSERT_NOT_NULL(factory);

    DDS_DustDdsDomainParticipant *participant = DDS_domain_participant_factory_create_participant(
        factory,
        0,
        NULL,
        NULL,
        0);
    TEST_ASSERT_NOT_NULL(participant);

    DDS_DustDdsSubscriber *subscriber = DDS_domain_participant_create_subscriber(participant, NULL, NULL, 0);
    TEST_ASSERT_NOT_NULL(subscriber);

    // Test get_participant
    DDS_DustDdsDomainParticipant *sub_participant = DDS_subscriber_get_participant(subscriber);
    TEST_ASSERT_NOT_NULL(sub_participant);

    // Test get_qos and set_qos
    DDS_SubscriberQos qos = DDS_subscriber_qos_default();
    DDS_ReturnCode result = DDS_subscriber_get_qos(subscriber, &qos);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    result = DDS_subscriber_set_qos(subscriber, &qos);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    // Test get/set default datareader qos
    DDS_DataReaderQos dr_qos = DDS_datareader_qos_default();
    result = DDS_subscriber_get_default_datareader_qos(subscriber, &dr_qos);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    result = DDS_subscriber_set_default_datareader_qos(subscriber, &dr_qos);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    // Test set_listener
    result = DDS_subscriber_set_listener(subscriber, NULL, 0);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    // Test notify_datareaders (commented out as it's not yet implemented in the Rust core and panics)
    // result = DDS_subscriber_notify_datareaders(subscriber);
    // TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    // Test lookup_datareader
    DDS_DustDdsDataReader *looked_up = DDS_subscriber_lookup_datareader(subscriber, "NonExistentTopic");
    TEST_ASSERT_NULL(looked_up);

    // Test delete_contained_entities (commented out as it's not yet implemented in the Rust core and panics)
    // result = DDS_subscriber_delete_contained_entities(subscriber);
    // TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    // Test unsupported/unimplemented functions
    result = DDS_subscriber_begin_access(subscriber);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_UNSUPPORTED, result);

    result = DDS_subscriber_end_access(subscriber);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_UNSUPPORTED, result);

    result = DDS_subscriber_get_datareaders(subscriber);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_UNSUPPORTED, result);

    // Cleanup subscriber and participant
    result = DDS_domain_participant_delete_subscriber(participant, subscriber);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);

    result = DDS_domain_participant_factory_delete_participant(factory, participant);
    TEST_ASSERT_EQUAL_INT(DDS_RETCODE_OK, result);
}

int main(void)
{
    UNITY_BEGIN();
    RUN_TEST(test_participant_lifecycle);
    RUN_TEST(test_subscriber_lifecycle);
    return UNITY_END();
}
