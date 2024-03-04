use std::future::Future;

use crate::{
    dds_async::data_reader_listener::DataReaderListenerAsync,
    infrastructure::status::{
        LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
        SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
    },
};

use super::data_reader::DataReader;

/// This trait represents a listener object which can be associated with the [`DataReader`] entity.
pub trait DataReaderListener {
    /// Type of the DataReader with which this Listener will be associated.
    type Foo;

    /// Method that is called when new data is received by the reader.
    fn on_data_available(&mut self, _the_reader: DataReader<Self::Foo>) {}

    /// Method that is called when this reader reports a sample rejected status.
    fn on_sample_rejected(
        &mut self,
        _the_reader: DataReader<Self::Foo>,
        _status: SampleRejectedStatus,
    ) {
    }
    /// Method that is called when this reader reports a liveliness changed status.
    fn on_liveliness_changed(
        &mut self,
        _the_reader: DataReader<Self::Foo>,
        _status: LivelinessChangedStatus,
    ) {
    }

    /// Method that is called when this reader reports a requested deadline missed status.
    fn on_requested_deadline_missed(
        &mut self,
        _the_reader: DataReader<Self::Foo>,
        _status: RequestedDeadlineMissedStatus,
    ) {
    }

    /// Method that is called when this reader reports a requested incompatible QoS status.
    fn on_requested_incompatible_qos(
        &mut self,
        _the_reader: DataReader<Self::Foo>,
        _status: RequestedIncompatibleQosStatus,
    ) {
    }

    /// Method that is called when this reader reports a subscription matched status.
    fn on_subscription_matched(
        &mut self,
        _the_reader: DataReader<Self::Foo>,
        _status: SubscriptionMatchedStatus,
    ) {
    }

    /// Method that is called when this reader reports a sample lost status.
    fn on_sample_lost(&mut self, _the_reader: DataReader<Self::Foo>, _status: SampleLostStatus) {}
}

pub(crate) struct DataReaderListenerSync<T>(T);

impl<T> DataReaderListenerSync<T> {
    pub fn new(listener: T) -> Self {
        Self(listener)
    }
}

impl<T> DataReaderListenerAsync for DataReaderListenerSync<T>
where
    T: DataReaderListener,
{
    type Foo = <T as DataReaderListener>::Foo;

    fn on_data_available(
        &mut self,
        the_reader: crate::dds_async::data_reader::DataReaderAsync<Self::Foo>,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| self.0.on_data_available(DataReader::new(the_reader)));
        async {}
    }

    fn on_sample_rejected(
        &mut self,
        the_reader: crate::dds_async::data_reader::DataReaderAsync<Self::Foo>,
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
        the_reader: crate::dds_async::data_reader::DataReaderAsync<Self::Foo>,
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
        the_reader: crate::dds_async::data_reader::DataReaderAsync<Self::Foo>,
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
        the_reader: crate::dds_async::data_reader::DataReaderAsync<Self::Foo>,
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
        the_reader: crate::dds_async::data_reader::DataReaderAsync<Self::Foo>,
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
        the_reader: crate::dds_async::data_reader::DataReaderAsync<Self::Foo>,
        status: SampleLostStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| self.0.on_sample_lost(DataReader::new(the_reader), status));
        async {}
    }
}
