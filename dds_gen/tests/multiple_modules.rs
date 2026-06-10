use std::path::Path;

use syn::File;

#[test]
fn template_types() {
    let idl_file = Path::new("tests/multiple_modules.idl");

    let expected = syn::parse2::<File>(
        r#"
            mod gha {
                #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
                pub struct ObjectiveType {
                    pub name: String,
                }
            }

            mod gha {
                #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
                pub struct Objective {
                    pub objective_type: ObjectiveType,
                }
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
