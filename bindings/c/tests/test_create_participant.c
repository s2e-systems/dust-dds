#include <stdio.h>
#include <assert.h>
#include <string.h>
#include "../include/dust_dds.h"

int main(void) {
    DustDdsDomainParticipantFactory* factory = (DustDdsDomainParticipantFactory*)dust_dds_domain_participant_factory_get_instance();
    assert(factory != NULL);

    // Test creating participant with NULL QoS (default QoS)
    DustDdsDomainParticipant* participant = dust_dds_domain_participant_factory_create_participant(
        factory,
        0,
        NULL
    );
    assert(participant != NULL);

    ReturnCode result = dust_dds_domain_participant_factory_delete_participant(
        factory,
        participant
    );
    assert(result == RETCODE_OK);

    // Test creating participant with specific DomainParticipantQos object and custom UserDataQosPolicy
    DustDdsDomainParticipantQos* qos = dust_dds_domain_participant_qos_default();
    assert(qos != NULL);

    const uint8_t custom_user_data[] = {0x01, 0x02, 0x03, 0x04, 0x05};
    DustDdsUserDataQosPolicy* user_data = dust_dds_user_data_qos_policy_new(custom_user_data, sizeof(custom_user_data));
    assert(user_data != NULL);

    dust_dds_domain_participant_qos_set_user_data(qos, user_data);

    // Verify getting user_data back from qos
    DustDdsUserDataQosPolicy* retrieved_user_data = dust_dds_domain_participant_qos_get_user_data(qos);
    assert(retrieved_user_data != NULL);
    assert(dust_dds_user_data_qos_policy_get_value_length(retrieved_user_data) == sizeof(custom_user_data));

    uint8_t buffer[10] = {0};
    size_t copied = dust_dds_user_data_qos_policy_get_value(retrieved_user_data, buffer, sizeof(buffer));
    assert(copied == sizeof(custom_user_data));
    assert(memcmp(buffer, custom_user_data, sizeof(custom_user_data)) == 0);

    dust_dds_user_data_qos_policy_free(retrieved_user_data);
    dust_dds_user_data_qos_policy_free(user_data);

    participant = dust_dds_domain_participant_factory_create_participant(
        factory,
        0,
        qos
    );
    assert(participant != NULL);

    result = dust_dds_domain_participant_factory_delete_participant(
        factory,
        participant
    );
    assert(result == RETCODE_OK);

    dust_dds_domain_participant_qos_free(qos);

    printf("C test passed: create_participant with qos and custom UserDataQosPolicy succeeded!\n");
    return 0;
}


