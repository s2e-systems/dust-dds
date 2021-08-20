use rust_rtps_pim::structure::types::Locator;

pub trait TransportWrite {
    type Message;
    fn write(&mut self, message: &Self::Message, destination_locator: &Locator);
}

pub trait TransportRead {
    type Message;
    fn read(&mut self) -> Option<(Locator, Self::Message)>;
}
