use syn::File;

#[test]
fn i11eperf_generation() {
    let idl = r#"
    module i11eperf {
        /* The "ts" field stores a time stamp and is used in lieu of the DDS source timestamp
           for two reasons:

           - Fast-DDS doesn't support write_w_timestamp

           - All variants have different ideas of the representation of a time stamp,
             this avoids the need for different conversions in the different cases

           @final @data_representation(XCDR1): the spec breaks backwards compatibility, with some
           implementations (OpenDDS, for example) following it, and some choosing not to break
           things just because a generally pretty low-quality spec says one must (Cyclone DDS).

           Keep all annotations on a single line starting with @topic for preproc-ospl-idl.gawk */

        @topic @final @data_representation(XCDR1)
        struct ou {
          unsigned long long ts;
          unsigned long s;
        };

        @topic @final @data_representation(XCDR1)
        struct a32 {
          unsigned long long ts;
          unsigned long s;
          octet xary[32 - 12];
        };

        @topic @final @data_representation(XCDR1)
        struct a128 {
          unsigned long long ts;
          unsigned long s;
          octet xary[128 - 12];
        };

        @topic @final @data_representation(XCDR1)
        struct a1024 {
          unsigned long long ts;
          unsigned long s;
          octet xary[1024 - 12];
        };

        @topic @final @data_representation(XCDR1)
        struct a16k {
          unsigned long long ts;
          unsigned long s;
          octet xary[16*1024 - 12];
        };

        @topic @final @data_representation(XCDR1)
        struct a48k {
          unsigned long long ts;
          unsigned long s;
          octet xary[48*1024 - 12];
        };

        @topic @final @data_representation(XCDR1)
        struct a64k {
          unsigned long long ts;
          unsigned long s;
          octet xary[64*1024 - 12];
        };

        @topic @final @data_representation(XCDR1)
        struct a1M {
          unsigned long long ts;
          unsigned long s;
          octet xary[1024*1024 - 12];
        };

        @topic @final @data_representation(XCDR1)
        struct a2M {
          unsigned long long ts;
          unsigned long s;
          octet xary[2*1024*1024 - 12];
        };

        @topic @final @data_representation(XCDR1)
        struct a4M {
          unsigned long long ts;
          unsigned long s;
          octet xary[4*1024*1024 - 12];
        };

        @topic @final @data_representation(XCDR1)
        struct a8M {
          unsigned long long ts;
          unsigned long s;
          octet xary[8*1024*1024 - 12];
        };

        @topic @final @data_representation(XCDR1)
        struct seq {
          unsigned long long ts;
          unsigned long s;
          sequence<octet> xseq;
        };
      };
    "#;

    let expected = syn::parse2::<File>(
        r#"
        pub mod i11eperf {
          #[derive(Debug, dust_dds::topic_definition::type_support::DdsType, dust_dds::dust_dds_xtypes::serialize::XTypesSerialize)]
            pub struct ou {
              pub ts: u64,
              pub s: u32,
            }
            #[derive(Debug, dust_dds::topic_definition::type_support::DdsType, dust_dds::dust_dds_xtypes::serialize::XTypesSerialize)]
            pub struct a32 {
              pub ts: u64,
              pub s: u32,
              pub xary: [u8; 32 - 12],
            }
            #[derive(Debug, dust_dds::topic_definition::type_support::DdsType, dust_dds::dust_dds_xtypes::serialize::XTypesSerialize)]
            pub struct a128 {
              pub ts: u64,
              pub s: u32,
              pub xary: [u8; 128 - 12],
            }
            #[derive(Debug, dust_dds::topic_definition::type_support::DdsType, dust_dds::dust_dds_xtypes::serialize::XTypesSerialize)]
            pub struct a1024 {
              pub ts: u64,
              pub s: u32,
              pub xary: [u8; 1024 - 12],
            }
            #[derive(Debug, dust_dds::topic_definition::type_support::DdsType, dust_dds::dust_dds_xtypes::serialize::XTypesSerialize)]
            pub struct a16k {
              pub ts: u64,
              pub s: u32,
              pub xary: [u8; 16*1024 - 12],
            }
            #[derive(Debug, dust_dds::topic_definition::type_support::DdsType, dust_dds::dust_dds_xtypes::serialize::XTypesSerialize)]
            pub struct a48k {
              pub ts: u64,
              pub s: u32,
              pub xary: [u8; 48*1024 - 12],
            }
            #[derive(Debug, dust_dds::topic_definition::type_support::DdsType, dust_dds::dust_dds_xtypes::serialize::XTypesSerialize)]
            pub struct a64k {
              pub ts: u64,
              pub s: u32,
              pub xary: [u8; 64*1024 - 12],
            }
            #[derive(Debug, dust_dds::topic_definition::type_support::DdsType, dust_dds::dust_dds_xtypes::serialize::XTypesSerialize)]
            pub struct a1M {
              pub ts: u64,
              pub s: u32,
              pub xary: [u8; 1024*1024 - 12],
            }
            #[derive(Debug, dust_dds::topic_definition::type_support::DdsType, dust_dds::dust_dds_xtypes::serialize::XTypesSerialize)]
            pub struct a2M {
              pub ts: u64,
              pub s: u32,
              pub xary: [u8; 2*1024*1024 - 12],
            }
            #[derive(Debug, dust_dds::topic_definition::type_support::DdsType, dust_dds::dust_dds_xtypes::serialize::XTypesSerialize)]
            pub struct a4M {
              pub ts: u64,
              pub s: u32,
              pub xary: [u8; 4*1024*1024 - 12],
            }
            #[derive(Debug, dust_dds::topic_definition::type_support::DdsType, dust_dds::dust_dds_xtypes::serialize::XTypesSerialize)]
            pub struct a8M {
              pub ts: u64,
              pub s: u32,
              pub xary: [u8; 8*1024*1024 - 12],
            }
            #[derive(Debug, dust_dds::topic_definition::type_support::DdsType, dust_dds::dust_dds_xtypes::serialize::XTypesSerialize)]
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

    let result =
        syn::parse2::<File>(dust_dds_gen::compile_idl(idl).unwrap().parse().unwrap()).unwrap();

    assert_eq!(result, expected);
}
