use std::path::Path;

#[test]
fn basic_types() {
    let idl_file = Path::new("tests/basic_types.idl");
    let expected = r#"
    #include <stdbool.h>
    #include <stdint.h>
    #include <stddef.h>
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
            dust_dds_dynamic_type_builder_add_member(builder, "a", 0, dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_BOOLEAN));
            dust_dds_dynamic_type_builder_add_member(builder, "b", 1, dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_CHAR8));
            dust_dds_dynamic_type_builder_add_member(builder, "c", 2, dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_CHAR8));
            dust_dds_dynamic_type_builder_add_member(builder, "d", 3, dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_UINT8));
            {
                DustDdsDynamicType* member_type = dust_dds_dynamic_type_create_string_type(4294967295);
                dust_dds_dynamic_type_builder_add_member(builder, "e", 4, member_type);
                dust_dds_dynamic_type_free(member_type);
            }
            {
                DustDdsDynamicType* member_type = dust_dds_dynamic_type_create_string_type(4294967295);
                dust_dds_dynamic_type_builder_add_member(builder, "f", 5, member_type);
                dust_dds_dynamic_type_free(member_type);
            }
            dust_dds_dynamic_type_builder_add_member(builder, "g", 6, dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_INT16));
            dust_dds_dynamic_type_builder_add_member(builder, "h", 7, dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_UINT16));
            dust_dds_dynamic_type_builder_add_member(builder, "i", 8, dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_INT32));
            dust_dds_dynamic_type_builder_add_member(builder, "j", 9, dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_UINT32));
            dust_dds_dynamic_type_builder_add_member(builder, "k", 10, dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_INT64));
            dust_dds_dynamic_type_builder_add_member(builder, "l", 11, dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_UINT64));
            dust_dds_dynamic_type_builder_add_member(builder, "m", 12, dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_FLOAT32));
            dust_dds_dynamic_type_builder_add_member(builder, "n", 13, dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_FLOAT64));
            type = dust_dds_dynamic_type_builder_build(builder);
        }
        return type;
    }
"#;

    let result = dust_dds_gen::compile_idl_c(idl_file).unwrap();

    assert_eq!(result, expected);
}
