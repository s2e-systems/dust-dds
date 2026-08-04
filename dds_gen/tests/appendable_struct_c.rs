use std::path::Path;

#[test]
fn appendable_struct() {
    let idl_file = Path::new("tests/appendable_struct.idl");
    let expected = r###"
    #include <stdbool.h>
    #include <stdint.h>
    #include <stddef.h>
    #include <stdlib.h>
    #include <string.h>
    #include "dust_dds.h"

    struct Point {
        double x;
        double y;
    };

    static inline const DDS_DynamicType* Point_get_type(void) {
        static const DDS_DynamicType* type = NULL;
        if (type == NULL) {
            DDS_TypeDescriptor descriptor = {
                .kind = DDS_TYPE_KIND_STRUCTURE,
                .name = "Point",
                .base_type = NULL,
                .discriminator_type = NULL,
                .bound = NULL,
                .element_type = NULL,
                .key_element_type = NULL,
                .extensibility_kind = DDS_EXTENSIBILITY_KIND_APPENDABLE,
                .is_nested = false
            };
            DDS_DynamicTypeBuilder* builder = DDS_DynamicTypeBuilderFactory_create_type(&descriptor);
            {
                DDS_MemberDescriptor member = {
                    .name = "x",
                    .id = 0,
                    .type = DDS_DynamicType_get_primitive_type(DDS_TYPE_KIND_FLOAT64),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                DDS_DynamicTypeBuilder_add_member(builder, &member);
            }
            {
                DDS_MemberDescriptor member = {
                    .name = "y",
                    .id = 1,
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

    static inline struct Point Point_create_sample(DDS_DynamicData* src) {
        struct Point sample;
        memset(&sample, 0, sizeof(sample));
        DDS_DynamicData_get_float64_value(src, 0, &sample.x);
        DDS_DynamicData_get_float64_value(src, 1, &sample.y);
        return sample;
    }

    static inline DDS_DynamicData* Point_create_dynamic_sample(const struct Point* src) {
        DDS_DynamicData* sample = DDS_DynamicData_create(Point_get_type());
        if (sample != NULL) {
            DDS_DynamicData_set_float64_value(sample, 0, src->x);
            DDS_DynamicData_set_float64_value(sample, 1, src->y);
        }
        return sample;
    }

    static inline void Point_free_sample(struct Point* sample) {
        if (sample != NULL) {
        }
    }

    static inline DDS_ReturnCode PointDataWriter_write(DDS_DataWriter* writer, const struct Point* data, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_write(writer, sample, handle);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode PointDataWriter_write_w_timestamp(DDS_DataWriter* writer, const struct Point* data, const DDS_InstanceHandle_t* handle, struct DDS_Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_write_w_timestamp(writer, sample, handle, source_timestamp);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode PointDataWriter_register_instance(DDS_DataWriter* writer, const struct Point* data, DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_register_instance(writer, sample, handle);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode PointDataWriter_register_instance_w_timestamp(DDS_DataWriter* writer, const struct Point* data, struct DDS_Time_t source_timestamp, DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_register_instance_w_timestamp(writer, sample, source_timestamp, handle);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode PointDataWriter_unregister_instance(DDS_DataWriter* writer, const struct Point* data, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_unregister_instance(writer, sample, handle);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode PointDataWriter_unregister_instance_w_timestamp(DDS_DataWriter* writer, const struct Point* data, const DDS_InstanceHandle_t* handle, struct DDS_Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_unregister_instance_w_timestamp(writer, sample, handle, source_timestamp);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode PointDataWriter_dispose(DDS_DataWriter* writer, const struct Point* data, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_dispose(writer, sample, handle);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode PointDataWriter_dispose_w_timestamp(DDS_DataWriter* writer, const struct Point* data, const DDS_InstanceHandle_t* handle, struct DDS_Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_dispose_w_timestamp(writer, sample, handle, source_timestamp);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode PointDataWriter_get_key_value(DDS_DataWriter* writer, struct Point* key_holder, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = Point_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_get_key_value(writer, sample, handle);
        if (result == DDS_RETCODE_OK) {
            *key_holder = Point_create_sample(sample);
        }
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode PointDataWriter_lookup_instance(DDS_DataWriter* writer, const struct Point* key_holder, DDS_InstanceHandle_t* handle) {
        if (writer == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = Point_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_lookup_instance(writer, sample, handle);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode PointDataReader_read(DDS_DataReader* reader, struct Point* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = Point_create_sample(samples[i]);
                    DDS_DynamicData_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode PointDataReader_take(DDS_DataReader* reader, struct Point* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = Point_create_sample(samples[i]);
                    DDS_DynamicData_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode PointDataReader_read_next_sample(DDS_DataReader* reader, struct Point* data_value, struct DDS_SampleInfo* sample_info) {
        if (reader == NULL || data_value == NULL || sample_info == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = NULL;
        DDS_ReturnCode result = DDS_DataReader_read_next_sample(reader, &sample, sample_info);
        if (result == DDS_RETCODE_OK) {
            if (sample != NULL) {
                *data_value = Point_create_sample(sample);
                DDS_DynamicData_free(sample);
            }
        }
        return result;
    }

    static inline DDS_ReturnCode PointDataReader_take_next_sample(DDS_DataReader* reader, struct Point* data_value, struct DDS_SampleInfo* sample_info) {
        if (reader == NULL || data_value == NULL || sample_info == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = NULL;
        DDS_ReturnCode result = DDS_DataReader_take_next_sample(reader, &sample, sample_info);
        if (result == DDS_RETCODE_OK) {
            if (sample != NULL) {
                *data_value = Point_create_sample(sample);
                DDS_DynamicData_free(sample);
            }
        }
        return result;
    }

    static inline DDS_ReturnCode PointDataReader_read_instance(DDS_DataReader* reader, struct Point* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* a_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = Point_create_sample(samples[i]);
                    DDS_DynamicData_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode PointDataReader_take_instance(DDS_DataReader* reader, struct Point* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* a_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = Point_create_sample(samples[i]);
                    DDS_DynamicData_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode PointDataReader_read_next_instance(DDS_DataReader* reader, struct Point* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* previous_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = Point_create_sample(samples[i]);
                    DDS_DynamicData_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode PointDataReader_take_next_instance(DDS_DataReader* reader, struct Point* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* previous_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = Point_create_sample(samples[i]);
                    DDS_DynamicData_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode PointDataReader_return_loan(DDS_DataReader* reader, struct Point* data_values, struct DDS_SampleInfo* sample_infos) {
        return DDS_DataReader_return_loan(reader, NULL, sample_infos);
    }

    static inline DDS_ReturnCode PointDataReader_get_key_value(DDS_DataReader* reader, struct Point* key_holder, const DDS_InstanceHandle_t* handle) {
        if (reader == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = Point_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataReader_get_key_value(reader, sample, handle);
        if (result == DDS_RETCODE_OK) {
            *key_holder = Point_create_sample(sample);
        }
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode PointDataReader_lookup_instance(DDS_DataReader* reader, const struct Point* key_holder, DDS_InstanceHandle_t* handle) {
        if (reader == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = Point_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataReader_lookup_instance(reader, sample, handle);
        DDS_DynamicData_free(sample);
        return result;
    }
    struct Data {
        int16_t id;
        double x;
    };

    static inline const DDS_DynamicType* Data_get_type(void) {
        static const DDS_DynamicType* type = NULL;
        if (type == NULL) {
            DDS_TypeDescriptor descriptor = {
                .kind = DDS_TYPE_KIND_STRUCTURE,
                .name = "Data",
                .base_type = NULL,
                .discriminator_type = NULL,
                .bound = NULL,
                .element_type = NULL,
                .key_element_type = NULL,
                .extensibility_kind = DDS_EXTENSIBILITY_KIND_MUTABLE,
                .is_nested = false
            };
            DDS_DynamicTypeBuilder* builder = DDS_DynamicTypeBuilderFactory_create_type(&descriptor);
            {
                DDS_MemberDescriptor member = {
                    .name = "id",
                    .id = 0,
                    .type = DDS_DynamicType_get_primitive_type(DDS_TYPE_KIND_INT16),
                    .is_key = true,
                    .is_optional = false,
                    .is_must_understand = true
                };
                DDS_DynamicTypeBuilder_add_member(builder, &member);
            }
            {
                DDS_MemberDescriptor member = {
                    .name = "x",
                    .id = 1,
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

    static inline struct Data Data_create_sample(DDS_DynamicData* src) {
        struct Data sample;
        memset(&sample, 0, sizeof(sample));
        DDS_DynamicData_get_int16_value(src, 0, &sample.id);
        DDS_DynamicData_get_float64_value(src, 1, &sample.x);
        return sample;
    }

    static inline DDS_DynamicData* Data_create_dynamic_sample(const struct Data* src) {
        DDS_DynamicData* sample = DDS_DynamicData_create(Data_get_type());
        if (sample != NULL) {
            DDS_DynamicData_set_int16_value(sample, 0, src->id);
            DDS_DynamicData_set_float64_value(sample, 1, src->x);
        }
        return sample;
    }

    static inline void Data_free_sample(struct Data* sample) {
        if (sample != NULL) {
        }
    }

    static inline DDS_ReturnCode DataDataWriter_write(DDS_DataWriter* writer, const struct Data* data, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = Data_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_write(writer, sample, handle);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode DataDataWriter_write_w_timestamp(DDS_DataWriter* writer, const struct Data* data, const DDS_InstanceHandle_t* handle, struct DDS_Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = Data_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_write_w_timestamp(writer, sample, handle, source_timestamp);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode DataDataWriter_register_instance(DDS_DataWriter* writer, const struct Data* data, DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = Data_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_register_instance(writer, sample, handle);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode DataDataWriter_register_instance_w_timestamp(DDS_DataWriter* writer, const struct Data* data, struct DDS_Time_t source_timestamp, DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = Data_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_register_instance_w_timestamp(writer, sample, source_timestamp, handle);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode DataDataWriter_unregister_instance(DDS_DataWriter* writer, const struct Data* data, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = Data_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_unregister_instance(writer, sample, handle);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode DataDataWriter_unregister_instance_w_timestamp(DDS_DataWriter* writer, const struct Data* data, const DDS_InstanceHandle_t* handle, struct DDS_Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = Data_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_unregister_instance_w_timestamp(writer, sample, handle, source_timestamp);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode DataDataWriter_dispose(DDS_DataWriter* writer, const struct Data* data, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = Data_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_dispose(writer, sample, handle);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode DataDataWriter_dispose_w_timestamp(DDS_DataWriter* writer, const struct Data* data, const DDS_InstanceHandle_t* handle, struct DDS_Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = Data_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_dispose_w_timestamp(writer, sample, handle, source_timestamp);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode DataDataWriter_get_key_value(DDS_DataWriter* writer, struct Data* key_holder, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = Data_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_get_key_value(writer, sample, handle);
        if (result == DDS_RETCODE_OK) {
            *key_holder = Data_create_sample(sample);
        }
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode DataDataWriter_lookup_instance(DDS_DataWriter* writer, const struct Data* key_holder, DDS_InstanceHandle_t* handle) {
        if (writer == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = Data_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_lookup_instance(writer, sample, handle);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode DataDataReader_read(DDS_DataReader* reader, struct Data* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = Data_create_sample(samples[i]);
                    DDS_DynamicData_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode DataDataReader_take(DDS_DataReader* reader, struct Data* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = Data_create_sample(samples[i]);
                    DDS_DynamicData_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode DataDataReader_read_next_sample(DDS_DataReader* reader, struct Data* data_value, struct DDS_SampleInfo* sample_info) {
        if (reader == NULL || data_value == NULL || sample_info == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = NULL;
        DDS_ReturnCode result = DDS_DataReader_read_next_sample(reader, &sample, sample_info);
        if (result == DDS_RETCODE_OK) {
            if (sample != NULL) {
                *data_value = Data_create_sample(sample);
                DDS_DynamicData_free(sample);
            }
        }
        return result;
    }

    static inline DDS_ReturnCode DataDataReader_take_next_sample(DDS_DataReader* reader, struct Data* data_value, struct DDS_SampleInfo* sample_info) {
        if (reader == NULL || data_value == NULL || sample_info == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = NULL;
        DDS_ReturnCode result = DDS_DataReader_take_next_sample(reader, &sample, sample_info);
        if (result == DDS_RETCODE_OK) {
            if (sample != NULL) {
                *data_value = Data_create_sample(sample);
                DDS_DynamicData_free(sample);
            }
        }
        return result;
    }

    static inline DDS_ReturnCode DataDataReader_read_instance(DDS_DataReader* reader, struct Data* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* a_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = Data_create_sample(samples[i]);
                    DDS_DynamicData_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode DataDataReader_take_instance(DDS_DataReader* reader, struct Data* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* a_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = Data_create_sample(samples[i]);
                    DDS_DynamicData_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode DataDataReader_read_next_instance(DDS_DataReader* reader, struct Data* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* previous_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = Data_create_sample(samples[i]);
                    DDS_DynamicData_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode DataDataReader_take_next_instance(DDS_DataReader* reader, struct Data* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* previous_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = Data_create_sample(samples[i]);
                    DDS_DynamicData_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode DataDataReader_return_loan(DDS_DataReader* reader, struct Data* data_values, struct DDS_SampleInfo* sample_infos) {
        return DDS_DataReader_return_loan(reader, NULL, sample_infos);
    }

    static inline DDS_ReturnCode DataDataReader_get_key_value(DDS_DataReader* reader, struct Data* key_holder, const DDS_InstanceHandle_t* handle) {
        if (reader == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = Data_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataReader_get_key_value(reader, sample, handle);
        if (result == DDS_RETCODE_OK) {
            *key_holder = Data_create_sample(sample);
        }
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode DataDataReader_lookup_instance(DDS_DataReader* reader, const struct Data* key_holder, DDS_InstanceHandle_t* handle) {
        if (reader == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = Data_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataReader_lookup_instance(reader, sample, handle);
        DDS_DynamicData_free(sample);
        return result;
    }
    struct MultiDimensionalPoint {
        double x;
        double y;
        double z;
    };

    static inline const DDS_DynamicType* MultiDimensionalPoint_get_type(void) {
        static const DDS_DynamicType* type = NULL;
        if (type == NULL) {
            DDS_TypeDescriptor descriptor = {
                .kind = DDS_TYPE_KIND_STRUCTURE,
                .name = "MultiDimensionalPoint",
                .base_type = NULL,
                .discriminator_type = NULL,
                .bound = NULL,
                .element_type = NULL,
                .key_element_type = NULL,
                .extensibility_kind = DDS_EXTENSIBILITY_KIND_APPENDABLE,
                .is_nested = false
            };
            DDS_DynamicTypeBuilder* builder = DDS_DynamicTypeBuilderFactory_create_type(&descriptor);
            {
                DDS_MemberDescriptor member = {
                    .name = "x",
                    .id = 0,
                    .type = DDS_DynamicType_get_primitive_type(DDS_TYPE_KIND_FLOAT64),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                DDS_DynamicTypeBuilder_add_member(builder, &member);
            }
            {
                DDS_MemberDescriptor member = {
                    .name = "y",
                    .id = 1,
                    .type = DDS_DynamicType_get_primitive_type(DDS_TYPE_KIND_FLOAT64),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                DDS_DynamicTypeBuilder_add_member(builder, &member);
            }
            {
                DDS_MemberDescriptor member = {
                    .name = "z",
                    .id = 2,
                    .type = DDS_DynamicType_get_primitive_type(DDS_TYPE_KIND_FLOAT64),
                    .is_key = false,
                    .is_optional = true,
                    .is_must_understand = false
                };
                DDS_DynamicTypeBuilder_add_member(builder, &member);
            }
            type = DDS_DynamicTypeBuilder_build(builder);
        }
        return type;
    }

    static inline struct MultiDimensionalPoint MultiDimensionalPoint_create_sample(DDS_DynamicData* src) {
        struct MultiDimensionalPoint sample;
        memset(&sample, 0, sizeof(sample));
        DDS_DynamicData_get_float64_value(src, 0, &sample.x);
        DDS_DynamicData_get_float64_value(src, 1, &sample.y);
        DDS_DynamicData_get_float64_value(src, 2, &sample.z);
        return sample;
    }

    static inline DDS_DynamicData* MultiDimensionalPoint_create_dynamic_sample(const struct MultiDimensionalPoint* src) {
        DDS_DynamicData* sample = DDS_DynamicData_create(MultiDimensionalPoint_get_type());
        if (sample != NULL) {
            DDS_DynamicData_set_float64_value(sample, 0, src->x);
            DDS_DynamicData_set_float64_value(sample, 1, src->y);
            DDS_DynamicData_set_float64_value(sample, 2, src->z);
        }
        return sample;
    }

    static inline void MultiDimensionalPoint_free_sample(struct MultiDimensionalPoint* sample) {
        if (sample != NULL) {
        }
    }

    static inline DDS_ReturnCode MultiDimensionalPointDataWriter_write(DDS_DataWriter* writer, const struct MultiDimensionalPoint* data, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_write(writer, sample, handle);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPointDataWriter_write_w_timestamp(DDS_DataWriter* writer, const struct MultiDimensionalPoint* data, const DDS_InstanceHandle_t* handle, struct DDS_Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_write_w_timestamp(writer, sample, handle, source_timestamp);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPointDataWriter_register_instance(DDS_DataWriter* writer, const struct MultiDimensionalPoint* data, DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_register_instance(writer, sample, handle);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPointDataWriter_register_instance_w_timestamp(DDS_DataWriter* writer, const struct MultiDimensionalPoint* data, struct DDS_Time_t source_timestamp, DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_register_instance_w_timestamp(writer, sample, source_timestamp, handle);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPointDataWriter_unregister_instance(DDS_DataWriter* writer, const struct MultiDimensionalPoint* data, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_unregister_instance(writer, sample, handle);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPointDataWriter_unregister_instance_w_timestamp(DDS_DataWriter* writer, const struct MultiDimensionalPoint* data, const DDS_InstanceHandle_t* handle, struct DDS_Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_unregister_instance_w_timestamp(writer, sample, handle, source_timestamp);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPointDataWriter_dispose(DDS_DataWriter* writer, const struct MultiDimensionalPoint* data, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_dispose(writer, sample, handle);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPointDataWriter_dispose_w_timestamp(DDS_DataWriter* writer, const struct MultiDimensionalPoint* data, const DDS_InstanceHandle_t* handle, struct DDS_Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_dispose_w_timestamp(writer, sample, handle, source_timestamp);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPointDataWriter_get_key_value(DDS_DataWriter* writer, struct MultiDimensionalPoint* key_holder, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_get_key_value(writer, sample, handle);
        if (result == DDS_RETCODE_OK) {
            *key_holder = MultiDimensionalPoint_create_sample(sample);
        }
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPointDataWriter_lookup_instance(DDS_DataWriter* writer, const struct MultiDimensionalPoint* key_holder, DDS_InstanceHandle_t* handle) {
        if (writer == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataWriter_lookup_instance(writer, sample, handle);
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPointDataReader_read(DDS_DataReader* reader, struct MultiDimensionalPoint* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = MultiDimensionalPoint_create_sample(samples[i]);
                    DDS_DynamicData_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPointDataReader_take(DDS_DataReader* reader, struct MultiDimensionalPoint* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = MultiDimensionalPoint_create_sample(samples[i]);
                    DDS_DynamicData_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPointDataReader_read_next_sample(DDS_DataReader* reader, struct MultiDimensionalPoint* data_value, struct DDS_SampleInfo* sample_info) {
        if (reader == NULL || data_value == NULL || sample_info == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = NULL;
        DDS_ReturnCode result = DDS_DataReader_read_next_sample(reader, &sample, sample_info);
        if (result == DDS_RETCODE_OK) {
            if (sample != NULL) {
                *data_value = MultiDimensionalPoint_create_sample(sample);
                DDS_DynamicData_free(sample);
            }
        }
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPointDataReader_take_next_sample(DDS_DataReader* reader, struct MultiDimensionalPoint* data_value, struct DDS_SampleInfo* sample_info) {
        if (reader == NULL || data_value == NULL || sample_info == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = NULL;
        DDS_ReturnCode result = DDS_DataReader_take_next_sample(reader, &sample, sample_info);
        if (result == DDS_RETCODE_OK) {
            if (sample != NULL) {
                *data_value = MultiDimensionalPoint_create_sample(sample);
                DDS_DynamicData_free(sample);
            }
        }
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPointDataReader_read_instance(DDS_DataReader* reader, struct MultiDimensionalPoint* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* a_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = MultiDimensionalPoint_create_sample(samples[i]);
                    DDS_DynamicData_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPointDataReader_take_instance(DDS_DataReader* reader, struct MultiDimensionalPoint* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* a_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = MultiDimensionalPoint_create_sample(samples[i]);
                    DDS_DynamicData_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPointDataReader_read_next_instance(DDS_DataReader* reader, struct MultiDimensionalPoint* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* previous_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = MultiDimensionalPoint_create_sample(samples[i]);
                    DDS_DynamicData_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPointDataReader_take_next_instance(DDS_DataReader* reader, struct MultiDimensionalPoint* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* previous_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = MultiDimensionalPoint_create_sample(samples[i]);
                    DDS_DynamicData_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPointDataReader_return_loan(DDS_DataReader* reader, struct MultiDimensionalPoint* data_values, struct DDS_SampleInfo* sample_infos) {
        return DDS_DataReader_return_loan(reader, NULL, sample_infos);
    }

    static inline DDS_ReturnCode MultiDimensionalPointDataReader_get_key_value(DDS_DataReader* reader, struct MultiDimensionalPoint* key_holder, const DDS_InstanceHandle_t* handle) {
        if (reader == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_DataReader_get_key_value(reader, sample, handle);
        if (result == DDS_RETCODE_OK) {
            *key_holder = MultiDimensionalPoint_create_sample(sample);
        }
        DDS_DynamicData_free(sample);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPointDataReader_lookup_instance(DDS_DataReader* reader, const struct MultiDimensionalPoint* key_holder, DDS_InstanceHandle_t* handle) {
        if (reader == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(key_holder);
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
