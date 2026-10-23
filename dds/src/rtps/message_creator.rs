use crate::{
    rtps_messages::{
        error::RtpsMessageResult,
        overall_structure::{
            RtpsMessageHeader, RtpsMessageWrite, Submessage, Write, write_rtps_message,
        },
    },
    transport::types::GuidPrefix,
};

use super::types::{PROTOCOLVERSION_2_4, VENDOR_ID_S2E};

struct TransportWrite<'a, W: ?Sized>(&'a mut W);

impl<W: crate::transport::interface::Write + ?Sized> Write for TransportWrite<'_, W> {
    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> RtpsMessageResult<()> {
        self.0.write(buf);
        Ok(())
    }
}

pub fn write_submessages_into_transport(
    submessages: &[&(dyn Submessage + Send)],
    guid_prefix: GuidPrefix,
    writer: &mut (impl crate::transport::interface::Write + ?Sized),
) {
    let header = RtpsMessageHeader::new(PROTOCOLVERSION_2_4, VENDOR_ID_S2E, guid_prefix);
    write_rtps_message(&header, submessages, &mut TransportWrite(writer));
}

impl<'a> RtpsMessageWrite<'a> {
    pub fn from_submessages(
        buffer: &'a mut [u8],
        submessages: &[&(dyn Submessage + Send)],
        guid_prefix: GuidPrefix,
    ) -> Self {
        let header = RtpsMessageHeader::new(PROTOCOLVERSION_2_4, VENDOR_ID_S2E, guid_prefix);
        RtpsMessageWrite::new(buffer, &header, submessages)
    }
}
