use crate::{
    implementation::{
        dds::dds_topic::DdsTopic,
        utils::actor::{ActorAddress, Mail, MailHandler},
    },
    infrastructure::error::DdsResult,
};

impl ActorAddress<DdsTopic> {
    pub fn get_name(&self) -> DdsResult<String> {
        struct GetName;

        impl Mail for GetName {
            type Result = String;
        }

        impl MailHandler<GetName> for DdsTopic {
            fn handle(&mut self, _mail: GetName) -> <GetName as Mail>::Result {
                self.get_name()
            }
        }
        self.send_blocking(GetName)
    }

    pub fn enable(&self) -> DdsResult<()> {
        struct Enable;

        impl Mail for Enable {
            type Result = ();
        }

        impl MailHandler<Enable> for DdsTopic {
            fn handle(&mut self, _mail: Enable) -> <Enable as Mail>::Result {
                self.enable().ok();
            }
        }

        self.send_blocking(Enable)
    }
}
