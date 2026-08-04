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

    static inline const DDS_DustDdsDynamicType* Point_get_type(void) {
        static const DDS_DustDdsDynamicType* type = NULL;
        if (type == NULL) {
            DDS_DustDdsTypeDescriptor descriptor = {
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
            DDS_DustDdsDynamicTypeBuilder* builder = DDS_dynamic_type_builder_factory_create_type(&descriptor);
            {
                DDS_DustDdsMemberDescriptor member = {
                    .name = "x",
                    .id = 0,
                    .type = DDS_dynamic_type_get_primitive_type(DDS_TYPE_KIND_FLOAT64),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                DDS_dynamic_type_builder_add_member(builder, &member);
            }
            {
                DDS_DustDdsMemberDescriptor member = {
                    .name = "y",
                    .id = 1,
                    .type = DDS_dynamic_type_get_primitive_type(DDS_TYPE_KIND_FLOAT64),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                DDS_dynamic_type_builder_add_member(builder, &member);
            }
            type = DDS_dynamic_type_builder_build(builder);
        }
        return type;
    }

    static inline struct Point Point_create_sample(DDS_DustDdsDynamicData* src) {
        struct Point sample;
        memset(&sample, 0, sizeof(sample));
        DDS_dynamic_data_get_float64_value(src, 0, &sample.x);
        DDS_dynamic_data_get_float64_value(src, 1, &sample.y);
        return sample;
    }

    static inline DDS_DustDdsDynamicData* Point_create_dynamic_sample(const struct Point* src) {
        DDS_DustDdsDynamicData* sample = DDS_dynamic_data_create(Point_get_type());
        if (sample != NULL) {
            DDS_dynamic_data_set_float64_value(sample, 0, src->x);
            DDS_dynamic_data_set_float64_value(sample, 1, src->y);
        }
        return sample;
    }

    static inline void Point_free_sample(struct Point* sample) {
        if (sample != NULL) {
        }
    }

    static inline DDS_ReturnCode Point_dds_datawriter_write(DDS_DustDdsDataWriter* writer, const struct Point* data, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datawriter_write(writer, sample, handle);
        DDS_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datawriter_write_w_timestamp(DDS_DustDdsDataWriter* writer, const struct Point* data, const DDS_InstanceHandle_t* handle, struct DDS_Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datawriter_write_w_timestamp(writer, sample, handle, source_timestamp);
        DDS_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datawriter_register_instance(DDS_DustDdsDataWriter* writer, const struct Point* data, DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datawriter_register_instance(writer, sample, handle);
        DDS_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datawriter_register_instance_w_timestamp(DDS_DustDdsDataWriter* writer, const struct Point* data, struct DDS_Time_t source_timestamp, DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datawriter_register_instance_w_timestamp(writer, sample, source_timestamp, handle);
        DDS_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datawriter_unregister_instance(DDS_DustDdsDataWriter* writer, const struct Point* data, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datawriter_unregister_instance(writer, sample, handle);
        DDS_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datawriter_unregister_instance_w_timestamp(DDS_DustDdsDataWriter* writer, const struct Point* data, const DDS_InstanceHandle_t* handle, struct DDS_Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datawriter_unregister_instance_w_timestamp(writer, sample, handle, source_timestamp);
        DDS_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datawriter_dispose(DDS_DustDdsDataWriter* writer, const struct Point* data, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datawriter_dispose(writer, sample, handle);
        DDS_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datawriter_dispose_w_timestamp(DDS_DustDdsDataWriter* writer, const struct Point* data, const DDS_InstanceHandle_t* handle, struct DDS_Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datawriter_dispose_w_timestamp(writer, sample, handle, source_timestamp);
        DDS_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datawriter_get_key_value(DDS_DustDdsDataWriter* writer, struct Point* key_holder, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Point_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datawriter_get_key_value(writer, sample, handle);
        if (result == DDS_RETCODE_OK) {
            *key_holder = Point_create_sample(sample);
        }
        DDS_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datawriter_lookup_instance(DDS_DustDdsDataWriter* writer, const struct Point* key_holder, DDS_InstanceHandle_t* handle) {
        if (writer == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Point_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datawriter_lookup_instance(writer, sample, handle);
        DDS_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datareader_read(DDS_DustDdsDataReader* reader, struct Point* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = DDS_datareader_read(reader, samples, sample_infos, max_samples, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = Point_create_sample(samples[i]);
                    DDS_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datareader_take(DDS_DustDdsDataReader* reader, struct Point* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = DDS_datareader_take(reader, samples, sample_infos, max_samples, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = Point_create_sample(samples[i]);
                    DDS_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datareader_read_next_sample(DDS_DustDdsDataReader* reader, struct Point* data_value, struct DDS_SampleInfo* sample_info) {
        if (reader == NULL || data_value == NULL || sample_info == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = NULL;
        DDS_ReturnCode result = DDS_datareader_read_next_sample(reader, &sample, sample_info);
        if (result == DDS_RETCODE_OK) {
            if (sample != NULL) {
                *data_value = Point_create_sample(sample);
                DDS_dynamic_data_free(sample);
            }
        }
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datareader_take_next_sample(DDS_DustDdsDataReader* reader, struct Point* data_value, struct DDS_SampleInfo* sample_info) {
        if (reader == NULL || data_value == NULL || sample_info == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = NULL;
        DDS_ReturnCode result = DDS_datareader_take_next_sample(reader, &sample, sample_info);
        if (result == DDS_RETCODE_OK) {
            if (sample != NULL) {
                *data_value = Point_create_sample(sample);
                DDS_dynamic_data_free(sample);
            }
        }
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datareader_read_instance(DDS_DustDdsDataReader* reader, struct Point* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* a_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || a_handle == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = DDS_datareader_read_instance(reader, samples, sample_infos, max_samples, a_handle, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = Point_create_sample(samples[i]);
                    DDS_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datareader_take_instance(DDS_DustDdsDataReader* reader, struct Point* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* a_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || a_handle == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = DDS_datareader_take_instance(reader, samples, sample_infos, max_samples, a_handle, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = Point_create_sample(samples[i]);
                    DDS_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datareader_read_next_instance(DDS_DustDdsDataReader* reader, struct Point* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* previous_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || previous_handle == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = DDS_datareader_read_next_instance(reader, samples, sample_infos, max_samples, previous_handle, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = Point_create_sample(samples[i]);
                    DDS_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datareader_take_next_instance(DDS_DustDdsDataReader* reader, struct Point* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* previous_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || previous_handle == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = DDS_datareader_take_next_instance(reader, samples, sample_infos, max_samples, previous_handle, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = Point_create_sample(samples[i]);
                    DDS_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datareader_return_loan(DDS_DustDdsDataReader* reader, struct Point* data_values, struct DDS_SampleInfo* sample_infos) {
        return DDS_datareader_return_loan(reader, NULL, sample_infos);
    }

    static inline DDS_ReturnCode Point_dds_datareader_get_key_value(DDS_DustDdsDataReader* reader, struct Point* key_holder, const DDS_InstanceHandle_t* handle) {
        if (reader == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Point_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datareader_get_key_value(reader, sample, handle);
        if (result == DDS_RETCODE_OK) {
            *key_holder = Point_create_sample(sample);
        }
        DDS_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datareader_lookup_instance(DDS_DustDdsDataReader* reader, const struct Point* key_holder, DDS_InstanceHandle_t* handle) {
        if (reader == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Point_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datareader_lookup_instance(reader, sample, handle);
        DDS_dynamic_data_free(sample);
        return result;
    }
    struct Data {
        int16_t id;
        double x;
    };

    static inline const DDS_DustDdsDynamicType* Data_get_type(void) {
        static const DDS_DustDdsDynamicType* type = NULL;
        if (type == NULL) {
            DDS_DustDdsTypeDescriptor descriptor = {
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
            DDS_DustDdsDynamicTypeBuilder* builder = DDS_dynamic_type_builder_factory_create_type(&descriptor);
            {
                DDS_DustDdsMemberDescriptor member = {
                    .name = "id",
                    .id = 0,
                    .type = DDS_dynamic_type_get_primitive_type(DDS_TYPE_KIND_INT16),
                    .is_key = true,
                    .is_optional = false,
                    .is_must_understand = true
                };
                DDS_dynamic_type_builder_add_member(builder, &member);
            }
            {
                DDS_DustDdsMemberDescriptor member = {
                    .name = "x",
                    .id = 1,
                    .type = DDS_dynamic_type_get_primitive_type(DDS_TYPE_KIND_FLOAT64),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                DDS_dynamic_type_builder_add_member(builder, &member);
            }
            type = DDS_dynamic_type_builder_build(builder);
        }
        return type;
    }

    static inline struct Data Data_create_sample(DDS_DustDdsDynamicData* src) {
        struct Data sample;
        memset(&sample, 0, sizeof(sample));
        DDS_dynamic_data_get_int16_value(src, 0, &sample.id);
        DDS_dynamic_data_get_float64_value(src, 1, &sample.x);
        return sample;
    }

    static inline DDS_DustDdsDynamicData* Data_create_dynamic_sample(const struct Data* src) {
        DDS_DustDdsDynamicData* sample = DDS_dynamic_data_create(Data_get_type());
        if (sample != NULL) {
            DDS_dynamic_data_set_int16_value(sample, 0, src->id);
            DDS_dynamic_data_set_float64_value(sample, 1, src->x);
        }
        return sample;
    }

    static inline void Data_free_sample(struct Data* sample) {
        if (sample != NULL) {
        }
    }

    static inline DDS_ReturnCode Data_dds_datawriter_write(DDS_DustDdsDataWriter* writer, const struct Data* data, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Data_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datawriter_write(writer, sample, handle);
        DDS_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Data_dds_datawriter_write_w_timestamp(DDS_DustDdsDataWriter* writer, const struct Data* data, const DDS_InstanceHandle_t* handle, struct DDS_Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Data_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datawriter_write_w_timestamp(writer, sample, handle, source_timestamp);
        DDS_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Data_dds_datawriter_register_instance(DDS_DustDdsDataWriter* writer, const struct Data* data, DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Data_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datawriter_register_instance(writer, sample, handle);
        DDS_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Data_dds_datawriter_register_instance_w_timestamp(DDS_DustDdsDataWriter* writer, const struct Data* data, struct DDS_Time_t source_timestamp, DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Data_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datawriter_register_instance_w_timestamp(writer, sample, source_timestamp, handle);
        DDS_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Data_dds_datawriter_unregister_instance(DDS_DustDdsDataWriter* writer, const struct Data* data, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Data_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datawriter_unregister_instance(writer, sample, handle);
        DDS_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Data_dds_datawriter_unregister_instance_w_timestamp(DDS_DustDdsDataWriter* writer, const struct Data* data, const DDS_InstanceHandle_t* handle, struct DDS_Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Data_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datawriter_unregister_instance_w_timestamp(writer, sample, handle, source_timestamp);
        DDS_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Data_dds_datawriter_dispose(DDS_DustDdsDataWriter* writer, const struct Data* data, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Data_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datawriter_dispose(writer, sample, handle);
        DDS_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Data_dds_datawriter_dispose_w_timestamp(DDS_DustDdsDataWriter* writer, const struct Data* data, const DDS_InstanceHandle_t* handle, struct DDS_Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Data_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datawriter_dispose_w_timestamp(writer, sample, handle, source_timestamp);
        DDS_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Data_dds_datawriter_get_key_value(DDS_DustDdsDataWriter* writer, struct Data* key_holder, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Data_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datawriter_get_key_value(writer, sample, handle);
        if (result == DDS_RETCODE_OK) {
            *key_holder = Data_create_sample(sample);
        }
        DDS_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Data_dds_datawriter_lookup_instance(DDS_DustDdsDataWriter* writer, const struct Data* key_holder, DDS_InstanceHandle_t* handle) {
        if (writer == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Data_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datawriter_lookup_instance(writer, sample, handle);
        DDS_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Data_dds_datareader_read(DDS_DustDdsDataReader* reader, struct Data* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = DDS_datareader_read(reader, samples, sample_infos, max_samples, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = Data_create_sample(samples[i]);
                    DDS_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode Data_dds_datareader_take(DDS_DustDdsDataReader* reader, struct Data* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = DDS_datareader_take(reader, samples, sample_infos, max_samples, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = Data_create_sample(samples[i]);
                    DDS_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode Data_dds_datareader_read_next_sample(DDS_DustDdsDataReader* reader, struct Data* data_value, struct DDS_SampleInfo* sample_info) {
        if (reader == NULL || data_value == NULL || sample_info == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = NULL;
        DDS_ReturnCode result = DDS_datareader_read_next_sample(reader, &sample, sample_info);
        if (result == DDS_RETCODE_OK) {
            if (sample != NULL) {
                *data_value = Data_create_sample(sample);
                DDS_dynamic_data_free(sample);
            }
        }
        return result;
    }

    static inline DDS_ReturnCode Data_dds_datareader_take_next_sample(DDS_DustDdsDataReader* reader, struct Data* data_value, struct DDS_SampleInfo* sample_info) {
        if (reader == NULL || data_value == NULL || sample_info == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = NULL;
        DDS_ReturnCode result = DDS_datareader_take_next_sample(reader, &sample, sample_info);
        if (result == DDS_RETCODE_OK) {
            if (sample != NULL) {
                *data_value = Data_create_sample(sample);
                DDS_dynamic_data_free(sample);
            }
        }
        return result;
    }

    static inline DDS_ReturnCode Data_dds_datareader_read_instance(DDS_DustDdsDataReader* reader, struct Data* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* a_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || a_handle == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = DDS_datareader_read_instance(reader, samples, sample_infos, max_samples, a_handle, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = Data_create_sample(samples[i]);
                    DDS_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode Data_dds_datareader_take_instance(DDS_DustDdsDataReader* reader, struct Data* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* a_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || a_handle == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = DDS_datareader_take_instance(reader, samples, sample_infos, max_samples, a_handle, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = Data_create_sample(samples[i]);
                    DDS_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode Data_dds_datareader_read_next_instance(DDS_DustDdsDataReader* reader, struct Data* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* previous_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || previous_handle == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = DDS_datareader_read_next_instance(reader, samples, sample_infos, max_samples, previous_handle, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = Data_create_sample(samples[i]);
                    DDS_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode Data_dds_datareader_take_next_instance(DDS_DustDdsDataReader* reader, struct Data* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* previous_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || previous_handle == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = DDS_datareader_take_next_instance(reader, samples, sample_infos, max_samples, previous_handle, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = Data_create_sample(samples[i]);
                    DDS_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode Data_dds_datareader_return_loan(DDS_DustDdsDataReader* reader, struct Data* data_values, struct DDS_SampleInfo* sample_infos) {
        return DDS_datareader_return_loan(reader, NULL, sample_infos);
    }

    static inline DDS_ReturnCode Data_dds_datareader_get_key_value(DDS_DustDdsDataReader* reader, struct Data* key_holder, const DDS_InstanceHandle_t* handle) {
        if (reader == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Data_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datareader_get_key_value(reader, sample, handle);
        if (result == DDS_RETCODE_OK) {
            *key_holder = Data_create_sample(sample);
        }
        DDS_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Data_dds_datareader_lookup_instance(DDS_DustDdsDataReader* reader, const struct Data* key_holder, DDS_InstanceHandle_t* handle) {
        if (reader == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Data_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datareader_lookup_instance(reader, sample, handle);
        DDS_dynamic_data_free(sample);
        return result;
    }
    struct MultiDimensionalPoint {
        double x;
        double y;
        double z;
    };

    static inline const DDS_DustDdsDynamicType* MultiDimensionalPoint_get_type(void) {
        static const DDS_DustDdsDynamicType* type = NULL;
        if (type == NULL) {
            DDS_DustDdsTypeDescriptor descriptor = {
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
            DDS_DustDdsDynamicTypeBuilder* builder = DDS_dynamic_type_builder_factory_create_type(&descriptor);
            {
                DDS_DustDdsMemberDescriptor member = {
                    .name = "x",
                    .id = 0,
                    .type = DDS_dynamic_type_get_primitive_type(DDS_TYPE_KIND_FLOAT64),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                DDS_dynamic_type_builder_add_member(builder, &member);
            }
            {
                DDS_DustDdsMemberDescriptor member = {
                    .name = "y",
                    .id = 1,
                    .type = DDS_dynamic_type_get_primitive_type(DDS_TYPE_KIND_FLOAT64),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                DDS_dynamic_type_builder_add_member(builder, &member);
            }
            {
                DDS_DustDdsMemberDescriptor member = {
                    .name = "z",
                    .id = 2,
                    .type = DDS_dynamic_type_get_primitive_type(DDS_TYPE_KIND_FLOAT64),
                    .is_key = false,
                    .is_optional = true,
                    .is_must_understand = false
                };
                DDS_dynamic_type_builder_add_member(builder, &member);
            }
            type = DDS_dynamic_type_builder_build(builder);
        }
        return type;
    }

    static inline struct MultiDimensionalPoint MultiDimensionalPoint_create_sample(DDS_DustDdsDynamicData* src) {
        struct MultiDimensionalPoint sample;
        memset(&sample, 0, sizeof(sample));
        DDS_dynamic_data_get_float64_value(src, 0, &sample.x);
        DDS_dynamic_data_get_float64_value(src, 1, &sample.y);
        DDS_dynamic_data_get_float64_value(src, 2, &sample.z);
        return sample;
    }

    static inline DDS_DustDdsDynamicData* MultiDimensionalPoint_create_dynamic_sample(const struct MultiDimensionalPoint* src) {
        DDS_DustDdsDynamicData* sample = DDS_dynamic_data_create(MultiDimensionalPoint_get_type());
        if (sample != NULL) {
            DDS_dynamic_data_set_float64_value(sample, 0, src->x);
            DDS_dynamic_data_set_float64_value(sample, 1, src->y);
            DDS_dynamic_data_set_float64_value(sample, 2, src->z);
        }
        return sample;
    }

    static inline void MultiDimensionalPoint_free_sample(struct MultiDimensionalPoint* sample) {
        if (sample != NULL) {
        }
    }

    static inline DDS_ReturnCode MultiDimensionalPoint_dds_datawriter_write(DDS_DustDdsDataWriter* writer, const struct MultiDimensionalPoint* data, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datawriter_write(writer, sample, handle);
        DDS_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPoint_dds_datawriter_write_w_timestamp(DDS_DustDdsDataWriter* writer, const struct MultiDimensionalPoint* data, const DDS_InstanceHandle_t* handle, struct DDS_Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datawriter_write_w_timestamp(writer, sample, handle, source_timestamp);
        DDS_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPoint_dds_datawriter_register_instance(DDS_DustDdsDataWriter* writer, const struct MultiDimensionalPoint* data, DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datawriter_register_instance(writer, sample, handle);
        DDS_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPoint_dds_datawriter_register_instance_w_timestamp(DDS_DustDdsDataWriter* writer, const struct MultiDimensionalPoint* data, struct DDS_Time_t source_timestamp, DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datawriter_register_instance_w_timestamp(writer, sample, source_timestamp, handle);
        DDS_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPoint_dds_datawriter_unregister_instance(DDS_DustDdsDataWriter* writer, const struct MultiDimensionalPoint* data, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datawriter_unregister_instance(writer, sample, handle);
        DDS_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPoint_dds_datawriter_unregister_instance_w_timestamp(DDS_DustDdsDataWriter* writer, const struct MultiDimensionalPoint* data, const DDS_InstanceHandle_t* handle, struct DDS_Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datawriter_unregister_instance_w_timestamp(writer, sample, handle, source_timestamp);
        DDS_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPoint_dds_datawriter_dispose(DDS_DustDdsDataWriter* writer, const struct MultiDimensionalPoint* data, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datawriter_dispose(writer, sample, handle);
        DDS_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPoint_dds_datawriter_dispose_w_timestamp(DDS_DustDdsDataWriter* writer, const struct MultiDimensionalPoint* data, const DDS_InstanceHandle_t* handle, struct DDS_Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datawriter_dispose_w_timestamp(writer, sample, handle, source_timestamp);
        DDS_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPoint_dds_datawriter_get_key_value(DDS_DustDdsDataWriter* writer, struct MultiDimensionalPoint* key_holder, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datawriter_get_key_value(writer, sample, handle);
        if (result == DDS_RETCODE_OK) {
            *key_holder = MultiDimensionalPoint_create_sample(sample);
        }
        DDS_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPoint_dds_datawriter_lookup_instance(DDS_DustDdsDataWriter* writer, const struct MultiDimensionalPoint* key_holder, DDS_InstanceHandle_t* handle) {
        if (writer == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datawriter_lookup_instance(writer, sample, handle);
        DDS_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPoint_dds_datareader_read(DDS_DustDdsDataReader* reader, struct MultiDimensionalPoint* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = DDS_datareader_read(reader, samples, sample_infos, max_samples, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = MultiDimensionalPoint_create_sample(samples[i]);
                    DDS_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPoint_dds_datareader_take(DDS_DustDdsDataReader* reader, struct MultiDimensionalPoint* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = DDS_datareader_take(reader, samples, sample_infos, max_samples, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = MultiDimensionalPoint_create_sample(samples[i]);
                    DDS_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPoint_dds_datareader_read_next_sample(DDS_DustDdsDataReader* reader, struct MultiDimensionalPoint* data_value, struct DDS_SampleInfo* sample_info) {
        if (reader == NULL || data_value == NULL || sample_info == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = NULL;
        DDS_ReturnCode result = DDS_datareader_read_next_sample(reader, &sample, sample_info);
        if (result == DDS_RETCODE_OK) {
            if (sample != NULL) {
                *data_value = MultiDimensionalPoint_create_sample(sample);
                DDS_dynamic_data_free(sample);
            }
        }
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPoint_dds_datareader_take_next_sample(DDS_DustDdsDataReader* reader, struct MultiDimensionalPoint* data_value, struct DDS_SampleInfo* sample_info) {
        if (reader == NULL || data_value == NULL || sample_info == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = NULL;
        DDS_ReturnCode result = DDS_datareader_take_next_sample(reader, &sample, sample_info);
        if (result == DDS_RETCODE_OK) {
            if (sample != NULL) {
                *data_value = MultiDimensionalPoint_create_sample(sample);
                DDS_dynamic_data_free(sample);
            }
        }
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPoint_dds_datareader_read_instance(DDS_DustDdsDataReader* reader, struct MultiDimensionalPoint* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* a_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || a_handle == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = DDS_datareader_read_instance(reader, samples, sample_infos, max_samples, a_handle, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = MultiDimensionalPoint_create_sample(samples[i]);
                    DDS_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPoint_dds_datareader_take_instance(DDS_DustDdsDataReader* reader, struct MultiDimensionalPoint* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* a_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || a_handle == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = DDS_datareader_take_instance(reader, samples, sample_infos, max_samples, a_handle, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = MultiDimensionalPoint_create_sample(samples[i]);
                    DDS_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPoint_dds_datareader_read_next_instance(DDS_DustDdsDataReader* reader, struct MultiDimensionalPoint* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* previous_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || previous_handle == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = DDS_datareader_read_next_instance(reader, samples, sample_infos, max_samples, previous_handle, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = MultiDimensionalPoint_create_sample(samples[i]);
                    DDS_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPoint_dds_datareader_take_next_instance(DDS_DustDdsDataReader* reader, struct MultiDimensionalPoint* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* previous_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || previous_handle == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = DDS_datareader_take_next_instance(reader, samples, sample_infos, max_samples, previous_handle, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = MultiDimensionalPoint_create_sample(samples[i]);
                    DDS_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPoint_dds_datareader_return_loan(DDS_DustDdsDataReader* reader, struct MultiDimensionalPoint* data_values, struct DDS_SampleInfo* sample_infos) {
        return DDS_datareader_return_loan(reader, NULL, sample_infos);
    }

    static inline DDS_ReturnCode MultiDimensionalPoint_dds_datareader_get_key_value(DDS_DustDdsDataReader* reader, struct MultiDimensionalPoint* key_holder, const DDS_InstanceHandle_t* handle) {
        if (reader == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datareader_get_key_value(reader, sample, handle);
        if (result == DDS_RETCODE_OK) {
            *key_holder = MultiDimensionalPoint_create_sample(sample);
        }
        DDS_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode MultiDimensionalPoint_dds_datareader_lookup_instance(DDS_DustDdsDataReader* reader, const struct MultiDimensionalPoint* key_holder, DDS_InstanceHandle_t* handle) {
        if (reader == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = DDS_datareader_lookup_instance(reader, sample, handle);
        DDS_dynamic_data_free(sample);
        return result;
    }
"###;

    let result = dust_dds_gen::compile_idl_c(idl_file).unwrap();

    assert_eq!(result, expected);
}
