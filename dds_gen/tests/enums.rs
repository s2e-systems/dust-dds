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

            pub use Suits::Spades;
            pub use Suits::Hearts;
            pub use Suits::Diamonds;
            pub use Suits::Clubs;

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

            pub use HttpStatusCode::CONTINUE;
            pub use HttpStatusCode::OK;
            pub use HttpStatusCode::MULTIPLE_CHOICES;
            pub use HttpStatusCode::BAD_REQUEST;
            pub use HttpStatusCode::NOT_FOUND;
            pub use HttpStatusCode::INTERNAL_SERVER_ERROR;

    "#
        .parse()
        .unwrap(),
    )
    .unwrap();

    let generated_string = dust_dds_gen::compile_idl(idl_file).unwrap();
    let result = syn::parse2::<File>(generated_string.parse().unwrap()).unwrap();

    assert_eq!(result, expected, "Generated {generated_string}");
}
