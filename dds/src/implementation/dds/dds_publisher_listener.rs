use crate::{
    implementation::{
        dds::nodes::DataWriterNode,
        utils::actor::{Mail, MailHandler},
    },
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
impl MailHandler<TriggerOnOfferedIncompatibleQos> for DdsPublisherListener {
    async fn handle(
        &mut self,
        mail: TriggerOnOfferedIncompatibleQos,
    ) -> <TriggerOnOfferedIncompatibleQos as Mail>::Result {
        tokio::task::block_in_place(|| {
            self.listener
                .on_offered_incompatible_qos(&mail.the_writer, mail.status)
        });
    }
}

pub struct TriggerOnPublicationMatched {
    the_writer: DataWriterNode,
    status: PublicationMatchedStatus,
}

impl TriggerOnPublicationMatched {
    pub fn new(the_writer: DataWriterNode, status: PublicationMatchedStatus) -> Self {
        Self { the_writer, status }
    }
}

impl Mail for TriggerOnPublicationMatched {
    type Result = ();
}

#[async_trait::async_trait]
impl MailHandler<TriggerOnPublicationMatched> for DdsPublisherListener {
    async fn handle(
        &mut self,
        mail: TriggerOnPublicationMatched,
    ) -> <TriggerOnPublicationMatched as Mail>::Result {
        tokio::task::block_in_place(|| {
            self.listener
                .on_publication_matched(&mail.the_writer, mail.status)
        });
    }
}
