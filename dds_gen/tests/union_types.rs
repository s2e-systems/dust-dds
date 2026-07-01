use std::path::Path;

use syn::File;

#[test]
fn union_types() {
    let idl_file = Path::new("tests/union_types.idl");

    let expected = syn::parse2::<File>(
        r#"
            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            pub enum TestUnion {
                case10{x: u8},
                case20{y: i32},
            }
            "#
        .parse()
        .unwrap(),
    )
    .unwrap();

    println!("Generated: {}", dust_dds_gen::compile_idl(idl_file)
            .unwrap());
        
    let result = syn::parse2::<File>(
        dust_dds_gen::compile_idl(idl_file)
            .unwrap()
            .parse()
            .unwrap(),
    )
    .unwrap();

    assert_eq!(result, expected);
}
