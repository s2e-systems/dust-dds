use crate::{
    implementation::utils::shared_object::DdsShared,
    infrastructure::status::{
        LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
        PublicationMatchedStatus,
    },
    publication::{data_writer::DataWriter, data_writer_listener::DataWriterListener},
    topic_definition::type_support::{DdsSerialize, DdsType},
};

use super::user_defined_data_writer::UserDefinedDataWriter;

pub trait AnyDataWriterListener {
    fn trigger_on_liveliness_lost(
        &mut self,
        _the_writer: &DdsShared<UserDefinedDataWriter>,
        _status: LivelinessLostStatus,
    );
    fn trigger_on_offered_deadline_missed(
        &mut self,
        _the_writer: &DdsShared<UserDefinedDataWriter>,
        _status: OfferedDeadlineMissedStatus,
    );
    fn trigger_on_offered_incompatible_qos(
        &mut self,
        _the_writer: &DdsShared<UserDefinedDataWriter>,
        _status: OfferedIncompatibleQosStatus,
    );
    fn trigger_on_publication_matched(
        &mut self,
        _the_writer: &DdsShared<UserDefinedDataWriter>,
        _status: PublicationMatchedStatus,
    );
}

impl<Foo> AnyDataWriterListener for Box<dyn DataWriterListener<Foo = Foo> + Send + Sync>
where
    Foo: DdsType + DdsSerialize + 'static,
{
    fn trigger_on_liveliness_lost(
        &mut self,
        the_writer: &DdsShared<UserDefinedDataWriter>,
        status: LivelinessLostStatus,
    ) {
        self.on_liveliness_lost(&DataWriter::new(the_writer.downgrade()), status);
    }

    fn trigger_on_offered_deadline_missed(
        &mut self,
        the_writer: &DdsShared<UserDefinedDataWriter>,
        status: OfferedDeadlineMissedStatus,
    ) {
        self.on_offered_deadline_missed(&DataWriter::new(the_writer.downgrade()), status);
    }

    fn trigger_on_offered_incompatible_qos(
        &mut self,
        the_writer: &DdsShared<UserDefinedDataWriter>,
        status: OfferedIncompatibleQosStatus,
    ) {
        self.on_offered_incompatible_qos(&DataWriter::new(the_writer.downgrade()), status);
    }

    fn trigger_on_publication_matched(
        &mut self,
        the_writer: &DdsShared<UserDefinedDataWriter>,
        status: PublicationMatchedStatus,
    ) {
        self.on_publication_matched(&DataWriter::new(the_writer.downgrade()), status)
    }
}
