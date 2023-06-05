use crate::implementation::{
    dds::dds_topic::DdsTopic,
    utils::actor::{MailHandler, Mail},
};

pub struct GetName;

impl Mail for GetName {
    type Result = String;
}

impl MailHandler<GetName> for DdsTopic {
    fn handle(&mut self, _mail: GetName) -> <GetName as Mail>::Result {
        self.get_name()
    }
}

pub struct Enable;

impl Mail for Enable {
    type Result = ();
}

impl MailHandler<Enable> for DdsTopic {
    fn handle(&mut self, _mail: Enable) -> <Enable as Mail>::Result {
        self.enable().ok();
    }
}
