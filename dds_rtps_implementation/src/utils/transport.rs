use rust_rtps_pim::{
    messages::{submessage_elements::Parameter, submessages::RtpsSubmessageType, RtpsMessage},
    structure::types::{Locator, SequenceNumber},
};
pub type RtpsSubmessageWrite<'a> =
    RtpsSubmessageType<'a, Vec<SequenceNumber>, &'a [Parameter<'a>], (), ()>;
pub type RtpsSubmessageRead<'a> =
    RtpsSubmessageType<'a, Vec<SequenceNumber>, Vec<Parameter<'a>>, (), ()>;

pub type RtpsMessageWrite<'a> =
    RtpsMessage<Vec<RtpsSubmessageWrite<'a>>>;
pub type RtpsMessageRead<'a> =
    RtpsMessage<Vec<RtpsSubmessageRead<'a>>>;

pub trait TransportWrite {
    fn write(&mut self, message: &RtpsMessageWrite, destination_locator: &Locator);
}

pub trait TransportRead {
    fn read(&mut self) -> Option<(Locator, RtpsMessageRead)>;
}
