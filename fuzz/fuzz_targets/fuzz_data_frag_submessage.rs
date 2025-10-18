#![no_main]

use dust_dds::rtps_messages::{
    overall_structure::SubmessageHeaderRead, submessages::data_frag::DataFragSubmessage,
};
use libfuzzer_sys::{Corpus, fuzz_target};

fuzz_target!(|data: &[u8]| -> Corpus {
    if data.len() > 4 {
        let mut data = data;
        let header = SubmessageHeaderRead::try_read_from_bytes(&mut data).unwrap();
        DataFragSubmessage::try_from_bytes(&header, data).ok();
        Corpus::Keep
    } else {
        Corpus::Reject
    }
});
