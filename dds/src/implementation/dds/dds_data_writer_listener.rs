use crate::{
    implementation::{
        dds::nodes::DataWriterNode,
        utils::actor::{actor_command_interface, Mail, MailHandler},
    },
    infrastructure::status::{OfferedIncompatibleQosStatus, PublicationMatchedStatus},
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

pub struct TriggerOnOfferedIncompatibleQos {
    the_writer: DataWriterNode,
    status: OfferedIncompatibleQosStatus,
}

impl TriggerOnOfferedIncompatibleQos {
    pub fn new(the_writer: DataWriterNode, status: OfferedIncompatibleQosStatus) -> Self {
        Self { the_writer, status }
    }
}

impl Mail for TriggerOnOfferedIncompatibleQos {
    type Result = ();
}

#[async_trait::async_trait]
impl MailHandler<TriggerOnOfferedIncompatibleQos> for DdsDataWriterListener {
    async fn handle(
        &mut self,
        mail: TriggerOnOfferedIncompatibleQos,
    ) -> <TriggerOnOfferedIncompatibleQos as Mail>::Result {
        tokio::task::block_in_place(|| {
            self.listener
                .trigger_on_offered_incompatible_qos(mail.the_writer, mail.status)
        });
    }
}

actor_command_interface! {
impl DdsDataWriterListener {

    pub fn trigger_on_publication_matched(
        &mut self,
        the_writer: DataWriterNode,
        status: PublicationMatchedStatus,
    ) {
        tokio::task::block_in_place(|| self.listener
            .trigger_on_publication_matched(the_writer, status));
    }
}
}
