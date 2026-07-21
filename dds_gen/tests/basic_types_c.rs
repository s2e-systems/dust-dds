use std::path::Path;

#[test]
fn basic_types() {
    let idl_file = Path::new("tests/basic_types.idl");
    let expected = r#"
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
            DustDdsDynamicTypeBuilder* builder = dust_dds_dynamic_type_builder_create_struct("BasicTypes");
            {
                DustDdsMemberDescriptor* member = dust_dds_member_descriptor_new("a", 0, dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_BOOLEAN));
                dust_dds_dynamic_type_builder_add_member(builder, member);
                dust_dds_member_descriptor_free(member);
            }
            {
                DustDdsMemberDescriptor* member = dust_dds_member_descriptor_new("b", 1, dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_CHAR8));
                dust_dds_dynamic_type_builder_add_member(builder, member);
                dust_dds_member_descriptor_free(member);
            }
            {
                DustDdsMemberDescriptor* member = dust_dds_member_descriptor_new("c", 2, dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_CHAR8));
                dust_dds_dynamic_type_builder_add_member(builder, member);
                dust_dds_member_descriptor_free(member);
            }
            {
                DustDdsMemberDescriptor* member = dust_dds_member_descriptor_new("d", 3, dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_UINT8));
                dust_dds_dynamic_type_builder_add_member(builder, member);
                dust_dds_member_descriptor_free(member);
            }
            {
                DustDdsDynamicType* member_type = dust_dds_dynamic_type_create_string_type(4294967295);
                DustDdsMemberDescriptor* member = dust_dds_member_descriptor_new("e", 4, member_type);
                dust_dds_dynamic_type_builder_add_member(builder, member);
                dust_dds_member_descriptor_free(member);
                dust_dds_dynamic_type_free(member_type);
            }
            {
                DustDdsDynamicType* member_type = dust_dds_dynamic_type_create_string_type(4294967295);
                DustDdsMemberDescriptor* member = dust_dds_member_descriptor_new("f", 5, member_type);
                dust_dds_dynamic_type_builder_add_member(builder, member);
                dust_dds_member_descriptor_free(member);
                dust_dds_dynamic_type_free(member_type);
            }
            {
                DustDdsMemberDescriptor* member = dust_dds_member_descriptor_new("g", 6, dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_INT16));
                dust_dds_dynamic_type_builder_add_member(builder, member);
                dust_dds_member_descriptor_free(member);
            }
            {
                DustDdsMemberDescriptor* member = dust_dds_member_descriptor_new("h", 7, dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_UINT16));
                dust_dds_dynamic_type_builder_add_member(builder, member);
                dust_dds_member_descriptor_free(member);
            }
            {
                DustDdsMemberDescriptor* member = dust_dds_member_descriptor_new("i", 8, dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_INT32));
                dust_dds_dynamic_type_builder_add_member(builder, member);
                dust_dds_member_descriptor_free(member);
            }
            {
                DustDdsMemberDescriptor* member = dust_dds_member_descriptor_new("j", 9, dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_UINT32));
                dust_dds_dynamic_type_builder_add_member(builder, member);
                dust_dds_member_descriptor_free(member);
            }
            {
                DustDdsMemberDescriptor* member = dust_dds_member_descriptor_new("k", 10, dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_INT64));
                dust_dds_dynamic_type_builder_add_member(builder, member);
                dust_dds_member_descriptor_free(member);
            }
            {
                DustDdsMemberDescriptor* member = dust_dds_member_descriptor_new("l", 11, dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_UINT64));
                dust_dds_dynamic_type_builder_add_member(builder, member);
                dust_dds_member_descriptor_free(member);
            }
            {
                DustDdsMemberDescriptor* member = dust_dds_member_descriptor_new("m", 12, dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_FLOAT32));
                dust_dds_dynamic_type_builder_add_member(builder, member);
                dust_dds_member_descriptor_free(member);
            }
            {
                DustDdsMemberDescriptor* member = dust_dds_member_descriptor_new("n", 13, dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_FLOAT64));
                dust_dds_dynamic_type_builder_add_member(builder, member);
                dust_dds_member_descriptor_free(member);
            }
            type = dust_dds_dynamic_type_builder_build(builder);
        }
        return type;
    }

    static inline struct BasicTypes BasicTypes_create_sample(DustDdsDynamicData* src) {
        struct BasicTypes sample;
        memset(&sample, 0, sizeof(sample));
        dust_dds_dynamic_data_get_boolean_value(src, 0, &sample.a);
        dust_dds_dynamic_data_get_char8_value(src, 1, &sample.b);
        {
            char temp;
            dust_dds_dynamic_data_get_char8_value(src, 2, &temp);
            sample.c = (wchar_t)temp;
        }
        dust_dds_dynamic_data_get_uint8_value(src, 3, &sample.d);
        dust_dds_dynamic_data_get_string_value(src, 4, &sample.e);
        {
            char* temp = NULL;
            dust_dds_dynamic_data_get_string_value(src, 5, &temp);
            if (temp != NULL) {
                size_t len = mbstowcs(NULL, temp, 0);
                if (len != (size_t)-1) {
                    sample.f = malloc((len + 1) * sizeof(wchar_t));
                    mbstowcs(sample.f, temp, len + 1);
                }
                dust_dds_string_free(temp);
            }
        }
        dust_dds_dynamic_data_get_int16_value(src, 6, &sample.g);
        dust_dds_dynamic_data_get_uint16_value(src, 7, &sample.h);
        dust_dds_dynamic_data_get_int32_value(src, 8, &sample.i);
        dust_dds_dynamic_data_get_uint32_value(src, 9, &sample.j);
        dust_dds_dynamic_data_get_int64_value(src, 10, &sample.k);
        dust_dds_dynamic_data_get_uint64_value(src, 11, &sample.l);
        dust_dds_dynamic_data_get_float32_value(src, 12, &sample.m);
        dust_dds_dynamic_data_get_float64_value(src, 13, &sample.n);
        return sample;
    }

    static inline DustDdsDynamicData* BasicTypes_create_dynamic_sample(const struct BasicTypes* src) {
        DustDdsDynamicData* sample = dust_dds_dynamic_data_create(BasicTypes_get_type());
        if (sample != NULL) {
            dust_dds_dynamic_data_set_boolean_value(sample, 0, src->a);
            dust_dds_dynamic_data_set_char8_value(sample, 1, src->b);
            dust_dds_dynamic_data_set_char8_value(sample, 2, (char)src->c);
            dust_dds_dynamic_data_set_uint8_value(sample, 3, src->d);
            dust_dds_dynamic_data_set_string_value(sample, 4, src->e);
            {
                if (src->f != NULL) {
                    size_t len = wcstombs(NULL, src->f, 0);
                    if (len != (size_t)-1) {
                        char* temp = malloc(len + 1);
                        wcstombs(temp, src->f, len + 1);
                        dust_dds_dynamic_data_set_string_value(sample, 5, temp);
                        free(temp);
                    }
                }
            }
            dust_dds_dynamic_data_set_int16_value(sample, 6, src->g);
            dust_dds_dynamic_data_set_uint16_value(sample, 7, src->h);
            dust_dds_dynamic_data_set_int32_value(sample, 8, src->i);
            dust_dds_dynamic_data_set_uint32_value(sample, 9, src->j);
            dust_dds_dynamic_data_set_int64_value(sample, 10, src->k);
            dust_dds_dynamic_data_set_uint64_value(sample, 11, src->l);
            dust_dds_dynamic_data_set_float32_value(sample, 12, src->m);
            dust_dds_dynamic_data_set_float64_value(sample, 13, src->n);
        }
        return sample;
    }

    static inline void BasicTypes_free_sample(struct BasicTypes* sample) {
        if (sample != NULL) {
        dust_dds_string_free(sample->e);
        free(sample->f);
        }
    }

    static inline ReturnCode dust_dds_datawriter_write_BasicTypes(DustDdsDataWriter* writer, const struct BasicTypes* data) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = BasicTypes_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dust_dds_datawriter_write(writer, sample);
        dust_dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode dust_dds_datareader_read_BasicTypes(DustDdsDataReader* reader, struct BasicTypes* data_values, int32_t max_samples, int32_t* received_samples) {
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
                    data_values[i] = BasicTypes_create_sample(samples[i]);
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
