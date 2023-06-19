use crate::{
    implementation::rtps::{
        messages::overall_structure::{RtpsMessageRead, RtpsSubmessageReadKind},
        types::{GuidPrefix, Locator, ProtocolVersion, VendorId, GUIDPREFIX_UNKNOWN},
    },
    infrastructure::time::{Time, TIME_INVALID},
};

pub struct MessageReceiver<'a> {
    source_version: ProtocolVersion,
    source_vendor_id: VendorId,
    source_guid_prefix: GuidPrefix,
    dest_guid_prefix: GuidPrefix,
    _unicast_reply_locator_list: Vec<Locator>,
    _multicast_reply_locator_list: Vec<Locator>,
    have_timestamp: bool,
    timestamp: Time,
    submessages: std::vec::IntoIter<RtpsSubmessageReadKind<'a>>,
}

impl<'a> Iterator for MessageReceiver<'a> {
    type Item = RtpsSubmessageReadKind<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(submessage) = self.submessages.next() {
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
                        self.timestamp =
                            Time::new(m.timestamp().seconds(), m.timestamp().fraction());
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
        Self {
            source_version: message.header().version(),
            source_vendor_id: message.header().vendor_id(),
            source_guid_prefix: message.header().guid_prefix(),
            dest_guid_prefix: GUIDPREFIX_UNKNOWN,
            _unicast_reply_locator_list: Vec::new(),
            _multicast_reply_locator_list: Vec::new(),
            have_timestamp: false,
            timestamp: TIME_INVALID,
            submessages: message.submessages().into_iter(),
        }
    }

    pub fn _source_version(&self) -> ProtocolVersion {
        self.source_version
    }

    pub fn _source_vendor_id(&self) -> VendorId {
        self.source_vendor_id
    }

    pub fn source_guid_prefix(&self) -> GuidPrefix {
        self.source_guid_prefix
    }

    pub fn _dest_guid_prefix(&self) -> GuidPrefix {
        self.dest_guid_prefix
    }

    pub fn _unicast_reply_locator_list(&self) -> &[Locator] {
        self._unicast_reply_locator_list.as_ref()
    }

    pub fn _multicast_reply_locator_list(&self) -> &[Locator] {
        self._multicast_reply_locator_list.as_ref()
    }

    pub fn source_timestamp(&self) -> Option<Time> {
        if self.have_timestamp {
            Some(self.timestamp)
        } else {
            None
        }
    }
}
