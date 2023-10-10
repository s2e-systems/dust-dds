use crate::{
    implementation::dds::dds_data_writer::DdsDataWriter,
    infrastructure::status::{
        LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
        PublicationMatchedStatus,
    },
    publication::{data_writer::DataWriter, data_writer_listener::DataWriterListener},
};

pub trait AnyDataWriterListener {
    fn trigger_on_liveliness_lost(
        &mut self,
        _the_writer: DdsDataWriter,
        status: LivelinessLostStatus,
    );
    fn trigger_on_offered_deadline_missed(
        &mut self,
        _the_writer: DdsDataWriter,
        status: OfferedDeadlineMissedStatus,
    );
    fn trigger_on_offered_incompatible_qos(
        &mut self,
        _the_writer: DdsDataWriter,
        status: OfferedIncompatibleQosStatus,
    );
    fn trigger_on_publication_matched(
        &mut self,
        _the_writer: DdsDataWriter,
        status: PublicationMatchedStatus,
    );
}

impl<Foo> AnyDataWriterListener for Box<dyn DataWriterListener<Foo> + Send> {
    fn trigger_on_liveliness_lost(
        &mut self,
        the_writer: DdsDataWriter,
        status: LivelinessLostStatus,
    ) {
        self.on_liveliness_lost(&DataWriter::new(the_writer), status);
    }

    fn trigger_on_offered_deadline_missed(
        &mut self,
        the_writer: DdsDataWriter,
        status: OfferedDeadlineMissedStatus,
    ) {
        self.on_offered_deadline_missed(&DataWriter::new(the_writer), status);
    }

    fn trigger_on_offered_incompatible_qos(
        &mut self,
        the_writer: DdsDataWriter,
        status: OfferedIncompatibleQosStatus,
    ) {
        self.on_offered_incompatible_qos(&DataWriter::new(the_writer), status);
    }

    fn trigger_on_publication_matched(
        &mut self,
        the_writer: DdsDataWriter,
        status: PublicationMatchedStatus,
    ) {
        self.on_publication_matched(&DataWriter::new(the_writer), status)
    }
}
