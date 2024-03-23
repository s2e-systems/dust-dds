use std::{future::Future, pin::Pin};

use crate::{
    dds_async::{data_writer::DataWriterAsync, data_writer_listener::DataWriterListenerAsync},
    infrastructure::status::{
        LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
        PublicationMatchedStatus,
    },
};

use super::data_writer::DataWriter;

/// This trait represents a listener object which can be associated with the [`DataWriter`] entity.
pub trait DataWriterListener: 'static {
    /// Type of the DataWriter with which this Listener will be associated.
    type Foo;

    /// Method that is called when this writer reports a liveliness lost status.
    fn on_liveliness_lost(
        &mut self,
        _the_writer: DataWriter<Self::Foo>,
        _status: LivelinessLostStatus,
    ) {
    }
    /// Method that is called when this writer reports an offered deadline missed status.
    fn on_offered_deadline_missed(
        &mut self,
        _the_writer: DataWriter<Self::Foo>,
        _status: OfferedDeadlineMissedStatus,
    ) {
    }
    /// Method that is called when this writer reports an offered incompatible qos status.
    fn on_offered_incompatible_qos(
        &mut self,
        _the_writer: DataWriter<Self::Foo>,
        _status: OfferedIncompatibleQosStatus,
    ) {
    }
    /// Method that is called when this writer reports a publication matched status.
    fn on_publication_matched(
        &mut self,
        _the_writer: DataWriter<Self::Foo>,
        _status: PublicationMatchedStatus,
    ) {
    }
}

impl<'a, Foo> DataWriterListenerAsync for Box<dyn DataWriterListener<Foo = Foo> + Send + 'a>
where
    Self: 'static,
{
    type Foo = Foo;

    fn on_liveliness_lost(
        &mut self,
        _the_writer: DataWriterAsync<Self::Foo>,
        _status: LivelinessLostStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async {})
    }

    fn on_offered_deadline_missed(
        &mut self,
        _the_writer: DataWriterAsync<Self::Foo>,
        _status: OfferedDeadlineMissedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async {})
    }

    fn on_offered_incompatible_qos(
        &mut self,
        _the_writer: DataWriterAsync<Self::Foo>,
        _status: OfferedIncompatibleQosStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async {})
    }

    fn on_publication_matched(
        &mut self,
        _the_writer: DataWriterAsync<Self::Foo>,
        _status: PublicationMatchedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async {})
    }

    // fn on_liveliness_lost(
    //     &mut self,
    //     the_writer: DataWriterAsync<Self::Foo>,
    //     status: LivelinessLostStatus,
    // ) -> impl Future<Output = ()> + Send {
    //     tokio::task::block_in_place(|| {
    //         self.0
    //             .on_liveliness_lost(DataWriter::new(the_writer), status)
    //     });
    //     async {}
    // }

    // fn on_offered_deadline_missed(
    //     &mut self,
    //     the_writer: DataWriterAsync<Self::Foo>,
    //     status: OfferedDeadlineMissedStatus,
    // ) -> impl Future<Output = ()> + Send {
    //     tokio::task::block_in_place(|| {
    //         self.0
    //             .on_offered_deadline_missed(DataWriter::new(the_writer), status)
    //     });
    //     async {}
    // }

    // fn on_offered_incompatible_qos(
    //     &mut self,
    //     the_writer: DataWriterAsync<Self::Foo>,
    //     status: OfferedIncompatibleQosStatus,
    // ) -> impl Future<Output = ()> + Send {
    //     tokio::task::block_in_place(|| {
    //         self.0
    //             .on_offered_incompatible_qos(DataWriter::new(the_writer), status)
    //     });
    //     async {}
    // }

    // fn on_publication_matched(
    //     &mut self,
    //     the_writer: DataWriterAsync<Self::Foo>,
    //     status: PublicationMatchedStatus,
    // ) -> impl Future<Output = ()> + Send {
    //     tokio::task::block_in_place(|| {
    //         self.0
    //             .on_publication_matched(DataWriter::new(the_writer), status)
    //     });
    //     async {}
    // }
}
