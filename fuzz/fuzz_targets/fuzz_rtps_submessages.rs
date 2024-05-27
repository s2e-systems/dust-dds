#![no_main]

use dust_dds::rtps::messages::{
    overall_structure::SubmessageHeaderRead,
    submessages::{
        data::DataSubmessage, gap::GapSubmessage, heartbeat::HeartbeatSubmessage,
        heartbeat_frag::HeartbeatFragSubmessage,
    },
};
use libfuzzer_sys::{fuzz_target, Corpus};

fuzz_target!(|data: &[u8]| -> Corpus {
    if data.len() > 4 {
        let mut data = data;
        let header = SubmessageHeaderRead::try_read_from_bytes(&mut data).unwrap();
        // HeartbeatSubmessage::try_from_bytes(&header, data).ok();
        // GapSubmessage::try_from_bytes(&header, data).ok();
        DataSubmessage::try_from_arc_slice(&header, data.into()).ok();
        Corpus::Keep
    } else {
        Corpus::Reject
    }
});
