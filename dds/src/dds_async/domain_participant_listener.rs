use std::{future::Future, pin::Pin};

use crate::{
    infrastructure::status::{
        InconsistentTopicStatus, LivelinessChangedStatus, LivelinessLostStatus,
        OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus,
        RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleLostStatus,
        SampleRejectedStatus, SubscriptionMatchedStatus,
    },
    publication::data_writer::AnyDataWriter,
    subscription::data_reader::AnyDataReader,
};

use super::topic::TopicAsync;

/// This trait represents a listener object which can be associated with the [`DomainParticipantAsync`] entity.
pub trait DomainParticipantListenerAsync {
    /// Method that is called when any inconsistent topic is discovered in the domain participant.
    fn on_inconsistent_topic(
        &mut self,
        _the_topic: TopicAsync,
        _status: InconsistentTopicStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(std::future::ready(()))
    }

    /// Method that is called when any writer in the domain participant reports a liveliness lost status.
    fn on_liveliness_lost<'a, 'b>(
        &'a mut self,
        _the_writer: &'b (dyn AnyDataWriter + Sync),
        _status: LivelinessLostStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(std::future::ready(()))
    }

    /// Method that is called when any data writer in the domain participant reports a deadline missed status.
    fn on_offered_deadline_missed<'a, 'b>(
        &'a mut self,
        _the_writer: &'b (dyn AnyDataWriter + Sync),
        _status: OfferedDeadlineMissedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(std::future::ready(()))
    }

    /// Method that is called when any data writer in the domain participant reports an offered incompatible QoS status.
    fn on_offered_incompatible_qos<'a, 'b>(
        &'a mut self,
        _the_writer: &'b (dyn AnyDataWriter + Sync),
        _status: OfferedIncompatibleQosStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(std::future::ready(()))
    }

    /// Method that is called when any data reader in the domain participant reports a sample lost status.
    fn on_sample_lost<'a, 'b>(
        &'a mut self,
        _the_reader: &'b (dyn AnyDataReader + Sync),
        _status: SampleLostStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(std::future::ready(()))
    }

    /// Method that is called when any data reader in the domain participant reports a data available status.
    fn on_data_available<'a, 'b>(
        &'a mut self,
        _the_reader: &'b (dyn AnyDataReader + Sync),
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(std::future::ready(()))
    }

    /// Method that is called when any data reader in the domain participant reports a sample rejected status.
    fn on_sample_rejected<'a, 'b>(
        &'a mut self,
        _the_reader: &'b (dyn AnyDataReader + Sync),
        _status: SampleRejectedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(std::future::ready(()))
    }

    /// Method that is called when any data reader in the domain participant reports a liveliness changed status.
    fn on_liveliness_changed<'a, 'b>(
        &'a mut self,
        _the_reader: &'b (dyn AnyDataReader + Sync),
        _status: LivelinessChangedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(std::future::ready(()))
    }

    /// Method that is called when any data reader in the domain participant reports a requested deadline missed status.
    fn on_requested_deadline_missed<'a, 'b>(
        &'a mut self,
        _the_reader: &'b (dyn AnyDataReader + Sync),
        _status: RequestedDeadlineMissedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(std::future::ready(()))
    }

    /// Method that is called when any data reader in the domain participant reports a requested incompatible QoS status.
    fn on_requested_incompatible_qos<'a, 'b>(
        &'a mut self,
        _the_reader: &'b (dyn AnyDataReader + Sync),
        _status: RequestedIncompatibleQosStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(std::future::ready(()))
    }

    /// Method that is called when any data writer in the domain participant reports a publication matched status.
    fn on_publication_matched<'a, 'b>(
        &'a mut self,
        _the_writer: &'b (dyn AnyDataWriter + Sync),
        _status: PublicationMatchedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(std::future::ready(()))
    }

    /// Method that is called when any data reader in the domain participant reports a subscription matched status.
    fn on_subscription_matched<'a, 'b>(
        &'a mut self,
        _the_reader: &'b (dyn AnyDataReader + Sync),
        _status: SubscriptionMatchedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(std::future::ready(()))
    }
}
