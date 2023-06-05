use crate::implementation::{
    dds::dds_data_reader::DdsDataReader,
    utils::actor::{MailHandler, Mail},
};

pub struct Enable;

impl Mail for Enable {
    type Result = ();
}

impl<T> MailHandler<Enable> for DdsDataReader<T> {
    fn handle(&mut self, _mail: Enable) -> <Enable as Mail>::Result {
        self.enable().ok();
    }
}
