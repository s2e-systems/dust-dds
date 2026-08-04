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

    static inline const DDS_DynamicType* BasicTypes_get_type(void) {
        static const DDS_DynamicType* type = NULL;
        if (type == NULL) {
            DDS_TypeDescriptor descriptor = {
                .kind = DDS_TYPE_KIND_STRUCTURE,
                .name = "BasicTypes",
                .base_type = NULL,
                .discriminator_type = NULL,
                .bound = NULL,
                .element_type = NULL,
                .key_element_type = NULL,
                .extensibility_kind = DDS_EXTENSIBILITY_KIND_FINAL,
                .is_nested = false
            };
            DDS_DynamicTypeBuilder* builder = DDS_DynamicTypeBuilderFactory_create_type(&descriptor);
            {
                DDS_MemberDescriptor member = {
                    .name = "a",
                    .id = 0,
                    .type = DDS_DynamicType_get_primitive_type(DDS_TYPE_KIND_BOOLEAN),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                DDS_DynamicTypeBuilder_add_member(builder, &member);
            }
            {
                DDS_MemberDescriptor member = {
                    .name = "b",
                    .id = 1,
                    .type = DDS_DynamicType_get_primitive_type(DDS_TYPE_KIND_CHAR8),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                DDS_DynamicTypeBuilder_add_member(builder, &member);
            }
            {
                DDS_MemberDescriptor member = {
                    .name = "c",
                    .id = 2,
                    .type = DDS_DynamicType_get_primitive_type(DDS_TYPE_KIND_CHAR8),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                DDS_DynamicTypeBuilder_add_member(builder, &member);
            }
            {
                DDS_MemberDescriptor member = {
                    .name = "d",
                    .id = 3,
                    .type = DDS_DynamicType_get_primitive_type(DDS_TYPE_KIND_UINT8),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                DDS_DynamicTypeBuilder_add_member(builder, &member);
            }
            {
                DDS_DynamicType* member_type = DDS_DynamicType_create_string_type(4294967295);
                DDS_MemberDescriptor member = {
                    .name = "e",
                    .id = 4,
                    .type = member_type,
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                DDS_DynamicTypeBuilder_add_member(builder, &member);
                DDS_DynamicType_free(member_type);
            }
            {
                DDS_DynamicType* member_type = DDS_DynamicType_create_string_type(4294967295);
                DDS_MemberDescriptor member = {
                    .name = "f",
                    .id = 5,
                    .type = member_type,
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                DDS_DynamicTypeBuilder_add_member(builder, &member);
                DDS_DynamicType_free(member_type);
            }
            {
                DDS_MemberDescriptor member = {
                    .name = "g",
                    .id = 6,
                    .type = DDS_DynamicType_get_primitive_type(DDS_TYPE_KIND_INT16),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                DDS_DynamicTypeBuilder_add_member(builder, &member);
            }
            {
                DDS_MemberDescriptor member = {
                    .name = "h",
                    .id = 7,
                    .type = DDS_DynamicType_get_primitive_type(DDS_TYPE_KIND_UINT16),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                DDS_DynamicTypeBuilder_add_member(builder, &member);
            }
            {
                DDS_MemberDescriptor member = {
                    .name = "i",
                    .id = 8,
                    .type = DDS_DynamicType_get_primitive_type(DDS_TYPE_KIND_INT32),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                DDS_DynamicTypeBuilder_add_member(builder, &member);
            }
            {
                DDS_MemberDescriptor member = {
                    .name = "j",
                    .id = 9,
                    .type = DDS_DynamicType_get_primitive_type(DDS_TYPE_KIND_UINT32),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                DDS_DynamicTypeBuilder_add_member(builder, &member);
            }
            {
                DDS_MemberDescriptor member = {
                    .name = "k",
                    .id = 10,
                    .type = DDS_DynamicType_get_primitive_type(DDS_TYPE_KIND_INT64),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                DDS_DynamicTypeBuilder_add_member(builder, &member);
            }
            {
                DDS_MemberDescriptor member = {
                    .name = "l",
                    .id = 11,
                    .type = DDS_DynamicType_get_primitive_type(DDS_TYPE_KIND_UINT64),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                DDS_DynamicTypeBuilder_add_member(builder, &member);
            }
            {
                DDS_MemberDescriptor member = {
                    .name = "m",
                    .id = 12,
                    .type = DDS_DynamicType_get_primitive_type(DDS_TYPE_KIND_FLOAT32),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                DDS_DynamicTypeBuilder_add_member(builder, &member);
            }
            {
                DDS_MemberDescriptor member = {
                    .name = "n",
                    .id = 13,
                    .type = DDS_DynamicType_get_primitive_type(DDS_TYPE_KIND_FLOAT64),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                DDS_DynamicTypeBuilder_add_member(builder, &member);
            }
            type = DDS_DynamicTypeBuilder_build(builder);
        }
        return type;
    }

    static inline struct BasicTypes BasicTypes_create_sample(DDS_DynamicData* src) {
        struct BasicTypes sample;
        memset(&sample, 0, sizeof(sample));
        DDS_DynamicData_get_boolean_value(src, 0, &sample.a);
        DDS_DynamicData_get_char8_value(src, 1, &sample.b);
        {
            char temp;
            DDS_DynamicData_get_char8_value(src, 2, &temp);
            sample.c = (wchar_t)temp;
        }
        DDS_DynamicData_get_uint8_value(src, 3, &sample.d);
        DDS_DynamicData_get_string_value(src, 4, &sample.e);
        {
            char* temp = NULL;
            DDS_DynamicData_get_string_value(src, 5, &temp);
            if (temp != NULL) {
                size_t len = mbstowcs(NULL, temp, 0);
                if (len != (size_t)-1) {
                    sample.f = malloc((len + 1) * sizeof(wchar_t));
                    mbstowcs(sample.f, temp, len + 1);
                }
                DDS_String_free(temp);
            }
        }
        DDS_DynamicData_get_int16_value(src, 6, &sample.g);
        DDS_DynamicData_get_uint16_value(src, 7, &sample.h);
        DDS_DynamicData_get_int32_value(src, 8, &sample.i);
        DDS_DynamicData_get_uint32_value(src, 9, &sample.j);
        DDS_DynamicData_get_int64_value(src, 10, &sample.k);
        DDS_DynamicData_get_uint64_value(src, 11, &sample.l);
        DDS_DynamicData_get_float32_value(src, 12, &sample.m);
        DDS_DynamicData_get_float64_value(src, 13, &sample.n);
        return sample;
    }

    static inline DDS_DynamicData* BasicTypes_create_dynamic_sample(const struct BasicTypes* src) {
        DDS_DynamicData* sample = DDS_DynamicData_create(BasicTypes_get_type());
        if (sample != NULL) {
            DDS_DynamicData_set_boolean_value(sample, 0, src->a);
            DDS_DynamicData_set_char8_value(sample, 1, src->b);
            DDS_DynamicData_set_char8_value(sample, 2, (char)src->c);
            DDS_DynamicData_set_uint8_value(sample, 3, src->d);
            DDS_DynamicData_set_string_value(sample, 4, src->e);
            {
                if (src->f != NULL) {
                    size_t len = wcstombs(NULL, src->f, 0);
                    if (len != (size_t)-1) {
                        char* temp = malloc(len + 1);
                        wcstombs(temp, src->f, len + 1);
                        DDS_DynamicData_set_string_value(sample, 5, temp);
                        free(temp);
                    }
                }
            }
            DDS_DynamicData_set_int16_value(sample, 6, src->g);
            DDS_DynamicData_set_uint16_value(sample, 7, src->h);
            DDS_DynamicData_set_int32_value(sample, 8, src->i);
            DDS_DynamicData_set_uint32_value(sample, 9, src->j);
            DDS_DynamicData_set_int64_value(sample, 10, src->k);
            DDS_DynamicData_set_uint64_value(sample, 11, src->l);
            DDS_DynamicData_set_float32_value(sample, 12, src->m);
            DDS_DynamicData_set_float64_value(sample, 13, src->n);
        }
        return sample;
    }

    static inline void BasicTypes_free_sample(struct BasicTypes* sample) {
        if (sample != NULL) {
        DDS_String_free(sample->e);
        free(sample->f);
        }
    }

    static inline DDS_ReturnCode BasicTypes_dds_datawriter_write(DDS_DataWriter* writer, const struct BasicTypes* data, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = BasicTypes_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_write(writer, sample, handle);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode BasicTypes_dds_datawriter_write_w_timestamp(DDS_DataWriter* writer, const struct BasicTypes* data, const DDS_InstanceHandle_t* handle, struct DDS_Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = BasicTypes_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_write_w_timestamp(writer, sample, handle, source_timestamp);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode BasicTypes_dds_datawriter_register_instance(DDS_DataWriter* writer, const struct BasicTypes* data, DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = BasicTypes_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_register_instance(writer, sample, handle);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode BasicTypes_dds_datawriter_register_instance_w_timestamp(DDS_DataWriter* writer, const struct BasicTypes* data, struct DDS_Time_t source_timestamp, DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = BasicTypes_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_register_instance_w_timestamp(writer, sample, source_timestamp, handle);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode BasicTypes_dds_datawriter_unregister_instance(DDS_DataWriter* writer, const struct BasicTypes* data, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = BasicTypes_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_unregister_instance(writer, sample, handle);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode BasicTypes_dds_datawriter_unregister_instance_w_timestamp(DDS_DataWriter* writer, const struct BasicTypes* data, const DDS_InstanceHandle_t* handle, struct DDS_Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = BasicTypes_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_unregister_instance_w_timestamp(writer, sample, handle, source_timestamp);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode BasicTypes_dds_datawriter_dispose(DDS_DataWriter* writer, const struct BasicTypes* data, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = BasicTypes_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_dispose(writer, sample, handle);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode BasicTypes_dds_datawriter_dispose_w_timestamp(DDS_DataWriter* writer, const struct BasicTypes* data, const DDS_InstanceHandle_t* handle, struct DDS_Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = BasicTypes_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_dispose_w_timestamp(writer, sample, handle, source_timestamp);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode BasicTypes_dds_datawriter_get_key_value(DDS_DataWriter* writer, struct BasicTypes* key_holder, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = BasicTypes_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_get_key_value(writer, sample, handle);
        if (result == DDS_RETCODE_OK) {
            *key_holder = BasicTypes_create_sample(sample);
        }
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode BasicTypes_dds_datawriter_lookup_instance(DDS_DataWriter* writer, const struct BasicTypes* key_holder, DDS_InstanceHandle_t* handle) {
        if (writer == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = BasicTypes_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_lookup_instance(writer, sample, handle);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode BasicTypes_dds_datareader_read(DDS_DataReader* reader, struct BasicTypes* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData** samples = (DDS_DynamicData**)calloc(max_samples, sizeof(DDS_DynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = DDS_DataReader_read(reader, samples, sample_infos, max_samples, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = BasicTypes_create_sample(samples[i]);
                    DDS_DynamicData_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode BasicTypes_dds_datareader_take(DDS_DataReader* reader, struct BasicTypes* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData** samples = (DDS_DynamicData**)calloc(max_samples, sizeof(DDS_DynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = DDS_DataReader_take(reader, samples, sample_infos, max_samples, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = BasicTypes_create_sample(samples[i]);
                    DDS_DynamicData_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode BasicTypes_dds_datareader_read_next_sample(DDS_DataReader* reader, struct BasicTypes* data_value, struct DDS_SampleInfo* sample_info) {
        if (reader == NULL || data_value == NULL || sample_info == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = NULL;
        DDS_ReturnCode result = DDS_DataReader_read_next_sample(reader, &sample, sample_info);
        if (result == DDS_RETCODE_OK) {
            if (sample != NULL) {
                *data_value = BasicTypes_create_sample(sample);
                DDS_DynamicData_free(sample);
            }
        }
        return result;
    }

    static inline DDS_ReturnCode BasicTypes_dds_datareader_take_next_sample(DDS_DataReader* reader, struct BasicTypes* data_value, struct DDS_SampleInfo* sample_info) {
        if (reader == NULL || data_value == NULL || sample_info == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = NULL;
        DDS_ReturnCode result = DDS_DataReader_take_next_sample(reader, &sample, sample_info);
        if (result == DDS_RETCODE_OK) {
            if (sample != NULL) {
                *data_value = BasicTypes_create_sample(sample);
                DDS_DynamicData_free(sample);
            }
        }
        return result;
    }

    static inline DDS_ReturnCode BasicTypes_dds_datareader_read_instance(DDS_DataReader* reader, struct BasicTypes* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* a_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || a_handle == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData** samples = (DDS_DynamicData**)calloc(max_samples, sizeof(DDS_DynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = DDS_DataReader_read_instance(reader, samples, sample_infos, max_samples, a_handle, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = BasicTypes_create_sample(samples[i]);
                    DDS_DynamicData_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode BasicTypes_dds_datareader_take_instance(DDS_DataReader* reader, struct BasicTypes* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* a_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || a_handle == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData** samples = (DDS_DynamicData**)calloc(max_samples, sizeof(DDS_DynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = DDS_DataReader_take_instance(reader, samples, sample_infos, max_samples, a_handle, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = BasicTypes_create_sample(samples[i]);
                    DDS_DynamicData_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode BasicTypes_dds_datareader_read_next_instance(DDS_DataReader* reader, struct BasicTypes* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* previous_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || previous_handle == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData** samples = (DDS_DynamicData**)calloc(max_samples, sizeof(DDS_DynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = DDS_DataReader_read_next_instance(reader, samples, sample_infos, max_samples, previous_handle, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = BasicTypes_create_sample(samples[i]);
                    DDS_DynamicData_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode BasicTypes_dds_datareader_take_next_instance(DDS_DataReader* reader, struct BasicTypes* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* previous_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || previous_handle == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData** samples = (DDS_DynamicData**)calloc(max_samples, sizeof(DDS_DynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = DDS_DataReader_take_next_instance(reader, samples, sample_infos, max_samples, previous_handle, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = BasicTypes_create_sample(samples[i]);
                    DDS_DynamicData_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode BasicTypes_dds_datareader_return_loan(DDS_DataReader* reader, struct BasicTypes* data_values, struct DDS_SampleInfo* sample_infos) {
        return DDS_DataReader_return_loan(reader, NULL, sample_infos);
    }

    static inline DDS_ReturnCode BasicTypes_dds_datareader_get_key_value(DDS_DataReader* reader, struct BasicTypes* key_holder, const DDS_InstanceHandle_t* handle) {
        if (reader == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = BasicTypes_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataReader_get_key_value(reader, sample, handle);
        if (result == DDS_RETCODE_OK) {
            *key_holder = BasicTypes_create_sample(sample);
        }
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode BasicTypes_dds_datareader_lookup_instance(DDS_DataReader* reader, const struct BasicTypes* key_holder, DDS_InstanceHandle_t* handle) {
        if (reader == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = BasicTypes_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataReader_lookup_instance(reader, sample, handle);
        DDS_DynamicData_free(sample);
        return result;
    }
"###;

    let result = dust_dds_gen::compile_idl_c(idl_file).unwrap();

    assert_eq!(result, expected);
}
