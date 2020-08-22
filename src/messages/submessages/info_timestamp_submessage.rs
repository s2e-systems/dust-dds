use crate::messages::serdes::{SubmessageElement, Endianness, RtpsSerdesResult, };
use super::{SubmessageKind, SubmessageFlag, UdpPsmMapping, };
use super::{Submessage, SubmessageHeader, };
use super::submessage_elements;
use crate::messages;

#[derive(PartialEq, Debug)]
pub struct InfoTs {
    endianness_flag: SubmessageFlag,
    invalidate_flag: SubmessageFlag,
    timestamp: Option<submessage_elements::Timestamp>,
}

impl InfoTs {
    const INVALID_TIME_FLAG_MASK: u8 = 0x02;

    pub fn new(time: Option<messages::types::Time>, endianness: Endianness) -> InfoTs {
        let endianness_flag = endianness.into();
        let invalidate_flag = !time.is_some();
        let timestamp = match time {
            Some(time) => Some(submessage_elements::Timestamp(time)),
            None => None,
        };
        InfoTs {
            endianness_flag,
            invalidate_flag,
            timestamp,
        }
    }

    pub fn time(&self) -> Option<messages::types::Time> {
        match self.invalidate_flag {
            true => None,
            false => Some((&self.timestamp).as_ref().unwrap().0),
        }
    }
}

impl Submessage for InfoTs {
    fn submessage_flags(&self) -> [SubmessageFlag; 8] {
        let x = false;
        let e = self.endianness_flag; // Indicates endianness.
        let i = self.invalidate_flag; // Indicates whether subsequent Submessages should be considered as having a timestamp or not.
        // X|X|X|X|X|X|I|E
        [e, i, x, x, x, x, x, x]
    }
    
    fn is_valid(&self) -> bool {
        true
    }
}