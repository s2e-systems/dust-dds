use crate::{
    implementation::{
        dds::nodes::DataWriterNode,
        utils::actor::{ActorAddress, CommandHandler},
    },
    infrastructure::{error::DdsResult, status::OfferedIncompatibleQosStatus},
    publication::publisher_listener::PublisherListener,
};

pub struct DdsPublisherListener {
    listener: Box<dyn PublisherListener + Send>,
}

impl DdsPublisherListener {
    pub fn new(listener: Box<dyn PublisherListener + Send>) -> Self {
        Self { listener }
    }
}

impl ActorAddress<DdsPublisherListener> {
    pub fn trigger_on_offered_incompatible_qos(
        &self,
        the_writer: DataWriterNode,
        status: OfferedIncompatibleQosStatus,
    ) -> DdsResult<()> {
        struct OnOfferedIncompatibleQos {
            the_writer: DataWriterNode,
            status: OfferedIncompatibleQosStatus,
        }

        impl CommandHandler<OnOfferedIncompatibleQos> for DdsPublisherListener {
            fn handle(&mut self, mail: OnOfferedIncompatibleQos) {
                self.listener
                    .on_offered_incompatible_qos(&mail.the_writer, mail.status)
            }
        }

        self.send_command(OnOfferedIncompatibleQos { the_writer, status })
    }
}
