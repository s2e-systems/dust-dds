use std::path::Path;

use syn::File;

#[test]
fn enums() {
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

            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            #[dust_dds(bit_bound(16))]
            pub enum HttpStatusCode {
                CONTINUE = 100,
                OK = 200,
                MULTIPLE_CHOICES = 300,
                BAD_REQUEST = 400,
                NOT_FOUND = 404,
                INTERNAL_SERVER_ERROR = 500,
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
