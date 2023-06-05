use crate::implementation::{
    dds::dds_data_reader::DdsDataReader,
    utils::actor::{Handler, Message},
};

pub struct Enable;

impl Message for Enable {
    type Result = ();
}

impl<T> Handler<Enable> for DdsDataReader<T> {
    fn handle(&mut self, _mail: Enable) -> <Enable as Message>::Result {
        self.enable().ok();
    }
}
