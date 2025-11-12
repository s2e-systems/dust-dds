use tracing::info;

use crate::{
    dds_async::{data_reader::DataReaderAsync, subscriber::SubscriberAsync},
    infrastructure::status::{
        LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
        SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
    },
    runtime::DdsRuntime,
};
use core::future::Future;

/// This trait represents a listener object which can be associated with the [`Subscriber`](super::subscriber::Subscriber) entity.
pub trait SubscriberListener<R: DdsRuntime> {
    /// Method that is called when any reader belonging to this subcriber reports new data available. This method is triggered before the on_data_available method.
    fn on_data_on_readers(
        &mut self,
        _the_subscriber: SubscriberAsync<R>,
    ) -> impl Future<Output = ()> + Send {
        info!("on_data_on_readers");
        core::future::ready(())
    }

    /// Method that is called when any reader belonging to this subcriber reports new data available.
    fn on_data_available(
        &mut self,
        _the_reader: DataReaderAsync<R, ()>,
    ) -> impl Future<Output = ()> + Send {
        info!("on_data_available");
        core::future::ready(())
    }

    /// Method that is called when any reader belonging to this subcriber reports a sample rejected status.
    fn on_sample_rejected(
        &mut self,
        _the_reader: DataReaderAsync<R, ()>,
        status: SampleRejectedStatus,
    ) -> impl Future<Output = ()> + Send {
        info!(?status, "on_sample_rejected");
        core::future::ready(())
    }

    /// Method that is called when any reader belonging to this subcriber reports a liveliness changed status.
    fn on_liveliness_changed(
        &mut self,
        _the_reader: DataReaderAsync<R, ()>,
        status: LivelinessChangedStatus,
    ) -> impl Future<Output = ()> + Send {
        info!(?status, "on_liveliness_changed");
        core::future::ready(())
    }

    /// Method that is called when any reader belonging to this subcriber reports a requested deadline missed status.
    fn on_requested_deadline_missed(
        &mut self,
        _the_reader: DataReaderAsync<R, ()>,
        status: RequestedDeadlineMissedStatus,
    ) -> impl Future<Output = ()> + Send {
        info!(?status, "on_requested_deadline_missed");
        core::future::ready(())
    }

    /// Method that is called when any reader belonging to this subcriber reports a requested incompatible QoS status.
    fn on_requested_incompatible_qos(
        &mut self,
        _the_reader: DataReaderAsync<R, ()>,
        status: RequestedIncompatibleQosStatus,
    ) -> impl Future<Output = ()> + Send {
        info!(?status, "on_requested_incompatible_qos");
        core::future::ready(())
    }

    /// Method that is called when any reader belonging to this subcriber reports a subscription matched status.
    fn on_subscription_matched(
        &mut self,
        _the_reader: DataReaderAsync<R, ()>,
        status: SubscriptionMatchedStatus,
    ) -> impl Future<Output = ()> + Send {
        info!(?status, "on_subscription_matched");
        core::future::ready(())
    }

    /// Method that is called when any reader belonging to this subcriber reports a sample lost status.
    fn on_sample_lost(
        &mut self,
        _the_reader: DataReaderAsync<R, ()>,
        status: SampleLostStatus,
    ) -> impl Future<Output = ()> + Send {
        info!(?status, "on_sample_lost");
        core::future::ready(())
    }
}
