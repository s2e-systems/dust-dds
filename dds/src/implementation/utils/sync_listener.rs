use std::future::Future;

use crate::{
    dds_async::{data_reader::DataReaderAsync, data_reader_listener::DataReaderListenerAsync},
    infrastructure::status::{
        LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
        SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
    },
    subscription::{data_reader::DataReader, data_reader_listener::DataReaderListener},
};

pub(crate) struct ListenerSyncToAsync<T>(T);

impl<T> ListenerSyncToAsync<T> {
    pub fn new(listener: T) -> Self {
        Self(listener)
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
