use crate::{
    implementation::{dds::nodes::DataWriterNode, utils::actor::actor_command_interface},
    infrastructure::status::{OfferedIncompatibleQosStatus, PublicationMatchedStatus},
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

actor_command_interface! {
impl DdsPublisherListener {
    pub fn trigger_on_offered_incompatible_qos(
        &mut self,
        the_writer: DataWriterNode,
        status: OfferedIncompatibleQosStatus,
    ) {
        tokio::task::block_in_place(|| self.listener
            .on_offered_incompatible_qos(&the_writer, status));
    }

    pub fn trigger_on_publication_matched(
        &mut self,
        the_writer: DataWriterNode,
        status: PublicationMatchedStatus,
    ) {
        tokio::task::block_in_place(|| self.listener
        .on_publication_matched(&the_writer, status));
    }
}
}
