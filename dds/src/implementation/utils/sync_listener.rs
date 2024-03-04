use std::future::Future;

use crate::{
    dds_async::{
        data_reader::DataReaderAsync, data_reader_listener::DataReaderListenerAsync,
        data_writer::DataWriterAsync, data_writer_listener::DataWriterListenerAsync,
        publisher_listener::PublisherListenerAsync,
    },
    infrastructure::status::{
        LivelinessChangedStatus, LivelinessLostStatus, OfferedDeadlineMissedStatus,
        OfferedIncompatibleQosStatus, PublicationMatchedStatus, RequestedDeadlineMissedStatus,
        RequestedIncompatibleQosStatus, SampleLostStatus, SampleRejectedStatus,
        SubscriptionMatchedStatus,
    },
    publication::{
        data_writer::{AnyDataWriter, DataWriter},
        data_writer_listener::DataWriterListener,
        publisher_listener::PublisherListener,
    },
    subscription::{data_reader::DataReader, data_reader_listener::DataReaderListener},
};

pub(crate) struct ListenerSyncToAsync<T>(T);

impl<T> ListenerSyncToAsync<T> {
    pub fn new(listener: T) -> Self {
        Self(listener)
    }
}

impl<T> PublisherListenerAsync for ListenerSyncToAsync<T>
where
    T: PublisherListener,
{
    fn on_liveliness_lost(
        &mut self,
        the_writer: &dyn AnyDataWriter,
        status: LivelinessLostStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| self.0.on_liveliness_lost(the_writer, status));
        async {}
    }

    fn on_offered_deadline_missed(
        &mut self,
        the_writer: &dyn AnyDataWriter,
        status: OfferedDeadlineMissedStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| self.0.on_offered_deadline_missed(the_writer, status));
        async {}
    }

    fn on_offered_incompatible_qos(
        &mut self,
        the_writer: &dyn AnyDataWriter,
        status: OfferedIncompatibleQosStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| self.0.on_offered_incompatible_qos(the_writer, status));
        async {}
    }

    fn on_publication_matched(
        &mut self,
        the_writer: &dyn AnyDataWriter,
        status: PublicationMatchedStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| self.0.on_publication_matched(the_writer, status));
        async {}
    }
}

impl<T> DataReaderListenerAsync for ListenerSyncToAsync<T>
where
    T: DataReaderListener,
{
    type Foo = <T as DataReaderListener>::Foo;

    fn on_data_available(
        &mut self,
        the_reader: DataReaderAsync<Self::Foo>,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| self.0.on_data_available(DataReader::new(the_reader)));
        async {}
    }

    fn on_sample_rejected(
        &mut self,
        the_reader: DataReaderAsync<Self::Foo>,
        status: SampleRejectedStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| {
            self.0
                .on_sample_rejected(DataReader::new(the_reader), status)
        });
        async {}
    }

    fn on_liveliness_changed(
        &mut self,
        the_reader: DataReaderAsync<Self::Foo>,
        status: LivelinessChangedStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| {
            self.0
                .on_liveliness_changed(DataReader::new(the_reader), status)
        });
        async {}
    }

    fn on_requested_deadline_missed(
        &mut self,
        the_reader: DataReaderAsync<Self::Foo>,
        status: RequestedDeadlineMissedStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| {
            self.0
                .on_requested_deadline_missed(DataReader::new(the_reader), status)
        });
        async {}
    }

    fn on_requested_incompatible_qos(
        &mut self,
        the_reader: DataReaderAsync<Self::Foo>,
        status: RequestedIncompatibleQosStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| {
            self.0
                .on_requested_incompatible_qos(DataReader::new(the_reader), status)
        });
        async {}
    }

    fn on_subscription_matched(
        &mut self,
        the_reader: DataReaderAsync<Self::Foo>,
        status: SubscriptionMatchedStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| {
            self.0
                .on_subscription_matched(DataReader::new(the_reader), status)
        });
        async {}
    }

    fn on_sample_lost(
        &mut self,
        the_reader: DataReaderAsync<Self::Foo>,
        status: SampleLostStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| self.0.on_sample_lost(DataReader::new(the_reader), status));
        async {}
    }
}

impl<T> DataWriterListenerAsync for ListenerSyncToAsync<T>
where
    T: DataWriterListener,
{
    type Foo = T::Foo;

    fn on_liveliness_lost(
        &mut self,
        the_writer: DataWriterAsync<Self::Foo>,
        status: LivelinessLostStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| {
            self.0
                .on_liveliness_lost(DataWriter::new(the_writer), status)
        });
        async {}
    }

    fn on_offered_deadline_missed(
        &mut self,
        the_writer: DataWriterAsync<Self::Foo>,
        status: OfferedDeadlineMissedStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| {
            self.0
                .on_offered_deadline_missed(DataWriter::new(the_writer), status)
        });
        async {}
    }

    fn on_offered_incompatible_qos(
        &mut self,
        the_writer: DataWriterAsync<Self::Foo>,
        status: OfferedIncompatibleQosStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| {
            self.0
                .on_offered_incompatible_qos(DataWriter::new(the_writer), status)
        });
        async {}
    }

    fn on_publication_matched(
        &mut self,
        the_writer: DataWriterAsync<Self::Foo>,
        status: PublicationMatchedStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| {
            self.0
                .on_publication_matched(DataWriter::new(the_writer), status)
        });
        async {}
    }
}
