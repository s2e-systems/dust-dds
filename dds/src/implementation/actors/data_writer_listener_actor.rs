use dust_dds_derive::actor_interface;

use crate::{
    implementation::dds::nodes::DataWriterNode,
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

#[actor_interface]
impl DdsDataWriterListener {
    async fn trigger_on_offered_incompatible_qos(
        &mut self,
        the_writer: DataWriterNode,
        status: OfferedIncompatibleQosStatus,
    ) {
        tokio::task::block_in_place(|| {
            self.listener
                .trigger_on_offered_incompatible_qos(the_writer, status)
        });
    }

    async fn trigger_on_publication_matched(
        &mut self,
        the_writer: DataWriterNode,
        status: PublicationMatchedStatus,
    ) {
        tokio::task::block_in_place(|| {
            self.listener
                .trigger_on_publication_matched(the_writer, status)
        });
    }
}
