use rust_rtps_pim::{
    messages::{
        overall_structure::RtpsMessage, submessage_elements::Parameter,
        submessages::RtpsSubmessageType,
    },
    structure::types::{Locator, SequenceNumber},
};
pub type RtpsSubmessageWrite<'a> =
    RtpsSubmessageType<Vec<SequenceNumber>, &'a [Parameter<&'a [u8]>], &'a [u8], (), ()>;
pub type RtpsSubmessageRead<'a> =
    RtpsSubmessageType<Vec<SequenceNumber>, Vec<Parameter<&'a [u8]>>, &'a [u8], (), ()>;

pub type RtpsMessageWrite<'a> = RtpsMessage<Vec<RtpsSubmessageWrite<'a>>>;
pub type RtpsMessageRead<'a> = RtpsMessage<Vec<RtpsSubmessageRead<'a>>>;

pub trait TransportWrite {
    fn write(&mut self, message: &RtpsMessageWrite, destination_locator: &Locator);
}

pub trait TransportRead {
    fn read(&mut self) -> Option<(Locator, RtpsMessageRead)>;
}
