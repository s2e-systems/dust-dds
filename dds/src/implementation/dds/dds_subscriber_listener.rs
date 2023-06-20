use crate::{
    implementation::{
        dds::nodes::DataReaderNode,
        utils::actor::{ActorAddress, CommandHandler},
    },
    infrastructure::{error::DdsResult, status::SampleRejectedStatus},
    subscription::subscriber_listener::SubscriberListener,
};

pub struct DdsSubscriberListener {
    listener: Box<dyn SubscriberListener + Send>,
}

impl DdsSubscriberListener {
    pub fn new(listener: Box<dyn SubscriberListener + Send>) -> Self {
        Self { listener }
    }
}

impl ActorAddress<DdsSubscriberListener> {
    pub fn trigger_on_sample_rejected(
        &self,
        the_reader: DataReaderNode,
        status: SampleRejectedStatus,
    ) -> DdsResult<()> {
        struct TriggerOnSampleRejected {
            the_reader: DataReaderNode,
            status: SampleRejectedStatus,
        }

        impl CommandHandler<TriggerOnSampleRejected> for DdsSubscriberListener {
            fn handle(&mut self, mail: TriggerOnSampleRejected) {
                self.listener
                    .on_sample_rejected(&mail.the_reader, mail.status)
            }
        }

        self.send_command(TriggerOnSampleRejected { the_reader, status })
    }
}
