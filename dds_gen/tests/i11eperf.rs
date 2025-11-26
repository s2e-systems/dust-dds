use std::path::Path;

use syn::File;

#[test]
fn i11eperf_generation() {
    let idl_file = Path::new("tests/i11eperf.idl");

    let expected = syn::parse2::<File>(
        r#"
        pub mod i11eperf {
          #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
          #[dust_dds(extensibility = "final")]
          #[dust_dds(name = "i11eperf::ou")]
            pub struct ou {
              pub ts: u64,
              pub s: u32,
            }
            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            #[dust_dds(extensibility = "final")]
            #[dust_dds(name = "i11eperf::a32")]
            pub struct a32 {
              pub ts: u64,
              pub s: u32,
              pub xary: [u8; 32 - 12],
            }
            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            #[dust_dds(extensibility = "final")]
            #[dust_dds(name = "i11eperf::a128")]
            pub struct a128 {
              pub ts: u64,
              pub s: u32,
              pub xary: [u8; 128 - 12],
            }
            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            #[dust_dds(extensibility = "final")]
            #[dust_dds(name = "i11eperf::a1024")]
            pub struct a1024 {
              pub ts: u64,
              pub s: u32,
              pub xary: [u8; 1024 - 12],
            }
            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            #[dust_dds(extensibility = "final")]
            #[dust_dds(name = "i11eperf::a16k")]
            pub struct a16k {
              pub ts: u64,
              pub s: u32,
              pub xary: [u8; 16*1024 - 12],
            }
            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            #[dust_dds(extensibility = "final")]
            #[dust_dds(name = "i11eperf::a48k")]
            pub struct a48k {
              pub ts: u64,
              pub s: u32,
              pub xary: [u8; 48*1024 - 12],
            }
            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            #[dust_dds(extensibility = "final")]
            #[dust_dds(name = "i11eperf::a64k")]
            pub struct a64k {
              pub ts: u64,
              pub s: u32,
              pub xary: [u8; 64*1024 - 12],
            }
            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            #[dust_dds(extensibility = "final")]
            #[dust_dds(name = "i11eperf::a1M")]
            pub struct a1M {
              pub ts: u64,
              pub s: u32,
              pub xary: [u8; 1024*1024 - 12],
            }
            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            #[dust_dds(extensibility = "final")]
            #[dust_dds(name = "i11eperf::a2M")]
            pub struct a2M {
              pub ts: u64,
              pub s: u32,
              pub xary: [u8; 2*1024*1024 - 12],
            }
            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            #[dust_dds(extensibility = "final")]
            #[dust_dds(name = "i11eperf::a4M")]
            pub struct a4M {
              pub ts: u64,
              pub s: u32,
              pub xary: [u8; 4*1024*1024 - 12],
            }
            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            #[dust_dds(extensibility = "final")]
            #[dust_dds(name = "i11eperf::a8M")]
            pub struct a8M {
              pub ts: u64,
              pub s: u32,
              pub xary: [u8; 8*1024*1024 - 12],
            }
            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            #[dust_dds(extensibility = "final")]
            #[dust_dds(name = "i11eperf::seq")]
            pub struct seq {
              pub ts: u64,
              pub s: u32,
              pub xseq: Vec<u8>,
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
