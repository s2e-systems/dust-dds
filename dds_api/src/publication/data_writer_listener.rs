use crate::{
    dcps_psm::{
        LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
        PublicationMatchedStatus,
    },
    dds_type::DDSType,
    infrastructure::listener::Listener,
};

use super::data_writer::DataWriter;

pub trait DataWriterListener<T: DDSType>: Listener {
    fn on_liveliness_lost(&self, the_writer: &dyn DataWriter<T>, status: LivelinessLostStatus);
    fn on_offered_deadline_missed(
        &self,
        the_writer: &dyn DataWriter<T>,
        status: OfferedDeadlineMissedStatus,
    );
    fn on_offered_incompatible_qos(
        &self,
        the_writer: &dyn DataWriter<T>,
        status: OfferedIncompatibleQosStatus,
    );
    fn on_publication_matched(
        &self,
        the_writer: &dyn DataWriter<T>,
        status: PublicationMatchedStatus,
    );
}
