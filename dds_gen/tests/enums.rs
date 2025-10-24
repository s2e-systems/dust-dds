use std::path::Path;

use syn::File;

#[test]
fn enums_generation() {
    let idl_file = Path::new("tests/enums.idl");

    let expected = syn::parse2::<File>(
        r#"
            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            pub enum Suits {
                Spades,
                Hearts,
                Diamonds,
                Clubs,
            }
    "#
        .parse()
        .unwrap(),
    )
    .unwrap();

    let generated_string = dust_dds_gen::compile_idl(idl_file).unwrap();
    let result = syn::parse2::<File>(generated_string.parse().unwrap()).unwrap();

    assert_eq!(result, expected, "Generated {generated_string}");
}
