use std::path::Path;

#[test]
fn basic_types() {
    let idl_file = Path::new("tests/basic_types.idl");
    let expected = r###"
    #include <stdbool.h>
    #include <stdint.h>
    #include <stddef.h>
    #include <stdlib.h>
    #include <string.h>
    #include "dust_dds.h"

    struct BasicTypes {
        bool a;
        char b;
        wchar_t c;
        uint8_t d;
        char* e;
        wchar_t* f;
        int16_t g;
        uint16_t h;
        int32_t i;
        uint32_t j;
        int64_t k;
        uint64_t l;
        float m;
        double n;
    };

    static inline const DustDdsDynamicType* BasicTypes_get_type(void) {
        static const DustDdsDynamicType* type = NULL;
        if (type == NULL) {
            DustDdsTypeDescriptor descriptor = {
                .kind = TYPE_KIND_STRUCTURE,
                .name = "BasicTypes",
                .base_type = NULL,
                .discriminator_type = NULL,
                .bound = NULL,
                .element_type = NULL,
                .key_element_type = NULL,
                .extensibility_kind = EXTENSIBILITY_KIND_FINAL,
                .is_nested = false
            };
            DustDdsDynamicTypeBuilder* builder = dds_dynamic_type_builder_factory_create_type(&descriptor);
            {
                DustDdsMemberDescriptor member = {
                    .name = "a",
                    .id = 0,
                    .type = dds_dynamic_type_get_primitive_type(TYPE_KIND_BOOLEAN),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                dds_dynamic_type_builder_add_member(builder, &member);
            }
            {
                DustDdsMemberDescriptor member = {
                    .name = "b",
                    .id = 1,
                    .type = dds_dynamic_type_get_primitive_type(TYPE_KIND_CHAR8),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                dds_dynamic_type_builder_add_member(builder, &member);
            }
            {
                DustDdsMemberDescriptor member = {
                    .name = "c",
                    .id = 2,
                    .type = dds_dynamic_type_get_primitive_type(TYPE_KIND_CHAR8),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                dds_dynamic_type_builder_add_member(builder, &member);
            }
            {
                DustDdsMemberDescriptor member = {
                    .name = "d",
                    .id = 3,
                    .type = dds_dynamic_type_get_primitive_type(TYPE_KIND_UINT8),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                dds_dynamic_type_builder_add_member(builder, &member);
            }
            {
                DustDdsDynamicType* member_type = dds_dynamic_type_create_string_type(4294967295);
                DustDdsMemberDescriptor member = {
                    .name = "e",
                    .id = 4,
                    .type = member_type,
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                dds_dynamic_type_builder_add_member(builder, &member);
                dds_dynamic_type_free(member_type);
            }
            {
                DustDdsDynamicType* member_type = dds_dynamic_type_create_string_type(4294967295);
                DustDdsMemberDescriptor member = {
                    .name = "f",
                    .id = 5,
                    .type = member_type,
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                dds_dynamic_type_builder_add_member(builder, &member);
                dds_dynamic_type_free(member_type);
            }
            {
                DustDdsMemberDescriptor member = {
                    .name = "g",
                    .id = 6,
                    .type = dds_dynamic_type_get_primitive_type(TYPE_KIND_INT16),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                dds_dynamic_type_builder_add_member(builder, &member);
            }
            {
                DustDdsMemberDescriptor member = {
                    .name = "h",
                    .id = 7,
                    .type = dds_dynamic_type_get_primitive_type(TYPE_KIND_UINT16),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                dds_dynamic_type_builder_add_member(builder, &member);
            }
            {
                DustDdsMemberDescriptor member = {
                    .name = "i",
                    .id = 8,
                    .type = dds_dynamic_type_get_primitive_type(TYPE_KIND_INT32),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                dds_dynamic_type_builder_add_member(builder, &member);
            }
            {
                DustDdsMemberDescriptor member = {
                    .name = "j",
                    .id = 9,
                    .type = dds_dynamic_type_get_primitive_type(TYPE_KIND_UINT32),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                dds_dynamic_type_builder_add_member(builder, &member);
            }
            {
                DustDdsMemberDescriptor member = {
                    .name = "k",
                    .id = 10,
                    .type = dds_dynamic_type_get_primitive_type(TYPE_KIND_INT64),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                dds_dynamic_type_builder_add_member(builder, &member);
            }
            {
                DustDdsMemberDescriptor member = {
                    .name = "l",
                    .id = 11,
                    .type = dds_dynamic_type_get_primitive_type(TYPE_KIND_UINT64),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                dds_dynamic_type_builder_add_member(builder, &member);
            }
            {
                DustDdsMemberDescriptor member = {
                    .name = "m",
                    .id = 12,
                    .type = dds_dynamic_type_get_primitive_type(TYPE_KIND_FLOAT32),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                dds_dynamic_type_builder_add_member(builder, &member);
            }
            {
                DustDdsMemberDescriptor member = {
                    .name = "n",
                    .id = 13,
                    .type = dds_dynamic_type_get_primitive_type(TYPE_KIND_FLOAT64),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                dds_dynamic_type_builder_add_member(builder, &member);
            }
            type = dds_dynamic_type_builder_build(builder);
        }
        return type;
    }

    static inline struct BasicTypes BasicTypes_create_sample(DustDdsDynamicData* src) {
        struct BasicTypes sample;
        memset(&sample, 0, sizeof(sample));
        dds_dynamic_data_get_boolean_value(src, 0, &sample.a);
        dds_dynamic_data_get_char8_value(src, 1, &sample.b);
        {
            char temp;
            dds_dynamic_data_get_char8_value(src, 2, &temp);
            sample.c = (wchar_t)temp;
        }
        dds_dynamic_data_get_uint8_value(src, 3, &sample.d);
        dds_dynamic_data_get_string_value(src, 4, &sample.e);
        {
            char* temp = NULL;
            dds_dynamic_data_get_string_value(src, 5, &temp);
            if (temp != NULL) {
                size_t len = mbstowcs(NULL, temp, 0);
                if (len != (size_t)-1) {
                    sample.f = malloc((len + 1) * sizeof(wchar_t));
                    mbstowcs(sample.f, temp, len + 1);
                }
                dds_string_free(temp);
            }
        }
        dds_dynamic_data_get_int16_value(src, 6, &sample.g);
        dds_dynamic_data_get_uint16_value(src, 7, &sample.h);
        dds_dynamic_data_get_int32_value(src, 8, &sample.i);
        dds_dynamic_data_get_uint32_value(src, 9, &sample.j);
        dds_dynamic_data_get_int64_value(src, 10, &sample.k);
        dds_dynamic_data_get_uint64_value(src, 11, &sample.l);
        dds_dynamic_data_get_float32_value(src, 12, &sample.m);
        dds_dynamic_data_get_float64_value(src, 13, &sample.n);
        return sample;
    }

    static inline DustDdsDynamicData* BasicTypes_create_dynamic_sample(const struct BasicTypes* src) {
        DustDdsDynamicData* sample = dds_dynamic_data_create(BasicTypes_get_type());
        if (sample != NULL) {
            dds_dynamic_data_set_boolean_value(sample, 0, src->a);
            dds_dynamic_data_set_char8_value(sample, 1, src->b);
            dds_dynamic_data_set_char8_value(sample, 2, (char)src->c);
            dds_dynamic_data_set_uint8_value(sample, 3, src->d);
            dds_dynamic_data_set_string_value(sample, 4, src->e);
            {
                if (src->f != NULL) {
                    size_t len = wcstombs(NULL, src->f, 0);
                    if (len != (size_t)-1) {
                        char* temp = malloc(len + 1);
                        wcstombs(temp, src->f, len + 1);
                        dds_dynamic_data_set_string_value(sample, 5, temp);
                        free(temp);
                    }
                }
            }
            dds_dynamic_data_set_int16_value(sample, 6, src->g);
            dds_dynamic_data_set_uint16_value(sample, 7, src->h);
            dds_dynamic_data_set_int32_value(sample, 8, src->i);
            dds_dynamic_data_set_uint32_value(sample, 9, src->j);
            dds_dynamic_data_set_int64_value(sample, 10, src->k);
            dds_dynamic_data_set_uint64_value(sample, 11, src->l);
            dds_dynamic_data_set_float32_value(sample, 12, src->m);
            dds_dynamic_data_set_float64_value(sample, 13, src->n);
        }
        return sample;
    }

    static inline void BasicTypes_free_sample(struct BasicTypes* sample) {
        if (sample != NULL) {
        dds_string_free(sample->e);
        free(sample->f);
        }
    }

    static inline ReturnCode BasicTypes_dds_datawriter_write(DustDdsDataWriter* writer, const struct BasicTypes* data, const InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = BasicTypes_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_write(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode BasicTypes_dds_datawriter_write_w_timestamp(DustDdsDataWriter* writer, const struct BasicTypes* data, const InstanceHandle_t* handle, struct Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = BasicTypes_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_write_w_timestamp(writer, sample, handle, source_timestamp);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode BasicTypes_dds_datawriter_register_instance(DustDdsDataWriter* writer, const struct BasicTypes* data, InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL || handle == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = BasicTypes_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_register_instance(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode BasicTypes_dds_datawriter_register_instance_w_timestamp(DustDdsDataWriter* writer, const struct BasicTypes* data, struct Time_t source_timestamp, InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL || handle == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = BasicTypes_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_register_instance_w_timestamp(writer, sample, source_timestamp, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode BasicTypes_dds_datawriter_unregister_instance(DustDdsDataWriter* writer, const struct BasicTypes* data, const InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = BasicTypes_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_unregister_instance(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode BasicTypes_dds_datawriter_unregister_instance_w_timestamp(DustDdsDataWriter* writer, const struct BasicTypes* data, const InstanceHandle_t* handle, struct Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = BasicTypes_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_unregister_instance_w_timestamp(writer, sample, handle, source_timestamp);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode BasicTypes_dds_datawriter_dispose(DustDdsDataWriter* writer, const struct BasicTypes* data, const InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = BasicTypes_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_dispose(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode BasicTypes_dds_datawriter_dispose_w_timestamp(DustDdsDataWriter* writer, const struct BasicTypes* data, const InstanceHandle_t* handle, struct Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = BasicTypes_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_dispose_w_timestamp(writer, sample, handle, source_timestamp);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode BasicTypes_dds_datawriter_get_key_value(DustDdsDataWriter* writer, struct BasicTypes* key_holder, const InstanceHandle_t* handle) {
        if (writer == NULL || key_holder == NULL || handle == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = BasicTypes_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_get_key_value(writer, sample, handle);
        if (result == RETCODE_OK) {
            *key_holder = BasicTypes_create_sample(sample);
        }
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode BasicTypes_dds_datawriter_lookup_instance(DustDdsDataWriter* writer, const struct BasicTypes* key_holder, InstanceHandle_t* handle) {
        if (writer == NULL || key_holder == NULL || handle == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = BasicTypes_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_lookup_instance(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode BasicTypes_dds_datareader_read(DustDdsDataReader* reader, struct BasicTypes* data_values, struct SampleInfo* sample_infos, int32_t max_samples, SampleStateMask sample_states, ViewStateMask view_states, InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || received_samples == NULL || max_samples <= 0) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData** samples = (DustDdsDynamicData**)calloc(max_samples, sizeof(DustDdsDynamicData*));
        if (samples == NULL) {
            return RETCODE_OUT_OF_RESOURCES;
        }
        ReturnCode result = dds_datareader_read(reader, samples, sample_infos, max_samples, sample_states, view_states, instance_states, received_samples);
        if (result == RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = BasicTypes_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline ReturnCode BasicTypes_dds_datareader_take(DustDdsDataReader* reader, struct BasicTypes* data_values, struct SampleInfo* sample_infos, int32_t max_samples, SampleStateMask sample_states, ViewStateMask view_states, InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || received_samples == NULL || max_samples <= 0) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData** samples = (DustDdsDynamicData**)calloc(max_samples, sizeof(DustDdsDynamicData*));
        if (samples == NULL) {
            return RETCODE_OUT_OF_RESOURCES;
        }
        ReturnCode result = dds_datareader_take(reader, samples, sample_infos, max_samples, sample_states, view_states, instance_states, received_samples);
        if (result == RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = BasicTypes_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline ReturnCode BasicTypes_dds_datareader_read_next_sample(DustDdsDataReader* reader, struct BasicTypes* data_value, struct SampleInfo* sample_info) {
        if (reader == NULL || data_value == NULL || sample_info == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = NULL;
        ReturnCode result = dds_datareader_read_next_sample(reader, &sample, sample_info);
        if (result == RETCODE_OK) {
            if (sample != NULL) {
                *data_value = BasicTypes_create_sample(sample);
                dds_dynamic_data_free(sample);
            }
        }
        return result;
    }

    static inline ReturnCode BasicTypes_dds_datareader_take_next_sample(DustDdsDataReader* reader, struct BasicTypes* data_value, struct SampleInfo* sample_info) {
        if (reader == NULL || data_value == NULL || sample_info == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = NULL;
        ReturnCode result = dds_datareader_take_next_sample(reader, &sample, sample_info);
        if (result == RETCODE_OK) {
            if (sample != NULL) {
                *data_value = BasicTypes_create_sample(sample);
                dds_dynamic_data_free(sample);
            }
        }
        return result;
    }

    static inline ReturnCode BasicTypes_dds_datareader_read_instance(DustDdsDataReader* reader, struct BasicTypes* data_values, struct SampleInfo* sample_infos, int32_t max_samples, const InstanceHandle_t* a_handle, SampleStateMask sample_states, ViewStateMask view_states, InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || a_handle == NULL || received_samples == NULL || max_samples <= 0) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData** samples = (DustDdsDynamicData**)calloc(max_samples, sizeof(DustDdsDynamicData*));
        if (samples == NULL) {
            return RETCODE_OUT_OF_RESOURCES;
        }
        ReturnCode result = dds_datareader_read_instance(reader, samples, sample_infos, max_samples, a_handle, sample_states, view_states, instance_states, received_samples);
        if (result == RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = BasicTypes_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline ReturnCode BasicTypes_dds_datareader_take_instance(DustDdsDataReader* reader, struct BasicTypes* data_values, struct SampleInfo* sample_infos, int32_t max_samples, const InstanceHandle_t* a_handle, SampleStateMask sample_states, ViewStateMask view_states, InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || a_handle == NULL || received_samples == NULL || max_samples <= 0) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData** samples = (DustDdsDynamicData**)calloc(max_samples, sizeof(DustDdsDynamicData*));
        if (samples == NULL) {
            return RETCODE_OUT_OF_RESOURCES;
        }
        ReturnCode result = dds_datareader_take_instance(reader, samples, sample_infos, max_samples, a_handle, sample_states, view_states, instance_states, received_samples);
        if (result == RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = BasicTypes_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline ReturnCode BasicTypes_dds_datareader_read_next_instance(DustDdsDataReader* reader, struct BasicTypes* data_values, struct SampleInfo* sample_infos, int32_t max_samples, const InstanceHandle_t* previous_handle, SampleStateMask sample_states, ViewStateMask view_states, InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || previous_handle == NULL || received_samples == NULL || max_samples <= 0) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData** samples = (DustDdsDynamicData**)calloc(max_samples, sizeof(DustDdsDynamicData*));
        if (samples == NULL) {
            return RETCODE_OUT_OF_RESOURCES;
        }
        ReturnCode result = dds_datareader_read_next_instance(reader, samples, sample_infos, max_samples, previous_handle, sample_states, view_states, instance_states, received_samples);
        if (result == RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = BasicTypes_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline ReturnCode BasicTypes_dds_datareader_take_next_instance(DustDdsDataReader* reader, struct BasicTypes* data_values, struct SampleInfo* sample_infos, int32_t max_samples, const InstanceHandle_t* previous_handle, SampleStateMask sample_states, ViewStateMask view_states, InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || previous_handle == NULL || received_samples == NULL || max_samples <= 0) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData** samples = (DustDdsDynamicData**)calloc(max_samples, sizeof(DustDdsDynamicData*));
        if (samples == NULL) {
            return RETCODE_OUT_OF_RESOURCES;
        }
        ReturnCode result = dds_datareader_take_next_instance(reader, samples, sample_infos, max_samples, previous_handle, sample_states, view_states, instance_states, received_samples);
        if (result == RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = BasicTypes_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline ReturnCode BasicTypes_dds_datareader_return_loan(DustDdsDataReader* reader, struct BasicTypes* data_values, struct SampleInfo* sample_infos) {
        return dds_datareader_return_loan(reader, NULL, sample_infos);
    }

    static inline ReturnCode BasicTypes_dds_datareader_get_key_value(DustDdsDataReader* reader, struct BasicTypes* key_holder, const InstanceHandle_t* handle) {
        if (reader == NULL || key_holder == NULL || handle == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = BasicTypes_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datareader_get_key_value(reader, sample, handle);
        if (result == RETCODE_OK) {
            *key_holder = BasicTypes_create_sample(sample);
        }
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode BasicTypes_dds_datareader_lookup_instance(DustDdsDataReader* reader, const struct BasicTypes* key_holder, InstanceHandle_t* handle) {
        if (reader == NULL || key_holder == NULL || handle == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = BasicTypes_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datareader_lookup_instance(reader, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }
"###;

    let result = dust_dds_gen::compile_idl_c(idl_file).unwrap();

    assert_eq!(result, expected);
}
