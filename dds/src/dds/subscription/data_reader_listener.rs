use crate::{
    dcps::runtime::DdsRuntime,
    dds_async::data_reader::DataReaderAsync,
    infrastructure::status::{
        LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
        SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
    },
};
use core::future::Future;

/// This trait represents a listener object which can be associated with the [`DataReader`] entity.
pub trait DataReaderListener<R: DdsRuntime, Foo>: 'static {
    /// Method that is called when new data is received by the reader.
    fn on_data_available(
        &mut self,
        _the_reader: DataReaderAsync<R, Foo>,
    ) -> impl Future<Output = ()> + Send {
        core::future::ready(())
    }

    /// Method that is called when this reader reports a sample rejected status.
    fn on_sample_rejected(
        &mut self,
        _the_reader: DataReaderAsync<R, Foo>,
        _status: SampleRejectedStatus,
    ) -> impl Future<Output = ()> + Send {
        core::future::ready(())
    }
    /// Method that is called when this reader reports a liveliness changed status.
    fn on_liveliness_changed(
        &mut self,
        _the_reader: DataReaderAsync<R, Foo>,
        _status: LivelinessChangedStatus,
    ) -> impl Future<Output = ()> + Send {
        core::future::ready(())
    }

    /// Method that is called when this reader reports a requested deadline missed status.
    fn on_requested_deadline_missed(
        &mut self,
        _the_reader: DataReaderAsync<R, Foo>,
        _status: RequestedDeadlineMissedStatus,
    ) -> impl Future<Output = ()> + Send {
        core::future::ready(())
    }

    /// Method that is called when this reader reports a requested incompatible QoS status.
    fn on_requested_incompatible_qos(
        &mut self,
        _the_reader: DataReaderAsync<R, Foo>,
        _status: RequestedIncompatibleQosStatus,
    ) -> impl Future<Output = ()> + Send {
        core::future::ready(())
    }

    /// Method that is called when this reader reports a subscription matched status.
    fn on_subscription_matched(
        &mut self,
        _the_reader: DataReaderAsync<R, Foo>,
        _status: SubscriptionMatchedStatus,
    ) -> impl Future<Output = ()> + Send {
        core::future::ready(())
    }

    /// Method that is called when this reader reports a sample lost status.
    fn on_sample_lost(
        &mut self,
        _the_reader: DataReaderAsync<R, Foo>,
        _status: SampleLostStatus,
    ) -> impl Future<Output = ()> + Send {
        core::future::ready(())
    }
}
