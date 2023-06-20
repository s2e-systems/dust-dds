use crate::{
    implementation::{
        dds::nodes::DataWriterNode,
        utils::actor::{ActorAddress, CommandHandler},
    },
    infrastructure::{error::DdsResult, status::OfferedIncompatibleQosStatus},
};

use super::any_data_writer_listener::AnyDataWriterListener;

pub struct DdsDataWriterListener {
    listener: Box<dyn AnyDataWriterListener + Send + 'static>,
}

impl DdsDataWriterListener {
    pub fn new(listener: Box<dyn AnyDataWriterListener + Send + 'static>) -> Self {
        Self { listener }
    }
}

impl ActorAddress<DdsDataWriterListener> {
    pub fn trigger_on_offered_incompatible_qos(
        &self,
        the_writer: DataWriterNode,
        status: OfferedIncompatibleQosStatus,
    ) -> DdsResult<()> {
        struct OnOfferedIncompatibleQos {
            the_writer: DataWriterNode,
            status: OfferedIncompatibleQosStatus,
        }

        impl CommandHandler<OnOfferedIncompatibleQos> for DdsDataWriterListener {
            fn handle(&mut self, mail: OnOfferedIncompatibleQos) {
                self.listener
                    .trigger_on_offered_incompatible_qos(mail.the_writer, mail.status)
            }
        }

        self.send_command(OnOfferedIncompatibleQos { the_writer, status })
    }
}
