#include <float.h>
#include <assert.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include "ddsc/dds.h"
#include "Union.h"

#define MAX_SAMPLES 1

void union_checker(const interoperability_test_UnionType * union_type);

int main(int argc, char *argv[]) {
	const char *topic_name = "Union";

	const dds_entity_t participant = dds_create_participant(DDS_DOMAIN_DEFAULT, NULL /*qos*/, NULL /*listener*/);
	if (participant < 0) {
		DDS_FATAL("dds_create_participant: %s\n", dds_strretcode(-participant));
	}
	const dds_entity_t topic = dds_create_topic(participant, &interoperability_test_UnionTypeWrapper_desc, topic_name, NULL /*qos*/, NULL /*listener*/);
	if (topic < 0) {
		DDS_FATAL("dds_create_topic: %s\n", dds_strretcode(-topic));
	}
	dds_qos_t *qos = dds_create_qos();
	dds_qset_reliability(qos, DDS_RELIABILITY_RELIABLE, DDS_SECS(1));
	dds_qset_durability(qos, DDS_DURABILITY_TRANSIENT_LOCAL);

	const dds_entity_t data_reader = dds_create_reader(participant, topic, qos, NULL /*listener*/);
	if (data_reader < 0) {
		DDS_FATAL("dds_create_reader: %s\n", dds_strretcode(-data_reader));
	}

	dds_return_t rc;

	rc = dds_set_status_mask(data_reader, DDS_SUBSCRIPTION_MATCHED_STATUS);
	if (rc != DDS_RETCODE_OK) {
		DDS_FATAL("dds_set_status_mask: %s\n", dds_strretcode(-rc));
	}

	dds_entity_t waitset = dds_create_waitset(participant);

	rc = dds_waitset_attach(waitset, data_reader, data_reader);
	if (rc != DDS_RETCODE_OK) {
		DDS_FATAL("dds_waitset_attach: %s\n", dds_strretcode(-rc));
	}

	dds_attach_t wsresults[1];
	const size_t wsresultsize = 1U;
	rc = dds_waitset_wait(waitset, wsresults, wsresultsize, DDS_SECS(3660));
	if (rc == 0) {
		DDS_FATAL("dds_waitset_wait: timeout: Subscription not matched");
	}
	if (rc != wsresultsize) {
		DDS_FATAL("dds_waitset_wait: %s\n", dds_strretcode(-rc));
	}

	rc = dds_set_status_mask(data_reader, DDS_DATA_AVAILABLE_STATUS);
	if (rc != DDS_RETCODE_OK) {
		DDS_FATAL("dds_set_status_mask: %s\n", dds_strretcode(-rc));
	}
	rc = dds_waitset_wait(waitset, wsresults, wsresultsize, DDS_SECS(30));
	if (rc == 0) {
		DDS_FATAL("dds_waitset_wait: timeout: No data received");
	}
	if (rc != wsresultsize) {
		DDS_FATAL("dds_waitset_wait: %s\n", dds_strretcode(-rc));
	}

	void *samples[MAX_SAMPLES];
	dds_sample_info_t infos[MAX_SAMPLES];
	samples[0] = interoperability_test_UnionTypeWrapper__alloc();

	rc = dds_read(data_reader, samples, infos, MAX_SAMPLES, MAX_SAMPLES);
	if (rc < 0) {
		DDS_FATAL("dds_read: %s\n", dds_strretcode(-rc));
	}

	if ((rc > 0) && (infos[0].valid_data)) {
		const interoperability_test_UnionTypeWrapper *msg = (interoperability_test_UnionTypeWrapper *) samples[0];
		const size_t length = 20;

		assert(msg->union_sequence._maximum == length);
		assert(msg->union_sequence._length == length);
		printf("union: ");
		union_checker(&msg->union_type);
		printf("array: [\n");
		for (size_t i = 0; i < length; ++i) {
			printf("    ");
			union_checker(&msg->union_array[i]);
		}
		printf("]\n");
		printf("sequence: [\n");
		for (size_t i = 0; i < length; ++i) {
			printf("    ");
			union_checker(&msg->union_sequence._buffer[i]);
		}
		printf("]\n");
	}

	// Sleep to allow sending acknowledgements
	dds_sleepfor(DDS_SECS(2));
}

