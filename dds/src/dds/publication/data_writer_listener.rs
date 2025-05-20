use crate::{
    dcps::runtime::DdsRuntime,
    dds_async::data_writer::DataWriterAsync,
    infrastructure::status::{
        LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
        PublicationMatchedStatus,
    },
};
use core::future::Future;

/// This trait represents a listener object which can be associated with the [`DataWriter`] entity.
pub trait DataWriterListener<R: DdsRuntime, Foo>: 'static {
    /// Method that is called when this writer reports a liveliness lost status.
    fn on_liveliness_lost(
        &mut self,
        _the_writer: DataWriterAsync<R, Foo>,
        _status: LivelinessLostStatus,
    ) -> impl Future<Output = ()> + Send {
        core::future::ready(())
    }

    /// Method that is called when this writer reports an offered deadline missed status.
    fn on_offered_deadline_missed(
        &mut self,
        _the_writer: DataWriterAsync<R, Foo>,
        _status: OfferedDeadlineMissedStatus,
    ) -> impl Future<Output = ()> + Send {
        core::future::ready(())
    }

    /// Method that is called when this writer reports an offered incompatible qos status.
    fn on_offered_incompatible_qos(
        &mut self,
        _the_writer: DataWriterAsync<R, Foo>,
        _status: OfferedIncompatibleQosStatus,
    ) -> impl Future<Output = ()> + Send {
        core::future::ready(())
    }

    /// Method that is called when this writer reports a publication matched status.
    fn on_publication_matched(
        &mut self,
        _the_writer: DataWriterAsync<R, Foo>,
        _status: PublicationMatchedStatus,
    ) -> impl Future<Output = ()> + Send {
        core::future::ready(())
    }
}
