use crate::messages::types::constants::TIME_INVALID;
use crate::messages;
use crate::messages::Endianness;

use super::{Submessage, SubmessageFlag, SubmessageKind, SubmessageHeader};
use super::submessage_elements;

#[derive(PartialEq, Debug)]
pub struct InfoTs {
    endianness_flag: SubmessageFlag,
    invalidate_flag: SubmessageFlag,
    timestamp: submessage_elements::Timestamp,
}

impl InfoTs {
    const INVALID_TIME_FLAG_MASK: u8 = 0x02;

    pub fn new(endianness: Endianness, time: Option<messages::types::Time>) -> InfoTs {
        let endianness_flag = endianness.into();
        let invalidate_flag = !time.is_some();
        let timestamp = match time {
            Some(time) => time,
            None => TIME_INVALID,
        };
        InfoTs {
            endianness_flag,
            invalidate_flag,
            timestamp,
        }
    }

    pub fn from_raw_parts(endianness_flag: SubmessageFlag, invalidate_flag: SubmessageFlag, timestamp: submessage_elements::Timestamp,) -> Self {
        InfoTs {
            endianness_flag,
            invalidate_flag,
            timestamp,
        }
    }

    pub fn endianness_flag(&self) -> SubmessageFlag {
        self.endianness_flag
    }

    pub fn invalidate_flag(&self) -> SubmessageFlag {
        self.invalidate_flag
    }

    pub fn timestamp(&self) -> &submessage_elements::Timestamp {
        &self.timestamp
    }

    pub fn time(&self) -> Option<messages::types::Time> {
        match self.invalidate_flag {
            true => None,
            false => Some(self.timestamp),
        }
    }
}

impl Submessage for InfoTs {
    fn submessage_header(&self, octets_to_next_header: u16) -> SubmessageHeader {
        let submessage_id = SubmessageKind::InfoTimestamp;

        let x = false;
        let e = self.endianness_flag; // Indicates endianness.
        let i = self.invalidate_flag; // Indicates whether subsequent Submessages should be considered as having a timestamp or not.
        // X|X|X|X|X|X|I|E
        let flags = [e, i, x, x, x, x, x, x];

        SubmessageHeader::new(submessage_id, flags, octets_to_next_header)
    }
    
    fn is_valid(&self) -> bool {
        true
    }
}