use dust_dds_derive::actor_interface;

use crate::{
    implementation::dds::nodes::DataWriterNode,
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

#[actor_interface]
impl DdsPublisherListener {
    async fn trigger_on_offered_incompatible_qos(
        &mut self,
        the_writer: DataWriterNode,
        status: OfferedIncompatibleQosStatus,
    ) {
        tokio::task::block_in_place(|| {
            self.listener
                .on_offered_incompatible_qos(&the_writer, status)
        });
    }

    async fn trigger_on_publication_matched(
        &mut self,
        the_writer: DataWriterNode,
        status: PublicationMatchedStatus,
    ) {
        tokio::task::block_in_place(|| self.listener.on_publication_matched(&the_writer, status));
    }
}
