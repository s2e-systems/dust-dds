use crate::implementation::{
    dds::dds_topic::DdsTopic,
    utils::actor::{Handler, Message},
};

pub struct GetName;

impl Message for GetName {
    type Result = String;
}

impl Handler<GetName> for DdsTopic {
    fn handle(&mut self, _mail: GetName) -> <GetName as Message>::Result {
        self.get_name()
    }
}

pub struct Enable;

impl Message for Enable {
    type Result = ();
}

impl Handler<Enable> for DdsTopic {
    fn handle(&mut self, _mail: Enable) -> <Enable as Message>::Result {
        self.enable().ok();
    }
}
