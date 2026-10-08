use crate::{
    rtps_messages::overall_structure::{RtpsMessageWrite, RtpsSubmessageWriteKind},
    transport::{
        interface::WriteMessage,
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
        let len = RtpsMessageWrite::from_submessages(
            self.message_writer.write_buffer_mut(),
            submessages,
            self.guid_prefix,
        )
        .buffer()
        .len();
        self.message_writer.write_message(len, destination_locators);
    }
}
