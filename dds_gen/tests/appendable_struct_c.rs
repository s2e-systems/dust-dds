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
            DustDdsDynamicTypeBuilder* builder = dust_dds_dynamic_type_builder_create_struct("Point");
            dust_dds_dynamic_type_builder_set_extensibility(builder, EXTENSIBILITY_KIND_APPENDABLE);
            {
                DustDdsMemberDescriptor* member = dust_dds_member_descriptor_new("x", 0, dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_FLOAT64));
                dust_dds_dynamic_type_builder_add_member(builder, member);
                dust_dds_member_descriptor_free(member);
            }
            {
                DustDdsMemberDescriptor* member = dust_dds_member_descriptor_new("y", 1, dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_FLOAT64));
                dust_dds_dynamic_type_builder_add_member(builder, member);
                dust_dds_member_descriptor_free(member);
            }
            type = dust_dds_dynamic_type_builder_build(builder);
        }
        return type;
    }

    static inline struct Point Point_create_sample(DustDdsDynamicData* src) {
        struct Point sample;
        memset(&sample, 0, sizeof(sample));
        dust_dds_dynamic_data_get_float64_value(src, 0, &sample.x);
        dust_dds_dynamic_data_get_float64_value(src, 1, &sample.y);
        return sample;
    }

    static inline DustDdsDynamicData* Point_create_dynamic_sample(const struct Point* src) {
        DustDdsDynamicData* sample = dust_dds_dynamic_data_create(Point_get_type());
        if (sample != NULL) {
            dust_dds_dynamic_data_set_float64_value(sample, 0, src->x);
            dust_dds_dynamic_data_set_float64_value(sample, 1, src->y);
        }
        return sample;
    }

    static inline void Point_free_sample(struct Point* sample) {
        if (sample != NULL) {
        }
    }

    static inline ReturnCode dust_dds_datawriter_write_Point(DustDdsDataWriter* writer, const struct Point* data) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dust_dds_datawriter_write(writer, sample);
        dust_dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode dust_dds_datareader_read_Point(DustDdsDataReader* reader, struct Point* data_values, int32_t max_samples, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || received_samples == NULL || max_samples <= 0) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData** samples = (DustDdsDynamicData**)calloc(max_samples, sizeof(DustDdsDynamicData*));
        if (samples == NULL) {
            return RETCODE_OUT_OF_RESOURCES;
        }
        ReturnCode result = dust_dds_datareader_read(reader, samples, max_samples, received_samples);
        if (result == RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = Point_create_sample(samples[i]);
                    dust_dds_dynamic_data_free(samples[i]);
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
            DustDdsDynamicTypeBuilder* builder = dust_dds_dynamic_type_builder_create_struct("Data");
            dust_dds_dynamic_type_builder_set_extensibility(builder, EXTENSIBILITY_KIND_MUTABLE);
            {
                DustDdsMemberDescriptor* member = dust_dds_member_descriptor_new("id", 0, dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_INT16));
                dust_dds_member_descriptor_set_is_key(member, true);
                dust_dds_dynamic_type_builder_add_member(builder, member);
                dust_dds_member_descriptor_free(member);
            }
            {
                DustDdsMemberDescriptor* member = dust_dds_member_descriptor_new("x", 1, dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_FLOAT64));
                dust_dds_dynamic_type_builder_add_member(builder, member);
                dust_dds_member_descriptor_free(member);
            }
            type = dust_dds_dynamic_type_builder_build(builder);
        }
        return type;
    }

    static inline struct Data Data_create_sample(DustDdsDynamicData* src) {
        struct Data sample;
        memset(&sample, 0, sizeof(sample));
        dust_dds_dynamic_data_get_int16_value(src, 0, &sample.id);
        dust_dds_dynamic_data_get_float64_value(src, 1, &sample.x);
        return sample;
    }

    static inline DustDdsDynamicData* Data_create_dynamic_sample(const struct Data* src) {
        DustDdsDynamicData* sample = dust_dds_dynamic_data_create(Data_get_type());
        if (sample != NULL) {
            dust_dds_dynamic_data_set_int16_value(sample, 0, src->id);
            dust_dds_dynamic_data_set_float64_value(sample, 1, src->x);
        }
        return sample;
    }

    static inline void Data_free_sample(struct Data* sample) {
        if (sample != NULL) {
        }
    }

    static inline ReturnCode dust_dds_datawriter_write_Data(DustDdsDataWriter* writer, const struct Data* data) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = Data_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dust_dds_datawriter_write(writer, sample);
        dust_dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode dust_dds_datareader_read_Data(DustDdsDataReader* reader, struct Data* data_values, int32_t max_samples, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || received_samples == NULL || max_samples <= 0) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData** samples = (DustDdsDynamicData**)calloc(max_samples, sizeof(DustDdsDynamicData*));
        if (samples == NULL) {
            return RETCODE_OUT_OF_RESOURCES;
        }
        ReturnCode result = dust_dds_datareader_read(reader, samples, max_samples, received_samples);
        if (result == RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = Data_create_sample(samples[i]);
                    dust_dds_dynamic_data_free(samples[i]);
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
            DustDdsDynamicTypeBuilder* builder = dust_dds_dynamic_type_builder_create_struct("MultiDimensionalPoint");
            dust_dds_dynamic_type_builder_set_extensibility(builder, EXTENSIBILITY_KIND_APPENDABLE);
            {
                DustDdsMemberDescriptor* member = dust_dds_member_descriptor_new("x", 0, dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_FLOAT64));
                dust_dds_dynamic_type_builder_add_member(builder, member);
                dust_dds_member_descriptor_free(member);
            }
            {
                DustDdsMemberDescriptor* member = dust_dds_member_descriptor_new("y", 1, dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_FLOAT64));
                dust_dds_dynamic_type_builder_add_member(builder, member);
                dust_dds_member_descriptor_free(member);
            }
            {
                DustDdsMemberDescriptor* member = dust_dds_member_descriptor_new("z", 2, dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_FLOAT64));
                dust_dds_member_descriptor_set_is_optional(member, true);
                dust_dds_dynamic_type_builder_add_member(builder, member);
                dust_dds_member_descriptor_free(member);
            }
            type = dust_dds_dynamic_type_builder_build(builder);
        }
        return type;
    }

    static inline struct MultiDimensionalPoint MultiDimensionalPoint_create_sample(DustDdsDynamicData* src) {
        struct MultiDimensionalPoint sample;
        memset(&sample, 0, sizeof(sample));
        dust_dds_dynamic_data_get_float64_value(src, 0, &sample.x);
        dust_dds_dynamic_data_get_float64_value(src, 1, &sample.y);
        dust_dds_dynamic_data_get_float64_value(src, 2, &sample.z);
        return sample;
    }

    static inline DustDdsDynamicData* MultiDimensionalPoint_create_dynamic_sample(const struct MultiDimensionalPoint* src) {
        DustDdsDynamicData* sample = dust_dds_dynamic_data_create(MultiDimensionalPoint_get_type());
        if (sample != NULL) {
            dust_dds_dynamic_data_set_float64_value(sample, 0, src->x);
            dust_dds_dynamic_data_set_float64_value(sample, 1, src->y);
            dust_dds_dynamic_data_set_float64_value(sample, 2, src->z);
        }
        return sample;
    }

    static inline void MultiDimensionalPoint_free_sample(struct MultiDimensionalPoint* sample) {
        if (sample != NULL) {
        }
    }

    static inline ReturnCode dust_dds_datawriter_write_MultiDimensionalPoint(DustDdsDataWriter* writer, const struct MultiDimensionalPoint* data) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = MultiDimensionalPoint_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dust_dds_datawriter_write(writer, sample);
        dust_dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode dust_dds_datareader_read_MultiDimensionalPoint(DustDdsDataReader* reader, struct MultiDimensionalPoint* data_values, int32_t max_samples, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || received_samples == NULL || max_samples <= 0) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData** samples = (DustDdsDynamicData**)calloc(max_samples, sizeof(DustDdsDynamicData*));
        if (samples == NULL) {
            return RETCODE_OUT_OF_RESOURCES;
        }
        ReturnCode result = dust_dds_datareader_read(reader, samples, max_samples, received_samples);
        if (result == RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = MultiDimensionalPoint_create_sample(samples[i]);
                    dust_dds_dynamic_data_free(samples[i]);
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
