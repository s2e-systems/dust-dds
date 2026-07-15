use std::path::Path;

use syn::File;

#[test]
fn nested_types() {
    let idl_file = Path::new("tests/nested_types.idl");

    let expected = syn::parse2::<File>(
        r#"
            #[derive(Debug, Clone, dust_dds::infrastructure::type_support::DdsType)]
            pub enum Presence {
                Present,
                NotPresent,
            }
            #[derive(Debug, Clone, dust_dds::infrastructure::type_support::DdsType)]
            pub struct Color {
                pub red: u8,
                pub green: u8,
                pub blue: u8,
            }

            #[derive(Debug, Clone, dust_dds::infrastructure::type_support::DdsType)]
            pub struct ColorSensor {
                pub state: Presence,
                pub value: Color,
            }"#
        .parse()
        .unwrap(),
    )
    .unwrap();

    let result = syn::parse2::<File>(
        dust_dds_gen::compile_idl(idl_file)
            .unwrap()
            .parse()
            .unwrap(),
    )
    .unwrap();

    assert_eq!(result, expected);
}
