use crate::{
    rtps_messages::overall_structure::{RtpsMessageWrite, RtpsSubmessageWriteKind},
    transport::{
        interface::{WritableMessage, WriteMessage},
        types::{GuidPrefix, Locator},
    },
};

pub trait SubmessageWriter {
    fn write_submessages(
        &mut self,
        submessages: &[RtpsSubmessageWriteKind],
        destination_locators: &[Locator],
    );
}

impl<F> SubmessageWriter for F
where
    F: FnMut(&[RtpsSubmessageWriteKind], &[Locator]),
{
    fn write_submessages(
        &mut self,
        submessages: &[RtpsSubmessageWriteKind],
        destination_locators: &[Locator],
    ) {
        self(submessages, destination_locators);
    }
}

pub struct RtpsMessageWritePayload<'a> {
    submessages: &'a [RtpsSubmessageWriteKind<'a>],
    guid_prefix: GuidPrefix,
}

impl<'a> RtpsMessageWritePayload<'a> {
    pub fn new(submessages: &'a [RtpsSubmessageWriteKind<'a>], guid_prefix: GuidPrefix) -> Self {
        Self {
            submessages,
            guid_prefix,
        }
    }
}

impl WritableMessage for RtpsMessageWritePayload<'_> {
    fn write_into_buffer(&self, buf: &mut [u8]) -> usize {
        RtpsMessageWrite::from_submessages(buf, self.submessages, self.guid_prefix)
            .buffer()
            .len()
    }
}

pub struct TransportSubmessageWriter<'a, W: ?Sized> {
    message_writer: &'a mut W,
    guid_prefix: GuidPrefix,
}

impl<'a, W: WriteMessage + ?Sized> TransportSubmessageWriter<'a, W> {
    pub fn new(message_writer: &'a mut W, guid_prefix: GuidPrefix) -> Self {
        Self {
            message_writer,
            guid_prefix,
        }
    }
}

impl<W: WriteMessage + ?Sized> SubmessageWriter for TransportSubmessageWriter<'_, W> {
    fn write_submessages(
        &mut self,
        submessages: &[RtpsSubmessageWriteKind],
        destination_locators: &[Locator],
    ) {
        let message = RtpsMessageWritePayload::new(submessages, self.guid_prefix);
        self.message_writer
            .write_message(destination_locators, &message);
    }
}
