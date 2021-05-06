use crate::{
    dcps_psm::{
        LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
        PublicationMatchedStatus,
    },
    infrastructure::listener::Listener,
};

use super::data_writer::DataWriter;

pub trait DataWriterListener: Listener {
    type DataType;
    fn on_liveliness_lost(&self, the_writer: &dyn DataWriter<Self::DataType>, status: LivelinessLostStatus) ;
    fn on_offered_deadline_missed(
        &self,
        the_writer: &dyn DataWriter<Self::DataType>,
        status: OfferedDeadlineMissedStatus,
    );
    fn on_offered_incompatible_qos(
        &self,
        the_writer: &dyn DataWriter<Self::DataType>,
        status: OfferedIncompatibleQosStatus,
    );
    fn on_publication_matched(
        &self,
        the_writer: &dyn DataWriter<Self::DataType>,
        status: PublicationMatchedStatus,
    );
}
