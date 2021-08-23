use rust_rtps_pim::{
    messages::{submessage_elements::Parameter, submessages::RtpsSubmessageType, RtpsMessage},
    structure::types::{Locator, SequenceNumber},
};

pub type TransportMessage<'a> =
    RtpsMessage<Vec<RtpsSubmessageType<'a, Vec<SequenceNumber>, &'a [Parameter<'a>], (), ()>>>;

pub trait TransportWrite {
    fn write(&mut self, message: &TransportMessage<'_>, destination_locator: &Locator);
}

pub trait TransportRead {
    fn read(&mut self) -> Option<(Locator, RtpsMessage<Vec<RtpsSubmessageType<'_, Vec<SequenceNumber>, Vec<Parameter<'_>>, (), ()>>>)>;
}
