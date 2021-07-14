use rust_rtps_pim::structure::types::Locator;

pub trait TransportWrite<'a> {
    type Message;
    fn write(&mut self, message: &Self::Message, destination_locator: &Locator);
}

pub trait TransportRead<'a> {
    type Message;
    fn read(&'a mut self) -> Option<(Locator, Self::Message)>;
}
