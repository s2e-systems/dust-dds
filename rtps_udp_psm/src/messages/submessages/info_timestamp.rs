use rust_rtps_pim::messages::submessages::{submessage_elements, SubmessageHeader};

use crate::messages::types::{SubmessageFlag, SubmessageKind, Time};

pub struct InfoTimestamp {
    endianness_flag: SubmessageFlag,
    invalidate_flag: SubmessageFlag,
    timestamp: submessage_elements::Timestamp<Time>,
}

impl rust_rtps_pim::messages::submessages::info_timestamp_submessage::InfoTimestamp
    for InfoTimestamp
{
    type Time = Time;

    fn new(
        endianness_flag: SubmessageFlag,
        invalidate_flag: SubmessageFlag,
        timestamp: submessage_elements::Timestamp<Self::Time>,
    ) -> Self {
        Self {
            endianness_flag,
            invalidate_flag,
            timestamp,
        }
    }

    fn endianness_flag(&self) -> SubmessageFlag {
        self.endianness_flag
    }

    fn invalidate_flag(&self) -> SubmessageFlag {
        self.invalidate_flag
    }

    fn timestamp(&self) -> &submessage_elements::Timestamp<Self::Time> {
        &self.timestamp
    }
}

impl rust_rtps_pim::messages::submessages::Submessage for InfoTimestamp {
    type SubmessageKind = SubmessageKind;
    type SubmessageFlag = SubmessageFlag;

    fn submessage_header(&self) -> SubmessageHeader<Self::SubmessageKind, Self::SubmessageFlag> {
        todo!()
    }
}
