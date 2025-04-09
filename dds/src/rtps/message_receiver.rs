use super::messages::{
    self,
    overall_structure::{RtpsMessageRead, RtpsSubmessageReadKind},
    types::TIME_INVALID,
};
use crate::transport::types::{GuidPrefix, Locator, ProtocolVersion, VendorId, GUIDPREFIX_UNKNOWN};

pub struct MessageReceiver<'a> {
    source_version: ProtocolVersion,
    source_vendor_id: VendorId,
    source_guid_prefix: GuidPrefix,
    dest_guid_prefix: GuidPrefix,
    _unicast_reply_locator_list: Vec<Locator>,
    _multicast_reply_locator_list: Vec<Locator>,
    have_timestamp: bool,
    timestamp: messages::types::Time,
    iter: std::slice::Iter<'a, RtpsSubmessageReadKind>,
}

impl<'a> Iterator for MessageReceiver<'a> {
    type Item = &'a RtpsSubmessageReadKind;

    fn next(&mut self) -> Option<Self::Item> {
        for submessage in self.iter.by_ref() {
            match &submessage {
                RtpsSubmessageReadKind::AckNack(_)
                | RtpsSubmessageReadKind::Data(_)
                | RtpsSubmessageReadKind::DataFrag(_)
                | RtpsSubmessageReadKind::Gap(_)
                | RtpsSubmessageReadKind::Heartbeat(_)
                | RtpsSubmessageReadKind::HeartbeatFrag(_)
                | RtpsSubmessageReadKind::NackFrag(_) => return Some(submessage),

                RtpsSubmessageReadKind::InfoDestination(m) => {
                    self.dest_guid_prefix = m.guid_prefix();
                }
                RtpsSubmessageReadKind::InfoReply(_) => todo!(),
                RtpsSubmessageReadKind::InfoSource(m) => {
                    self.source_vendor_id = m.vendor_id();
                    self.source_version = m.protocol_version();
                    self.source_guid_prefix = m.guid_prefix();
                }
                RtpsSubmessageReadKind::InfoTimestamp(m) => {
                    if !m.invalidate_flag() {
                        self.have_timestamp = true;
                        self.timestamp = m.timestamp();
                    } else {
                        self.have_timestamp = false;
                        self.timestamp = TIME_INVALID;
                    }
                }
                RtpsSubmessageReadKind::Pad(_) => (),
            }
        }
        None
    }
}

impl<'a> MessageReceiver<'a> {
    pub fn new(message: &'a RtpsMessageRead) -> Self {
        let header = message.header();
        Self {
            source_version: header.version(),
            source_vendor_id: header.vendor_id(),
            source_guid_prefix: header.guid_prefix(),
            dest_guid_prefix: GUIDPREFIX_UNKNOWN,
            _unicast_reply_locator_list: Vec::new(),
            _multicast_reply_locator_list: Vec::new(),
            have_timestamp: false,
            timestamp: TIME_INVALID,
            iter: message.submessages().iter(),
        }
    }

    pub fn source_guid_prefix(&self) -> GuidPrefix {
        self.source_guid_prefix
    }

    pub fn source_timestamp(&self) -> Option<messages::types::Time> {
        if self.have_timestamp {
            Some(self.timestamp)
        } else {
            None
        }
    }
}
