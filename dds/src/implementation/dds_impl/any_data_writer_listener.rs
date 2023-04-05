use crate::{
    infrastructure::status::{
        LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
        PublicationMatchedStatus,
    },
    publication::{data_writer::DataWriter, data_writer_listener::DataWriterListener},
    topic_definition::type_support::{DdsSerialize, DdsType},
};

use super::{node_kind::DataWriterNodeKind, listener_data_writer::ListenerDataWriterNode};

pub trait AnyDataWriterListener {
    fn trigger_on_liveliness_lost(
        &mut self,
        _the_writer: ListenerDataWriterNode,
        status: LivelinessLostStatus,
    );
    fn trigger_on_offered_deadline_missed(
        &mut self,
        _the_writer: ListenerDataWriterNode,
        status: OfferedDeadlineMissedStatus,
    );
    fn trigger_on_offered_incompatible_qos(
        &mut self,
        _the_writer: ListenerDataWriterNode,
        status: OfferedIncompatibleQosStatus,
    );
    fn trigger_on_publication_matched(
        &mut self,
        _the_writer: ListenerDataWriterNode,
        status: PublicationMatchedStatus,
    );
}

impl<Foo> AnyDataWriterListener for Box<dyn DataWriterListener<Foo = Foo> + Send + Sync>
where
    Foo: DdsType + DdsSerialize + 'static,
{
    fn trigger_on_liveliness_lost(
        &mut self,
        the_writer: ListenerDataWriterNode,
        status: LivelinessLostStatus,
    ) {
        self.on_liveliness_lost(
            &DataWriter::new(DataWriterNodeKind::Listener(the_writer)),
            status,
        );
    }

    fn trigger_on_offered_deadline_missed(
        &mut self,
        the_writer: ListenerDataWriterNode,
        status: OfferedDeadlineMissedStatus,
    ) {
        self.on_offered_deadline_missed(
            &DataWriter::new(DataWriterNodeKind::Listener(the_writer)),
            status,
        );
    }

    fn trigger_on_offered_incompatible_qos(
        &mut self,
        the_writer: ListenerDataWriterNode,
        status: OfferedIncompatibleQosStatus,
    ) {
        self.on_offered_incompatible_qos(
            &DataWriter::new(DataWriterNodeKind::Listener(the_writer)),
            status,
        );
    }

    fn trigger_on_publication_matched(
        &mut self,
        the_writer: ListenerDataWriterNode,
        status: PublicationMatchedStatus,
    ) {
        self.on_publication_matched(
            &DataWriter::new(DataWriterNodeKind::Listener(the_writer)),
            status,
        )
    }
}
