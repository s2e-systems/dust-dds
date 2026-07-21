use std::path::Path;

#[test]
fn basic_types() {
    let idl_file = Path::new("tests/basic_types.idl");
    let expected = r#"
    #include <stdbool.h>
    #include <stdint.h>
    #include <stddef.h>

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
"#;

    let result = dust_dds_gen::compile_idl_c(idl_file).unwrap();

    assert_eq!(result, expected);
}
