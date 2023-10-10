use dust_dds_derive::actor_interface;

use crate::{
    implementation::dds::dds_data_writer::DdsDataWriter,
    infrastructure::status::{OfferedIncompatibleQosStatus, PublicationMatchedStatus},
};

use super::any_data_writer_listener::AnyDataWriterListener;

pub struct DataWriterListenerActor {
    listener: Box<dyn AnyDataWriterListener + Send + 'static>,
}

impl DataWriterListenerActor {
    pub fn new(listener: Box<dyn AnyDataWriterListener + Send + 'static>) -> Self {
        Self { listener }
    }
}

#[actor_interface]
impl DataWriterListenerActor {
    async fn trigger_on_offered_incompatible_qos(
        &mut self,
        the_writer: DdsDataWriter,
        status: OfferedIncompatibleQosStatus,
    ) {
        tokio::task::block_in_place(|| {
            self.listener
                .trigger_on_offered_incompatible_qos(the_writer, status)
        });
    }

    async fn trigger_on_publication_matched(
        &mut self,
        the_writer: DdsDataWriter,
        status: PublicationMatchedStatus,
    ) {
        tokio::task::block_in_place(|| {
            self.listener
                .trigger_on_publication_matched(the_writer, status)
        });
    }
}
