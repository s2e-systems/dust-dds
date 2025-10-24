use crate::{
    dds_async::data_writer::DataWriterAsync,
    infrastructure::status::{
        LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
        PublicationMatchedStatus,
    },
    runtime::DdsRuntime,
};
use core::future::Future;

/// This trait represents a listener object which can be associated with the [`Publisher`](super::publisher::Publisher) entity.
pub trait PublisherListener<R: DdsRuntime> {
    /// Method that is called when any writer belonging to this publisher reports a liveliness lost status.
    fn on_liveliness_lost(
        &mut self,
        _the_writer: DataWriterAsync<R, ()>,
        _status: LivelinessLostStatus,
    ) -> impl Future<Output = ()> + Send {
        core::future::ready(())
    }

    /// Method that is called when any writer belonging to this publisher reports an offered deadline missed status.
    fn on_offered_deadline_missed(
        &mut self,
        _the_writer: DataWriterAsync<R, ()>,
        _status: OfferedDeadlineMissedStatus,
    ) -> impl Future<Output = ()> + Send {
        core::future::ready(())
    }

    /// Method that is called when any writer belonging to this publisher reports an offered incompatible qos status.
    fn on_offered_incompatible_qos(
        &mut self,
        _the_writer: DataWriterAsync<R, ()>,
        _status: OfferedIncompatibleQosStatus,
    ) -> impl Future<Output = ()> + Send {
        core::future::ready(())
    }

    /// Method that is called when any writer belonging to this publisher reports a publication matched status.
    fn on_publication_matched(
        &mut self,
        _the_writer: DataWriterAsync<R, ()>,
        _status: PublicationMatchedStatus,
    ) -> impl Future<Output = ()> + Send {
        core::future::ready(())
    }
}