void union_checker(const interoperability_test_UnionType * union_type) {
	printf("{ d: %d, v: ", union_type->_d);
	switch (union_type->_d) {
		case interoperability_test_UNION_DISCRIMINATOR_NONE: {
			break;
		}
		case interoperability_test_UNION_DISCRIMINATOR_BOOLEAN: {
			printf("%s", union_type->_u.v_boolean ? "true" : "false");
			assert(union_type->_u.v_boolean);
			break;
		}
		case interoperability_test_UNION_DISCRIMINATOR_BYTE: {
			printf("%" PRIu8, union_type->_u.v_byte);
			assert(union_type->_u.v_byte == (UINT8_MAX / 2));
			break;
		}
		case interoperability_test_UNION_DISCRIMINATOR_INT16: {
			printf("%" PRId16, union_type->_u.v_int16);
			assert(union_type->_u.v_int16 == (INT16_MAX / 2));
			break;
		}
		case interoperability_test_UNION_DISCRIMINATOR_INT32: {
			printf("%" PRId32, union_type->_u.v_int32);
			assert(union_type->_u.v_int32 == (INT32_MAX / 2));
			break;
		}
		case interoperability_test_UNION_DISCRIMINATOR_INT64: {
			printf("%" PRId64, union_type->_u.v_int64);
			assert(union_type->_u.v_int64 == (INT64_MAX / 2));
			break;
		}
		case interoperability_test_UNION_DISCRIMINATOR_UINT16: {
			printf("%" PRIu16, union_type->_u.v_uint16);
			assert(union_type->_u.v_uint16 == (UINT16_MAX / 2));
			break;
		}
		case interoperability_test_UNION_DISCRIMINATOR_UINT32: {
			printf("%" PRIu32, union_type->_u.v_uint32);
			assert(union_type->_u.v_uint32 == (UINT32_MAX / 2));
			break;
		}
		case interoperability_test_UNION_DISCRIMINATOR_UINT64: {
			printf("%" PRIu64, union_type->_u.v_uint64);
			assert(union_type->_u.v_uint64 == (UINT64_MAX / 2));
			break;
		}
		case interoperability_test_UNION_DISCRIMINATOR_FLOAT32: {
			printf("%e", union_type->_u.v_float32);
			assert(union_type->_u.v_float32 == (FLT_MAX / 2.0));
			break;
		}
		case interoperability_test_UNION_DISCRIMINATOR_FLOAT64: {
			printf("%e", union_type->_u.v_float64);
			assert(union_type->_u.v_float64 == (DBL_MAX / 2.0));
			break;
		}
		case interoperability_test_UNION_DISCRIMINATOR_INT8: {
			printf("%" PRId8, union_type->_u.v_int8);
			assert(union_type->_u.v_int8 == (INT8_MAX / 2));
			break;
		}
		case interoperability_test_UNION_DISCRIMINATOR_UINT8: {
			printf("%" PRIu8, union_type->_u.v_uint8);
			assert(union_type->_u.v_uint8 == (UINT8_MAX / 2));
			break;
		}
		case interoperability_test_UNION_DISCRIMINATOR_CHAR8: {
			printf("%c", union_type->_u.v_char8);
			assert(union_type->_u.v_char8 == 'M');
			break;
		}
		case interoperability_test_UNION_DISCRIMINATOR_STRING8: {
			printf("\"%s\"", union_type->_u.v_string8);
			assert(strcmp(union_type->_u.v_string8, "Hello World!") == 0);
			break;
		}
		case interoperability_test_UNION_DISCRIMINATOR_ENUM: {
			switch (union_type->_u.v_enum) {
				case interoperability_test_VARIANT_ENUM_X: {
					printf("X");
					break;
				}
				case interoperability_test_VARIANT_ENUM_Y: {
					printf("Y");
					break;
				}
				case interoperability_test_VARIANT_ENUM_Z: {
					printf("Z");
					break;
				}
				default: {
					DDS_FATAL("variant enum: %d", union_type->_u.v_enum);
					break;
				}
			}
			assert(union_type->_u.v_enum == interoperability_test_VARIANT_ENUM_Y);
			break;
		}
		case interoperability_test_UNION_DISCRIMINATOR_STRUCTURE: {
			const interoperability_test_VariantStructure* variant_structure = &union_type->_u.v_structure;
			printf("{ x: %" PRIu8 ", y: %" PRIu16 ", z: %" PRIu32 " }", variant_structure->x, variant_structure->y, variant_structure->z);
			assert(variant_structure->x == (UINT8_MAX / 2));
			assert(variant_structure->y == (UINT16_MAX / 2));
			assert(variant_structure->z == (UINT32_MAX / 2));
			break;
		}
		case interoperability_test_UNION_DISCRIMINATOR_UNION: {
			const interoperability_test_VariantUnion* variant_union = &union_type->_u.v_union;
			switch (variant_union->_d) {
				case 10:
				case 20:
				case 30: {
					printf("{ x: %" PRIu8 " }", variant_union->_u.x);
					assert(variant_union->_u.x == (UINT8_MAX / 2));
					break;
				}
				case 100:
				case 200:
				case 300: {
					printf("{ y: %" PRIu16 " }", variant_union->_u.y);
					assert(variant_union->_u.y == (UINT16_MAX / 2));
					break;
				}
				case 1000:
				case 2000:
				case 3000: {
					printf("{ z: %" PRIu32 " }", variant_union->_u.z);
					assert(variant_union->_u.z == (UINT32_MAX / 2));
					break;
				}
				default: {
					printf("variant union discriminator: %d\n", variant_union->_d);
					assert(false);
					break;
				}
			}
			break;
		}
		case interoperability_test_UNION_DISCRIMINATOR_SEQUENCE: {
			const dds_sequence_string* variant_sequence = &union_type->_u.v_sequence;

			printf("[");
			for(uint32_t i = 0; i < variant_sequence->_length; ++i) {
				printf("\"%s\"", variant_sequence->_buffer[i]);
				if (i < (variant_sequence->_length - 1)) {
					printf(", ");
				}
			}
			printf("]");
			assert(variant_sequence->_maximum == 3);
			assert(variant_sequence->_length == 3);
			assert(strcmp(variant_sequence->_buffer[0], "Hello") == 0);
			assert(strcmp(variant_sequence->_buffer[1], "World") == 0);
			assert(strcmp(variant_sequence->_buffer[2], "!") == 0);
			break;
		}
		case interoperability_test_UNION_DISCRIMINATOR_ARRAY: {
			const double *variant_array = union_type->_u.v_array;
			const uint32_t length = 3;

			printf("[");
			for (uint32_t i = 0; i < length; ++i) {
				printf("%e", variant_array[i]);
				if (i < (length - 1)) {
					printf(", ");
				}
			}
			printf("]");
			assert(variant_array[0] == -DBL_MAX);
			assert(variant_array[1] == 3.14159265358979323846264338327950288);
			assert(variant_array[2] == DBL_MAX);
			break;
		}
		default: {
			printf("union discriminator: %d\n", union_type->_d);
			assert(false);
			break;
		}
	}

	printf(" }\n");
}