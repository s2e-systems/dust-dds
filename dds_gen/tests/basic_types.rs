use syn::File;

#[test]
fn structs_generation() {
    let idl = r#"
        struct BasicTypes {
            boolean a;
            char b;
            wchar c;
            octet d;
            string e;
            wstring f;
            short g;
            unsigned short h;
            long i;
            unsigned long j;
            long long k;
            unsigned long long l;
            float m;
            double n;
            // fixed o;
        };
    "#;

    let expected = syn::parse2::<File>(
        r#"
            #[derive(Debug, dust_dds::topic_definition::type_support::DdsType)]
            pub struct BasicTypes {
                pub a: bool,
                pub b: char,
                pub c: char,
                pub d: u8,
                pub e: String,
                pub f: String,
                pub g: i16,
                pub h: u16,
                pub i: i32,
                pub j: u32,
                pub k: i64,
                pub l: u64,
                pub m: f32,
                pub n: f64,
            }
    "#
        .parse()
        .unwrap(),
    )
    .unwrap();

    let result =
        syn::parse2::<File>(dust_dds_gen::compile_idl(idl).unwrap().parse().unwrap()).unwrap();

    assert_eq!(result, expected);
}
