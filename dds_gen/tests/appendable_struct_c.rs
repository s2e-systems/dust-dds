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

    static inline const DustDdsDynamicType* Point_get_type(void) {
        static const DustDdsDynamicType* type = NULL;
        if (type == NULL) {
            DustDdsTypeDescriptor descriptor = {
                .kind = TYPE_KIND_STRUCTURE,
                .name = "Point",
                .base_type = NULL,
                .discriminator_type = NULL,
                .bound = NULL,
                .element_type = NULL,
                .key_element_type = NULL,
                .extensibility_kind = EXTENSIBILITY_KIND_APPENDABLE,
                .is_nested = false
            };
            DustDdsDynamicTypeBuilder* builder = dds_dynamic_type_builder_factory_create_type(&descriptor);
            {
                DustDdsMemberDescriptor member = {
                    .name = "x",
                    .id = 0,
                    .type = dds_dynamic_type_get_primitive_type(TYPE_KIND_FLOAT64),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                dds_dynamic_type_builder_add_member(builder, &member);
            }
            {
                DustDdsMemberDescriptor member = {
                    .name = "y",
                    .id = 1,
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

    static inline struct Point Point_create_sample(DustDdsDynamicData* src) {
        struct Point sample;
        memset(&sample, 0, sizeof(sample));
        dds_dynamic_data_get_float64_value(src, 0, &sample.x);
        dds_dynamic_data_get_float64_value(src, 1, &sample.y);
        return sample;
    }

    static inline DustDdsDynamicData* Point_create_dynamic_sample(const struct Point* src) {
        DustDdsDynamicData* sample = dds_dynamic_data_create(Point_get_type());
        if (sample != NULL) {
            dds_dynamic_data_set_float64_value(sample, 0, src->x);
            dds_dynamic_data_set_float64_value(sample, 1, src->y);
        }
        return sample;
    }

    static inline void Point_free_sample(struct Point* sample) {
        if (sample != NULL) {
        }
    }

    static inline ReturnCode Point_dds_datawriter_write(DustDdsDataWriter* writer, const struct Point* data, const InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_write(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode Point_dds_datawriter_write_w_timestamp(DustDdsDataWriter* writer, const struct Point* data, const InstanceHandle_t* handle, struct Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_write_w_timestamp(writer, sample, handle, source_timestamp);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode Point_dds_datawriter_register_instance(DustDdsDataWriter* writer, const struct Point* data, InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL || handle == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_register_instance(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode Point_dds_datawriter_register_instance_w_timestamp(DustDdsDataWriter* writer, const struct Point* data, struct Time_t source_timestamp, InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL || handle == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_register_instance_w_timestamp(writer, sample, source_timestamp, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode Point_dds_datawriter_unregister_instance(DustDdsDataWriter* writer, const struct Point* data, const InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_unregister_instance(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode Point_dds_datawriter_unregister_instance_w_timestamp(DustDdsDataWriter* writer, const struct Point* data, const InstanceHandle_t* handle, struct Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_unregister_instance_w_timestamp(writer, sample, handle, source_timestamp);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode Point_dds_datawriter_dispose(DustDdsDataWriter* writer, const struct Point* data, const InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_dispose(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode Point_dds_datawriter_dispose_w_timestamp(DustDdsDataWriter* writer, const struct Point* data, const InstanceHandle_t* handle, struct Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_dispose_w_timestamp(writer, sample, handle, source_timestamp);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode Point_dds_datawriter_get_key_value(DustDdsDataWriter* writer, struct Point* key_holder, const InstanceHandle_t* handle) {
        if (writer == NULL || key_holder == NULL || handle == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = Point_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_get_key_value(writer, sample, handle);
        if (result == RETCODE_OK) {
            *key_holder = Point_create_sample(sample);
        }
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode Point_dds_datawriter_lookup_instance(DustDdsDataWriter* writer, const struct Point* key_holder, InstanceHandle_t* handle) {
        if (writer == NULL || key_holder == NULL || handle == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = Point_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_lookup_instance(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode Point_dds_datareader_read(DustDdsDataReader* reader, struct Point* data_values, struct SampleInfo* sample_infos, int32_t max_samples, SampleStateMask sample_states, ViewStateMask view_states, InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = Point_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline ReturnCode Point_dds_datareader_take(DustDdsDataReader* reader, struct Point* data_values, struct SampleInfo* sample_infos, int32_t max_samples, SampleStateMask sample_states, ViewStateMask view_states, InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = Point_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline ReturnCode Point_dds_datareader_read_next_sample(DustDdsDataReader* reader, struct Point* data_value, struct SampleInfo* sample_info) {
        if (reader == NULL || data_value == NULL || sample_info == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = NULL;
        ReturnCode result = dds_datareader_read_next_sample(reader, &sample, sample_info);
        if (result == RETCODE_OK) {
            if (sample != NULL) {
                *data_value = Point_create_sample(sample);
                dds_dynamic_data_free(sample);
            }
        }
        return result;
    }

    static inline ReturnCode Point_dds_datareader_take_next_sample(DustDdsDataReader* reader, struct Point* data_value, struct SampleInfo* sample_info) {
        if (reader == NULL || data_value == NULL || sample_info == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = NULL;
        ReturnCode result = dds_datareader_take_next_sample(reader, &sample, sample_info);
        if (result == RETCODE_OK) {
            if (sample != NULL) {
                *data_value = Point_create_sample(sample);
                dds_dynamic_data_free(sample);
            }
        }
        return result;
    }

    static inline ReturnCode Point_dds_datareader_read_instance(DustDdsDataReader* reader, struct Point* data_values, struct SampleInfo* sample_infos, int32_t max_samples, const InstanceHandle_t* a_handle, SampleStateMask sample_states, ViewStateMask view_states, InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = Point_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline ReturnCode Point_dds_datareader_take_instance(DustDdsDataReader* reader, struct Point* data_values, struct SampleInfo* sample_infos, int32_t max_samples, const InstanceHandle_t* a_handle, SampleStateMask sample_states, ViewStateMask view_states, InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = Point_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline ReturnCode Point_dds_datareader_read_next_instance(DustDdsDataReader* reader, struct Point* data_values, struct SampleInfo* sample_infos, int32_t max_samples, const InstanceHandle_t* previous_handle, SampleStateMask sample_states, ViewStateMask view_states, InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = Point_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline ReturnCode Point_dds_datareader_take_next_instance(DustDdsDataReader* reader, struct Point* data_values, struct SampleInfo* sample_infos, int32_t max_samples, const InstanceHandle_t* previous_handle, SampleStateMask sample_states, ViewStateMask view_states, InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = Point_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline ReturnCode Point_dds_datareader_return_loan(DustDdsDataReader* reader, struct Point* data_values, struct SampleInfo* sample_infos) {
        return dds_datareader_return_loan(reader, NULL, sample_infos);
    }

    static inline ReturnCode Point_dds_datareader_get_key_value(DustDdsDataReader* reader, struct Point* key_holder, const InstanceHandle_t* handle) {
        if (reader == NULL || key_holder == NULL || handle == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = Point_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datareader_get_key_value(reader, sample, handle);
        if (result == RETCODE_OK) {
            *key_holder = Point_create_sample(sample);
        }
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode Point_dds_datareader_lookup_instance(DustDdsDataReader* reader, const struct Point* key_holder, InstanceHandle_t* handle) {
        if (reader == NULL || key_holder == NULL || handle == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = Point_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datareader_lookup_instance(reader, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }
    struct Data {
        int16_t id;
        double x;
    };

    static inline const DustDdsDynamicType* Data_get_type(void) {
        static const DustDdsDynamicType* type = NULL;
        if (type == NULL) {
            DustDdsTypeDescriptor descriptor = {
                .kind = TYPE_KIND_STRUCTURE,
                .name = "Data",
                .base_type = NULL,
                .discriminator_type = NULL,
                .bound = NULL,
                .element_type = NULL,
                .key_element_type = NULL,
                .extensibility_kind = EXTENSIBILITY_KIND_MUTABLE,
                .is_nested = false
            };
            DustDdsDynamicTypeBuilder* builder = dds_dynamic_type_builder_factory_create_type(&descriptor);
            {
                DustDdsMemberDescriptor member = {
                    .name = "id",
                    .id = 0,
                    .type = dds_dynamic_type_get_primitive_type(TYPE_KIND_INT16),
                    .is_key = true,
                    .is_optional = false,
                    .is_must_understand = true
                };
                dds_dynamic_type_builder_add_member(builder, &member);
            }
            {
                DustDdsMemberDescriptor member = {
                    .name = "x",
                    .id = 1,
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

    static inline struct Data Data_create_sample(DustDdsDynamicData* src) {
        struct Data sample;
        memset(&sample, 0, sizeof(sample));
        dds_dynamic_data_get_int16_value(src, 0, &sample.id);
        dds_dynamic_data_get_float64_value(src, 1, &sample.x);
        return sample;
    }

    static inline DustDdsDynamicData* Data_create_dynamic_sample(const struct Data* src) {
        DustDdsDynamicData* sample = dds_dynamic_data_create(Data_get_type());
        if (sample != NULL) {
            dds_dynamic_data_set_int16_value(sample, 0, src->id);
            dds_dynamic_data_set_float64_value(sample, 1, src->x);
        }
        return sample;
    }

    static inline void Data_free_sample(struct Data* sample) {
        if (sample != NULL) {
        }
    }

    static inline ReturnCode Data_dds_datawriter_write(DustDdsDataWriter* writer, const struct Data* data, const InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = Data_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_write(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode Data_dds_datawriter_write_w_timestamp(DustDdsDataWriter* writer, const struct Data* data, const InstanceHandle_t* handle, struct Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = Data_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_write_w_timestamp(writer, sample, handle, source_timestamp);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode Data_dds_datawriter_register_instance(DustDdsDataWriter* writer, const struct Data* data, InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL || handle == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = Data_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_register_instance(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode Data_dds_datawriter_register_instance_w_timestamp(DustDdsDataWriter* writer, const struct Data* data, struct Time_t source_timestamp, InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL || handle == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = Data_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_register_instance_w_timestamp(writer, sample, source_timestamp, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode Data_dds_datawriter_unregister_instance(DustDdsDataWriter* writer, const struct Data* data, const InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = Data_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_unregister_instance(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode Data_dds_datawriter_unregister_instance_w_timestamp(DustDdsDataWriter* writer, const struct Data* data, const InstanceHandle_t* handle, struct Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = Data_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_unregister_instance_w_timestamp(writer, sample, handle, source_timestamp);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode Data_dds_datawriter_dispose(DustDdsDataWriter* writer, const struct Data* data, const InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = Data_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_dispose(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode Data_dds_datawriter_dispose_w_timestamp(DustDdsDataWriter* writer, const struct Data* data, const InstanceHandle_t* handle, struct Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = Data_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_dispose_w_timestamp(writer, sample, handle, source_timestamp);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode Data_dds_datawriter_get_key_value(DustDdsDataWriter* writer, struct Data* key_holder, const InstanceHandle_t* handle) {
        if (writer == NULL || key_holder == NULL || handle == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = Data_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_get_key_value(writer, sample, handle);
        if (result == RETCODE_OK) {
            *key_holder = Data_create_sample(sample);
        }
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode Data_dds_datawriter_lookup_instance(DustDdsDataWriter* writer, const struct Data* key_holder, InstanceHandle_t* handle) {
        if (writer == NULL || key_holder == NULL || handle == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = Data_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_lookup_instance(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode Data_dds_datareader_read(DustDdsDataReader* reader, struct Data* data_values, struct SampleInfo* sample_infos, int32_t max_samples, SampleStateMask sample_states, ViewStateMask view_states, InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = Data_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline ReturnCode Data_dds_datareader_take(DustDdsDataReader* reader, struct Data* data_values, struct SampleInfo* sample_infos, int32_t max_samples, SampleStateMask sample_states, ViewStateMask view_states, InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = Data_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline ReturnCode Data_dds_datareader_read_next_sample(DustDdsDataReader* reader, struct Data* data_value, struct SampleInfo* sample_info) {
        if (reader == NULL || data_value == NULL || sample_info == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = NULL;
        ReturnCode result = dds_datareader_read_next_sample(reader, &sample, sample_info);
        if (result == RETCODE_OK) {
            if (sample != NULL) {
                *data_value = Data_create_sample(sample);
                dds_dynamic_data_free(sample);
            }
        }
        return result;
    }

    static inline ReturnCode Data_dds_datareader_take_next_sample(DustDdsDataReader* reader, struct Data* data_value, struct SampleInfo* sample_info) {
        if (reader == NULL || data_value == NULL || sample_info == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = NULL;
        ReturnCode result = dds_datareader_take_next_sample(reader, &sample, sample_info);
        if (result == RETCODE_OK) {
            if (sample != NULL) {
                *data_value = Data_create_sample(sample);
                dds_dynamic_data_free(sample);
            }
        }
        return result;
    }

    static inline ReturnCode Data_dds_datareader_read_instance(DustDdsDataReader* reader, struct Data* data_values, struct SampleInfo* sample_infos, int32_t max_samples, const InstanceHandle_t* a_handle, SampleStateMask sample_states, ViewStateMask view_states, InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = Data_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline ReturnCode Data_dds_datareader_take_instance(DustDdsDataReader* reader, struct Data* data_values, struct SampleInfo* sample_infos, int32_t max_samples, const InstanceHandle_t* a_handle, SampleStateMask sample_states, ViewStateMask view_states, InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = Data_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline ReturnCode Data_dds_datareader_read_next_instance(DustDdsDataReader* reader, struct Data* data_values, struct SampleInfo* sample_infos, int32_t max_samples, const InstanceHandle_t* previous_handle, SampleStateMask sample_states, ViewStateMask view_states, InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = Data_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline ReturnCode Data_dds_datareader_take_next_instance(DustDdsDataReader* reader, struct Data* data_values, struct SampleInfo* sample_infos, int32_t max_samples, const InstanceHandle_t* previous_handle, SampleStateMask sample_states, ViewStateMask view_states, InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = Data_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline ReturnCode Data_dds_datareader_return_loan(DustDdsDataReader* reader, struct Data* data_values, struct SampleInfo* sample_infos) {
        return dds_datareader_return_loan(reader, NULL, sample_infos);
    }

    static inline ReturnCode Data_dds_datareader_get_key_value(DustDdsDataReader* reader, struct Data* key_holder, const InstanceHandle_t* handle) {
        if (reader == NULL || key_holder == NULL || handle == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = Data_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datareader_get_key_value(reader, sample, handle);
        if (result == RETCODE_OK) {
            *key_holder = Data_create_sample(sample);
        }
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode Data_dds_datareader_lookup_instance(DustDdsDataReader* reader, const struct Data* key_holder, InstanceHandle_t* handle) {
        if (reader == NULL || key_holder == NULL || handle == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = Data_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datareader_lookup_instance(reader, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }
    struct MultiDimensionalPoint {
        double x;
        double y;
        double z;
    };

    static inline const DustDdsDynamicType* MultiDimensionalPoint_get_type(void) {
        static const DustDdsDynamicType* type = NULL;
        if (type == NULL) {
            DustDdsTypeDescriptor descriptor = {
                .kind = TYPE_KIND_STRUCTURE,
                .name = "MultiDimensionalPoint",
                .base_type = NULL,
                .discriminator_type = NULL,
                .bound = NULL,
                .element_type = NULL,
                .key_element_type = NULL,
                .extensibility_kind = EXTENSIBILITY_KIND_APPENDABLE,
                .is_nested = false
            };
            DustDdsDynamicTypeBuilder* builder = dds_dynamic_type_builder_factory_create_type(&descriptor);
            {
                DustDdsMemberDescriptor member = {
                    .name = "x",
                    .id = 0,
                    .type = dds_dynamic_type_get_primitive_type(TYPE_KIND_FLOAT64),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                dds_dynamic_type_builder_add_member(builder, &member);
            }
            {
                DustDdsMemberDescriptor member = {
                    .name = "y",
                    .id = 1,
                    .type = dds_dynamic_type_get_primitive_type(TYPE_KIND_FLOAT64),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                dds_dynamic_type_builder_add_member(builder, &member);
            }
            {
                DustDdsMemberDescriptor member = {
                    .name = "z",
                    .id = 2,
                    .type = dds_dynamic_type_get_primitive_type(TYPE_KIND_FLOAT64),
                    .is_key = false,
                    .is_optional = true,
                    .is_must_understand = false
                };
                dds_dynamic_type_builder_add_member(builder, &member);
            }
            type = dds_dynamic_type_builder_build(builder);
        }
        return type;
    }

    static inline struct MultiDimensionalPoint MultiDimensionalPoint_create_sample(DustDdsDynamicData* src) {
        struct MultiDimensionalPoint sample;
        memset(&sample, 0, sizeof(sample));
        dds_dynamic_data_get_float64_value(src, 0, &sample.x);
        dds_dynamic_data_get_float64_value(src, 1, &sample.y);
        dds_dynamic_data_get_float64_value(src, 2, &sample.z);
        return sample;
    }

    static inline DustDdsDynamicData* MultiDimensionalPoint_create_dynamic_sample(const struct MultiDimensionalPoint* src) {
        DustDdsDynamicData* sample = dds_dynamic_data_create(MultiDimensionalPoint_get_type());
        if (sample != NULL) {
            dds_dynamic_data_set_float64_value(sample, 0, src->x);
            dds_dynamic_data_set_float64_value(sample, 1, src->y);
            dds_dynamic_data_set_float64_value(sample, 2, src->z);
        }
        return sample;
    }

    static inline void MultiDimensionalPoint_free_sample(struct MultiDimensionalPoint* sample) {
        if (sample != NULL) {
        }
    }

    static inline ReturnCode MultiDimensionalPoint_dds_datawriter_write(DustDdsDataWriter* writer, const struct MultiDimensionalPoint* data, const InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_write(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode MultiDimensionalPoint_dds_datawriter_write_w_timestamp(DustDdsDataWriter* writer, const struct MultiDimensionalPoint* data, const InstanceHandle_t* handle, struct Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_write_w_timestamp(writer, sample, handle, source_timestamp);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode MultiDimensionalPoint_dds_datawriter_register_instance(DustDdsDataWriter* writer, const struct MultiDimensionalPoint* data, InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL || handle == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_register_instance(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode MultiDimensionalPoint_dds_datawriter_register_instance_w_timestamp(DustDdsDataWriter* writer, const struct MultiDimensionalPoint* data, struct Time_t source_timestamp, InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL || handle == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_register_instance_w_timestamp(writer, sample, source_timestamp, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode MultiDimensionalPoint_dds_datawriter_unregister_instance(DustDdsDataWriter* writer, const struct MultiDimensionalPoint* data, const InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_unregister_instance(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode MultiDimensionalPoint_dds_datawriter_unregister_instance_w_timestamp(DustDdsDataWriter* writer, const struct MultiDimensionalPoint* data, const InstanceHandle_t* handle, struct Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_unregister_instance_w_timestamp(writer, sample, handle, source_timestamp);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode MultiDimensionalPoint_dds_datawriter_dispose(DustDdsDataWriter* writer, const struct MultiDimensionalPoint* data, const InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_dispose(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode MultiDimensionalPoint_dds_datawriter_dispose_w_timestamp(DustDdsDataWriter* writer, const struct MultiDimensionalPoint* data, const InstanceHandle_t* handle, struct Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_dispose_w_timestamp(writer, sample, handle, source_timestamp);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode MultiDimensionalPoint_dds_datawriter_get_key_value(DustDdsDataWriter* writer, struct MultiDimensionalPoint* key_holder, const InstanceHandle_t* handle) {
        if (writer == NULL || key_holder == NULL || handle == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_get_key_value(writer, sample, handle);
        if (result == RETCODE_OK) {
            *key_holder = MultiDimensionalPoint_create_sample(sample);
        }
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode MultiDimensionalPoint_dds_datawriter_lookup_instance(DustDdsDataWriter* writer, const struct MultiDimensionalPoint* key_holder, InstanceHandle_t* handle) {
        if (writer == NULL || key_holder == NULL || handle == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_lookup_instance(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode MultiDimensionalPoint_dds_datareader_read(DustDdsDataReader* reader, struct MultiDimensionalPoint* data_values, struct SampleInfo* sample_infos, int32_t max_samples, SampleStateMask sample_states, ViewStateMask view_states, InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = MultiDimensionalPoint_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline ReturnCode MultiDimensionalPoint_dds_datareader_take(DustDdsDataReader* reader, struct MultiDimensionalPoint* data_values, struct SampleInfo* sample_infos, int32_t max_samples, SampleStateMask sample_states, ViewStateMask view_states, InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = MultiDimensionalPoint_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline ReturnCode MultiDimensionalPoint_dds_datareader_read_next_sample(DustDdsDataReader* reader, struct MultiDimensionalPoint* data_value, struct SampleInfo* sample_info) {
        if (reader == NULL || data_value == NULL || sample_info == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = NULL;
        ReturnCode result = dds_datareader_read_next_sample(reader, &sample, sample_info);
        if (result == RETCODE_OK) {
            if (sample != NULL) {
                *data_value = MultiDimensionalPoint_create_sample(sample);
                dds_dynamic_data_free(sample);
            }
        }
        return result;
    }

    static inline ReturnCode MultiDimensionalPoint_dds_datareader_take_next_sample(DustDdsDataReader* reader, struct MultiDimensionalPoint* data_value, struct SampleInfo* sample_info) {
        if (reader == NULL || data_value == NULL || sample_info == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = NULL;
        ReturnCode result = dds_datareader_take_next_sample(reader, &sample, sample_info);
        if (result == RETCODE_OK) {
            if (sample != NULL) {
                *data_value = MultiDimensionalPoint_create_sample(sample);
                dds_dynamic_data_free(sample);
            }
        }
        return result;
    }

    static inline ReturnCode MultiDimensionalPoint_dds_datareader_read_instance(DustDdsDataReader* reader, struct MultiDimensionalPoint* data_values, struct SampleInfo* sample_infos, int32_t max_samples, const InstanceHandle_t* a_handle, SampleStateMask sample_states, ViewStateMask view_states, InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = MultiDimensionalPoint_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline ReturnCode MultiDimensionalPoint_dds_datareader_take_instance(DustDdsDataReader* reader, struct MultiDimensionalPoint* data_values, struct SampleInfo* sample_infos, int32_t max_samples, const InstanceHandle_t* a_handle, SampleStateMask sample_states, ViewStateMask view_states, InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = MultiDimensionalPoint_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline ReturnCode MultiDimensionalPoint_dds_datareader_read_next_instance(DustDdsDataReader* reader, struct MultiDimensionalPoint* data_values, struct SampleInfo* sample_infos, int32_t max_samples, const InstanceHandle_t* previous_handle, SampleStateMask sample_states, ViewStateMask view_states, InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = MultiDimensionalPoint_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline ReturnCode MultiDimensionalPoint_dds_datareader_take_next_instance(DustDdsDataReader* reader, struct MultiDimensionalPoint* data_values, struct SampleInfo* sample_infos, int32_t max_samples, const InstanceHandle_t* previous_handle, SampleStateMask sample_states, ViewStateMask view_states, InstanceStateMask instance_states, int32_t* received_samples) {
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
                    data_values[i] = MultiDimensionalPoint_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline ReturnCode MultiDimensionalPoint_dds_datareader_return_loan(DustDdsDataReader* reader, struct MultiDimensionalPoint* data_values, struct SampleInfo* sample_infos) {
        return dds_datareader_return_loan(reader, NULL, sample_infos);
    }

    static inline ReturnCode MultiDimensionalPoint_dds_datareader_get_key_value(DustDdsDataReader* reader, struct MultiDimensionalPoint* key_holder, const InstanceHandle_t* handle) {
        if (reader == NULL || key_holder == NULL || handle == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datareader_get_key_value(reader, sample, handle);
        if (result == RETCODE_OK) {
            *key_holder = MultiDimensionalPoint_create_sample(sample);
        }
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode MultiDimensionalPoint_dds_datareader_lookup_instance(DustDdsDataReader* reader, const struct MultiDimensionalPoint* key_holder, InstanceHandle_t* handle) {
        if (reader == NULL || key_holder == NULL || handle == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(key_holder);
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
