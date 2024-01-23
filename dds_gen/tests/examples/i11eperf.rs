mod i11eperf {
    #[derive(Debug, serde::Deserialize, serde::Serialize, dust_dds::DdsType)]
    struct ou {
        ts: u64,
        s: u32,
    }
    #[derive(Debug, serde::Deserialize, serde::Serialize, dust_dds::DdsType)]
    struct a32 {
        ts: u64,
        s: u32,
        xary: [u8; 32 - 12],
    }
    #[derive(Debug, serde::Deserialize, serde::Serialize, dust_dds::DdsType)]
    struct a128 {
        ts: u64,
        s: u32,
        xary: [u8; 128 - 12],
    }
    #[derive(Debug, serde::Deserialize, serde::Serialize, dust_dds::DdsType)]
    struct a1024 {
        ts: u64,
        s: u32,
        xary: [u8; 1024 - 12],
    }
    #[derive(Debug, serde::Deserialize, serde::Serialize, dust_dds::DdsType)]
    struct a16k {
        ts: u64,
        s: u32,
        xary: [u8; 16*1024 - 12],
    }
    #[derive(Debug, serde::Deserialize, serde::Serialize, dust_dds::DdsType)]
    struct a48k {
        ts: u64,
        s: u32,
        xary: [u8; 48*1024 - 12],
    }
    #[derive(Debug, serde::Deserialize, serde::Serialize, dust_dds::DdsType)]
    struct a64k {
        ts: u64,
        s: u32,
        xary: [u8; 64*1024 - 12],
    }
    #[derive(Debug, serde::Deserialize, serde::Serialize, dust_dds::DdsType)]
    struct a1M {
        ts: u64,
        s: u32,
        xary: [u8; 1024*1024 - 12],
    }
    #[derive(Debug, serde::Deserialize, serde::Serialize, dust_dds::DdsType)]
    struct a2M {
        ts: u64,
        s: u32,
        xary: [u8; 2*1024*1024 - 12],
    }
    #[derive(Debug, serde::Deserialize, serde::Serialize, dust_dds::DdsType)]
    struct a4M {
        ts: u64,
        s: u32,
        xary: [u8; 4*1024*1024 - 12],
    }
    #[derive(Debug, serde::Deserialize, serde::Serialize, dust_dds::DdsType)]
    struct a8M {
        ts: u64,
        s: u32,
        xary: [u8; 8*1024*1024 - 12],
    }
    #[derive(Debug, serde::Deserialize, serde::Serialize, dust_dds::DdsType)]
    struct seq {
        ts: u64,
        s: u32,
        xseq: Vec<u8>,
    }
}
