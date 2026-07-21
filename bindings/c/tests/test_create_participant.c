#include <stdio.h>
#include <assert.h>
#include "../include/dust_dds.h"

int main(void) {
    const DustDdsDomainParticipantFactory* factory = dust_dds_domain_participant_factory_get_instance();
    assert(factory != NULL);

    DustDdsDomainParticipant* participant = dust_dds_domain_participant_factory_create_participant(
        factory,
        0
    );
    assert(participant != NULL);

    ReturnCode result = dust_dds_domain_participant_factory_delete_participant(
        factory,
        participant
    );
    assert(result == RETCODE_OK);

    printf("C test passed: create_participant and delete_participant succeeded!\n");
    return 0;
}
