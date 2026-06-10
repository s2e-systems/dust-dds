#include <float.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include "ddsc/dds.h"
#include "Union.h"

size_t random_value(size_t min, size_t max);

interoperability_test_VariantUnion variant_union_random();

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

	const dds_entity_t data_writer = dds_create_writer(participant, topic, qos, NULL /*listener*/);
	if (data_writer < 0) {
		DDS_FATAL("dds_create_writer: %s\n", dds_strretcode(-data_writer));
	}

	dds_return_t rc;

	rc = dds_set_status_mask(data_writer, DDS_PUBLICATION_MATCHED_STATUS);
	if (rc != DDS_RETCODE_OK) {
		DDS_FATAL("dds_set_status_mask: %s\n", dds_strretcode(-rc));
	}

	dds_entity_t waitset = dds_create_waitset(participant);

	rc = dds_waitset_attach(waitset, data_writer, data_writer);
	if (rc != DDS_RETCODE_OK) {
		DDS_FATAL("dds_waitset_attach: %s\n", dds_strretcode(-rc));
	}

	dds_attach_t wsresults[1];
	const size_t wsresultsize = 1U;
	rc = dds_waitset_wait(waitset, wsresults, wsresultsize, DDS_SECS(60));
	if (rc == 0) {
		DDS_FATAL("dds_waitset_wait: timeout");
	}
	if (rc != wsresultsize) {
		DDS_FATAL("dds_waitset_wait: %s\n", dds_strretcode(-rc));
	}

    srand((unsigned int) time(NULL));

	const interoperability_test_UnionType unions[] = {
		{
			._d = interoperability_test_UNION_DISCRIMINATOR_NONE
		},
		{
			._d = interoperability_test_UNION_DISCRIMINATOR_BOOLEAN,
			._u = {
				.v_boolean = true
			}
		},
		{
			._d = interoperability_test_UNION_DISCRIMINATOR_BYTE,
			._u = {
				.v_byte = UINT8_MAX / 2
			}
		},
		{
			._d = interoperability_test_UNION_DISCRIMINATOR_INT16,
			._u = {
				.v_int16 = INT16_MAX / 2
			}
		},
		{
			._d = interoperability_test_UNION_DISCRIMINATOR_INT32,
			._u = {
				.v_int32 = INT32_MAX / 2
			}
		},
		{
			._d = interoperability_test_UNION_DISCRIMINATOR_INT64,
			._u = {
				.v_int64 = INT64_MAX / 2
			}
		},
		{
			._d = interoperability_test_UNION_DISCRIMINATOR_UINT16,
			._u = {
				.v_uint16 = UINT16_MAX / 2
			}
		},
		{
			._d = interoperability_test_UNION_DISCRIMINATOR_UINT32,
			._u = {
				.v_uint32 = UINT32_MAX / 2
			}
		},
		{
			._d = interoperability_test_UNION_DISCRIMINATOR_UINT64,
			._u = {
				.v_uint64 = UINT64_MAX / 2
			}
		},
		{
			._d = interoperability_test_UNION_DISCRIMINATOR_FLOAT32,
			._u = {
				.v_float32 = FLT_MAX / 2.0
			}
		},
		{
			._d = interoperability_test_UNION_DISCRIMINATOR_FLOAT64,
			._u = {
				.v_float64 = DBL_MAX / 2.0
			}
		},
		{
			._d = interoperability_test_UNION_DISCRIMINATOR_INT8,
			._u = {
				.v_int8 = INT8_MAX / 2
			}
		},
		{
			._d = interoperability_test_UNION_DISCRIMINATOR_UINT8,
			._u = {
				.v_uint8 = UINT8_MAX / 2
			}
		},
		{
			._d = interoperability_test_UNION_DISCRIMINATOR_CHAR8,
			._u = {
				.v_char8 = 'M'
			}
		},
		{
			._d = interoperability_test_UNION_DISCRIMINATOR_STRING8,
			._u = {
				.v_string8 = "Hello World!"
			}
		},
		{
			._d = interoperability_test_UNION_DISCRIMINATOR_ENUM,
			._u = {
				.v_enum = interoperability_test_VARIANT_ENUM_Y
			}
		},
		{
			._d = interoperability_test_UNION_DISCRIMINATOR_STRUCTURE,
			._u = {
				.v_structure = {
					.x = UINT8_MAX / 2,
					.y = UINT16_MAX / 2,
					.z = UINT32_MAX / 2
				}
			}
		},
		{
			._d = interoperability_test_UNION_DISCRIMINATOR_UNION,
			._u = {
				.v_union = variant_union_random()
			}
		},
		{
			._d = interoperability_test_UNION_DISCRIMINATOR_SEQUENCE,
			._u = {
				.v_sequence = {
					._maximum = 3,
					._length = 3,
					._buffer = (char *[]) {"Hello", "World", "!"},
					._release = false
				}
			}
		},
		{
			._d = interoperability_test_UNION_DISCRIMINATOR_ARRAY,
			._u = {
				.v_array = {-DBL_MAX, 3.14159265358979323846264338327950288, DBL_MAX}
			}
		}
	};

	interoperability_test_UnionTypeWrapper msg;
	msg.union_type =  unions[random_value(0, (sizeof(unions) / sizeof(unions[0])) - 1)];
	memcpy(msg.union_array, unions, sizeof(unions));
	msg.union_sequence._maximum = (uint32_t) (sizeof(unions) / sizeof(unions[0]));
	msg.union_sequence._length = (uint32_t) (sizeof(unions) / sizeof(unions[0]));
	msg.union_sequence._buffer = (interoperability_test_UnionType *) unions;
	msg.union_sequence._release = false;

	dds_write(data_writer, &msg);

	rc = dds_wait_for_acks(data_writer, DDS_SECS(30));
	if (rc != DDS_RETCODE_OK) {
		DDS_FATAL("dds_wait_for_acks: %s\n", dds_strretcode(-rc));
	}
}

size_t random_value(size_t min, size_t max) {
	return ((size_t) rand() % (max - min + 1)) + min;
}

interoperability_test_VariantUnion variant_union_random() {
	switch (random_value(0, 2)) {
		case 0: {
			return (interoperability_test_VariantUnion) {
				._d = (int32_t) (10 * random_value(1, 3)),
				._u = {
					.x = UINT8_MAX / 2
				}
			};
		}
		case 1: {
			return (interoperability_test_VariantUnion) {
				._d = (int32_t) (100 * random_value(1, 3)),
				._u = {
					.y = UINT16_MAX / 2
				}
			};
		}
		case 2: {
			return (interoperability_test_VariantUnion) {
				._d = (int32_t) (1000 * random_value(1, 3)),
				._u = {
					.z = UINT32_MAX / 2
				}
			};
		}
		default: {
			#if defined(__STDC_VERSION__) && __STDC_VERSION__ >= 202311L
				unreachable();
			#elif defined(__GNUC__) || defined(__clang__)
				__builtin_unreachable();
			#elif defined(_MSC_VER)
				__assume(false);
			#else
				assert(false);
			#endif
		}
	}
}