use crate::{messages::overall_structure::RtpsMessage, structure::types::Locator};

pub trait TransportWrite<M> {
    fn write(&mut self, message: &RtpsMessage<M>, destination_locator: Locator);
}

pub trait TransportRead<'a, M> {
    fn read(&'a mut self) -> Option<(Locator, RtpsMessage<M>)>;
}
