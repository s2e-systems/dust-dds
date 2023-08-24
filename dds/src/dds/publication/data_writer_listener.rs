use crate::{
    infrastructure::status::{
        LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
        PublicationMatchedStatus,
    },
    topic_definition::type_support::DdsType,
};

use super::data_writer::DataWriter;

pub trait DataWriterListener {
    type Foo: DdsType + serde::Serialize;

    fn on_liveliness_lost(
        &mut self,
        _the_writer: &DataWriter<Self::Foo>,
        _status: LivelinessLostStatus,
    ) {
    }
    fn on_offered_deadline_missed(
        &mut self,
        _the_writer: &DataWriter<Self::Foo>,
        _status: OfferedDeadlineMissedStatus,
    ) {
    }
    fn on_offered_incompatible_qos(
        &mut self,
        _the_writer: &DataWriter<Self::Foo>,
        _status: OfferedIncompatibleQosStatus,
    ) {
    }
    fn on_publication_matched(
        &mut self,
        _the_writer: &DataWriter<Self::Foo>,
        _status: PublicationMatchedStatus,
    ) {
    }
}
