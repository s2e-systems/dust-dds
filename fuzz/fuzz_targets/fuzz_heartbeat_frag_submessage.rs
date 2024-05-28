#![no_main]

use dust_dds::rtps::messages::{
    overall_structure::SubmessageHeaderRead, submessages::heartbeat_frag::HeartbeatFragSubmessage,
};
use libfuzzer_sys::{fuzz_target, Corpus};

fuzz_target!(|data: &[u8]| -> Corpus {
    if data.len() > 4 {
        let mut data = data;
        let header = SubmessageHeaderRead::try_read_from_bytes(&mut data).unwrap();
        HeartbeatFragSubmessage::try_from_bytes(&header, data).ok();
        Corpus::Keep
    } else {
        Corpus::Reject
    }
});
