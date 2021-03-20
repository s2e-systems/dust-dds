use crate::messages::submessage_elements;
use rust_rtps_pim::messages::{
    submessages::{info_timestamp_submessage, Submessage},
    types::SubmessageFlag,
};

use super::SubmessageHeader;

pub struct InfoTimestamp {
    endianness_flag: SubmessageFlag,
    invalidate_flag: SubmessageFlag,
    timestamp: <Self as info_timestamp_submessage::InfoTimestamp>::Timestamp,
}

impl Submessage for InfoTimestamp {
    type SubmessageHeader = SubmessageHeader;

    fn submessage_header(&self) -> Self::SubmessageHeader {
        todo!()
    }

    fn is_valid(&self) -> bool {
        todo!()
    }
}

impl rust_rtps_pim::messages::submessages::info_timestamp_submessage::InfoTimestamp
    for InfoTimestamp
{
    type Timestamp = submessage_elements::Timestamp;

    fn endianness_flag(&self) -> SubmessageFlag {
        self.endianness_flag
    }

    fn invalidate_flag(&self) -> SubmessageFlag {
        self.invalidate_flag
    }

    fn timestamp(&self) -> &Self::Timestamp {
        &self.timestamp
    }
}
