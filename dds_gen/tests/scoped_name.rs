use std::path::Path;
use syn::File;

#[test]
fn basic_types() {
    let idl_file = Path::new("tests/scoped_name.idl");
    let expected = syn::parse2::<File>(
        r#"
            pub mod first{
                #[derive(::core::fmt::Debug, ::core::clone::Clone, ::dust_dds::infrastructure::type_support::DdsType)]
                #[dust_dds(name = "first::MyEnum")]
                pub enum MyEnum {
                    A,
                    B,
                    C,
                }
                pub mod second{
                    #[derive(::core::fmt::Debug, ::core::clone::Clone, ::dust_dds::infrastructure::type_support::DdsType)]
                    #[dust_dds(name = "first::second::ColorRGB")]
                    pub enum ColorRGB {
                        RED,
                        GREEN,
                        BLUE,
                    }
                    #[derive(::core::fmt::Debug, ::core::clone::Clone, ::dust_dds::infrastructure::type_support::DdsType)]
                    #[dust_dds(name = "first::second::MyStruct")]
                    pub struct MyStruct {
                        pub my_enum_fqn: super::super::first::MyEnum,
                        pub color_rgb_fqn: super::super::first::second::ColorRGB,
                        pub color_rgb: ColorRGB,
                    }
                }
            }
            #[derive(::core::fmt::Debug, ::core::clone::Clone, ::dust_dds::infrastructure::type_support::DdsType)]
            pub struct Wrapper {
                pub my_enum: first::MyEnum,
                pub color_rgb: first::second::ColorRGB,
                pub my_struct: first::second::MyStruct,
                pub my_enum_fqn: first::MyEnum,
                pub color_rgb_fqn: first::second::ColorRGB,
                pub my_struct_fqn: first::second::MyStruct,
            }
        "#
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
