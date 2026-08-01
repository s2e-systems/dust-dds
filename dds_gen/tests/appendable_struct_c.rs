use std::path::Path;

#[test]
fn appendable_struct() {
    let idl_file = Path::new("tests/appendable_struct.idl");
    let expected = r#"
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

    static inline ReturnCode dds_datawriter_write_Point(DustDdsDataWriter* writer, const struct Point* data) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_write(writer, sample);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode dds_datareader_read_Point(DustDdsDataReader* reader, struct Point* data_values, int32_t max_samples, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || received_samples == NULL || max_samples <= 0) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData** samples = (DustDdsDynamicData**)calloc(max_samples, sizeof(DustDdsDynamicData*));
        if (samples == NULL) {
            return RETCODE_OUT_OF_RESOURCES;
        }
        ReturnCode result = dds_datareader_read(reader, samples, max_samples, received_samples);
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

    static inline ReturnCode dds_datawriter_write_Data(DustDdsDataWriter* writer, const struct Data* data) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = Data_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_write(writer, sample);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode dds_datareader_read_Data(DustDdsDataReader* reader, struct Data* data_values, int32_t max_samples, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || received_samples == NULL || max_samples <= 0) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData** samples = (DustDdsDynamicData**)calloc(max_samples, sizeof(DustDdsDynamicData*));
        if (samples == NULL) {
            return RETCODE_OUT_OF_RESOURCES;
        }
        ReturnCode result = dds_datareader_read(reader, samples, max_samples, received_samples);
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

    static inline ReturnCode dds_datawriter_write_MultiDimensionalPoint(DustDdsDataWriter* writer, const struct MultiDimensionalPoint* data) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_write(writer, sample);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode dds_datareader_read_MultiDimensionalPoint(DustDdsDataReader* reader, struct MultiDimensionalPoint* data_values, int32_t max_samples, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || received_samples == NULL || max_samples <= 0) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData** samples = (DustDdsDynamicData**)calloc(max_samples, sizeof(DustDdsDynamicData*));
        if (samples == NULL) {
            return RETCODE_OUT_OF_RESOURCES;
        }
        ReturnCode result = dds_datareader_read(reader, samples, max_samples, received_samples);
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
"#;

    let result = dust_dds_gen::compile_idl_c(idl_file).unwrap();

    assert_eq!(result, expected);
}
