use crate::{
    implementation::{
        dds::dds_data_reader::DdsDataReader,
        rtps::{
            messages::overall_structure::RtpsMessageRead, stateful_reader::RtpsStatefulReader,
            stateless_reader::RtpsStatelessReader,
        },
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

impl ActorAddress<DdsDataReader<RtpsStatefulReader>> {
    pub fn process_rtps_message(&self, message: RtpsMessageRead) -> DdsResult<()> {
        struct ProcessRtpsMessage {
            message: RtpsMessageRead,
        }

        impl Mail for ProcessRtpsMessage {
            type Result = ();
        }

        impl MailHandler<ProcessRtpsMessage> for DdsDataReader<RtpsStatefulReader> {
            fn handle(&mut self, mail: ProcessRtpsMessage) -> <ProcessRtpsMessage as Mail>::Result {
                self.process_rtps_message(mail.message)
            }
        }

        self.send_blocking(ProcessRtpsMessage { message })
    }
}

impl ActorAddress<DdsDataReader<RtpsStatelessReader>> {
    pub fn process_rtps_message(&self, message: RtpsMessageRead) -> DdsResult<()> {
        struct ProcessRtpsMessage {
            message: RtpsMessageRead,
        }

        impl Mail for ProcessRtpsMessage {
            type Result = ();
        }

        impl MailHandler<ProcessRtpsMessage> for DdsDataReader<RtpsStatelessReader> {
            fn handle(&mut self, mail: ProcessRtpsMessage) -> <ProcessRtpsMessage as Mail>::Result {
                self.process_rtps_message(mail.message)
            }
        }

        self.send_blocking(ProcessRtpsMessage { message })
    }
}
