use std::path::Path;
use syn::File;

#[test]
fn union_types() {
    let idl_file = Path::new("tests/union_types.idl");

    let expected = syn::parse2::<File>(
        r#"
            #[derive(::core::fmt::Debug, ::core::clone::Clone, ::dust_dds::infrastructure::type_support::DdsType)]
            #[dust_dds(switch(u8))]
            pub enum TestUnion {
                #[dust_dds(case = 10, )]
                x(u8),
                #[dust_dds(case = 20, case = 25, case = 30, )]
                y(i32),
                #[dust_dds(case = 50, default, case = 60, )]
                z(i64),
            }

            #[derive(::core::fmt::Debug, ::core::clone::Clone, ::dust_dds::infrastructure::type_support::DdsType)]
            #[dust_dds(switch(u8), extensibility = "final")]
            pub enum FinalUnion {
                #[dust_dds(case = 0, )]
            	x(u8),
                #[dust_dds(case = 1, )]
            	y(u8),
                #[dust_dds(case = 2, )]
            	z(u8),
            }

            #[derive(::core::fmt::Debug, ::core::clone::Clone, ::dust_dds::infrastructure::type_support::DdsType)]
            #[dust_dds(switch(u8), extensibility = "appendable")]
            pub enum AppendableUnion {
                #[dust_dds(case = 0, )]
                x(u8),
                #[dust_dds(case = 1, )]
                y(u8),
                #[dust_dds(case = 2, )]
                z(u8),
            }

            #[derive(::core::fmt::Debug, ::core::clone::Clone, ::dust_dds::infrastructure::type_support::DdsType)]
            #[dust_dds(switch(u8), extensibility = "mutable")]
            pub enum MutableUnion {
                #[dust_dds(case = 0, )]
            	x(u8),
                #[dust_dds(case = 1, )]
            	y(u8),
                #[dust_dds(case = 2, )]
            	z(u8),
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
