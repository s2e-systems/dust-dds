use crate::{
    implementation::{
        dds::dds_data_reader::DdsDataReader,
        utils::actor::{ActorAddress, Mail, MailHandler},
    },
    infrastructure::error::DdsResult,
};

impl<T> ActorAddress<DdsDataReader<T>> {
    pub fn enable(&self) -> DdsResult<()> {
        struct Enable;

        impl Mail for Enable {
            type Result = ();
        }

        impl<T> MailHandler<Enable> for DdsDataReader<T> {
            fn handle(&mut self, _mail: Enable) -> <Enable as Mail>::Result {
                self.enable().ok();
            }
        }
        self.send_blocking(Enable)
    }
}
