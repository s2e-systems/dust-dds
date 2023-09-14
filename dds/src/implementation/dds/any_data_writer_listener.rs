use crate::{
    infrastructure::status::{
        LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
        PublicationMatchedStatus,
    },
    publication::{data_writer::DataWriter, data_writer_listener::DataWriterListener},
};

use super::nodes::{DataWriterNode, DataWriterNodeKind};

pub trait AnyDataWriterListener {
    fn trigger_on_liveliness_lost(
        &mut self,
        _the_writer: DataWriterNode,
        status: LivelinessLostStatus,
    );
    fn trigger_on_offered_deadline_missed(
        &mut self,
        _the_writer: DataWriterNode,
        status: OfferedDeadlineMissedStatus,
    );
    fn trigger_on_offered_incompatible_qos(
        &mut self,
        _the_writer: DataWriterNode,
        status: OfferedIncompatibleQosStatus,
    );
    fn trigger_on_publication_matched(
        &mut self,
        _the_writer: DataWriterNode,
        status: PublicationMatchedStatus,
    );
}

impl<Foo> AnyDataWriterListener for Box<dyn DataWriterListener<Foo> + Send + Sync> {
    fn trigger_on_liveliness_lost(
        &mut self,
        the_writer: DataWriterNode,
        status: LivelinessLostStatus,
    ) {
        self.on_liveliness_lost(
            &DataWriter::new(DataWriterNodeKind::Listener(the_writer)),
            status,
        );
    }

    fn trigger_on_offered_deadline_missed(
        &mut self,
        the_writer: DataWriterNode,
        status: OfferedDeadlineMissedStatus,
    ) {
        self.on_offered_deadline_missed(
            &DataWriter::new(DataWriterNodeKind::Listener(the_writer)),
            status,
        );
    }

    fn trigger_on_offered_incompatible_qos(
        &mut self,
        the_writer: DataWriterNode,
        status: OfferedIncompatibleQosStatus,
    ) {
        self.on_offered_incompatible_qos(
            &DataWriter::new(DataWriterNodeKind::Listener(the_writer)),
            status,
        );
    }

    fn trigger_on_publication_matched(
        &mut self,
        the_writer: DataWriterNode,
        status: PublicationMatchedStatus,
    ) {
        self.on_publication_matched(
            &DataWriter::new(DataWriterNodeKind::Listener(the_writer)),
            status,
        )
    }
}
