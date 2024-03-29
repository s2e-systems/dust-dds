use std::{future::Future, pin::Pin};

use crate::{
    dds_async::{
        data_reader::DataReaderAsync, data_reader_listener::DataReaderListenerAsync,
        data_writer::DataWriterAsync, data_writer_listener::DataWriterListenerAsync,
        domain_participant_listener::DomainParticipantListenerAsync,
        publisher_listener::PublisherListenerAsync, subscriber::SubscriberAsync,
        subscriber_listener::SubscriberListenerAsync, topic::TopicAsync,
        topic_listener::TopicListenerAsync,
    },
    domain::domain_participant_listener::DomainParticipantListener,
    infrastructure::status::{
        InconsistentTopicStatus, LivelinessChangedStatus, LivelinessLostStatus,
        OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus,
        RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleLostStatus,
        SampleRejectedStatus, SubscriptionMatchedStatus,
    },
    publication::{
        data_writer::{AnyDataWriter, DataWriter},
        data_writer_listener::DataWriterListener,
        publisher_listener::PublisherListener,
    },
    subscription::{
        data_reader::{AnyDataReader, DataReader},
        data_reader_listener::DataReaderListener,
        subscriber::Subscriber,
        subscriber_listener::SubscriberListener,
    },
    topic_definition::{topic::Topic, topic_listener::TopicListener},
};

pub(crate) struct ListenerSyncToAsync<T>(T);

impl<T> ListenerSyncToAsync<T> {
    pub fn new(listener: T) -> Self {
        Self(listener)
    }
}

impl<T> DomainParticipantListenerAsync for ListenerSyncToAsync<T>
where
    T: DomainParticipantListener,
{
    fn on_inconsistent_topic(
        &mut self,
        the_topic: TopicAsync,
        status: InconsistentTopicStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| self.0.on_inconsistent_topic(Topic::new(the_topic), status));
        std::future::ready(())
    }

    fn on_liveliness_lost(
        &mut self,
        the_writer: &dyn AnyDataWriter,
        status: LivelinessLostStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| self.0.on_liveliness_lost(the_writer, status));
        std::future::ready(())
    }

    fn on_offered_deadline_missed(
        &mut self,
        the_writer: &dyn AnyDataWriter,
        status: OfferedDeadlineMissedStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| self.0.on_offered_deadline_missed(the_writer, status));
        std::future::ready(())
    }

    fn on_offered_incompatible_qos(
        &mut self,
        the_writer: &dyn AnyDataWriter,
        status: OfferedIncompatibleQosStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| self.0.on_offered_incompatible_qos(the_writer, status));
        std::future::ready(())
    }

    fn on_sample_lost(
        &mut self,
        the_reader: &dyn AnyDataReader,
        status: SampleLostStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| self.0.on_sample_lost(the_reader, status));
        std::future::ready(())
    }

    fn on_data_available(
        &mut self,
        the_reader: &dyn AnyDataReader,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| self.0.on_data_available(the_reader));
        std::future::ready(())
    }

    fn on_sample_rejected(
        &mut self,
        the_reader: &dyn AnyDataReader,
        status: SampleRejectedStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| self.0.on_sample_rejected(the_reader, status));
        std::future::ready(())
    }

    fn on_liveliness_changed(
        &mut self,
        the_reader: &dyn AnyDataReader,
        status: LivelinessChangedStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| self.0.on_liveliness_changed(the_reader, status));
        std::future::ready(())
    }

    fn on_requested_deadline_missed(
        &mut self,
        the_reader: &dyn AnyDataReader,
        status: RequestedDeadlineMissedStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| self.0.on_requested_deadline_missed(the_reader, status));
        std::future::ready(())
    }

    fn on_requested_incompatible_qos(
        &mut self,
        the_reader: &dyn AnyDataReader,
        status: RequestedIncompatibleQosStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| self.0.on_requested_incompatible_qos(the_reader, status));
        std::future::ready(())
    }

    fn on_publication_matched(
        &mut self,
        the_writer: &dyn AnyDataWriter,
        status: PublicationMatchedStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| self.0.on_publication_matched(the_writer, status));
        std::future::ready(())
    }

    fn on_subscription_matched(
        &mut self,
        the_reader: &dyn AnyDataReader,
        status: SubscriptionMatchedStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| self.0.on_subscription_matched(the_reader, status));
        std::future::ready(())
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
        std::future::ready(())
    }

    fn on_offered_deadline_missed(
        &mut self,
        the_writer: &dyn AnyDataWriter,
        status: OfferedDeadlineMissedStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| self.0.on_offered_deadline_missed(the_writer, status));
        std::future::ready(())
    }

    fn on_offered_incompatible_qos(
        &mut self,
        the_writer: &dyn AnyDataWriter,
        status: OfferedIncompatibleQosStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| self.0.on_offered_incompatible_qos(the_writer, status));
        std::future::ready(())
    }

    fn on_publication_matched(
        &mut self,
        the_writer: &dyn AnyDataWriter,
        status: PublicationMatchedStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| self.0.on_publication_matched(the_writer, status));
        std::future::ready(())
    }
}

impl<T> SubscriberListenerAsync for ListenerSyncToAsync<T>
where
    T: SubscriberListener,
{
    fn on_data_on_readers(
        &mut self,
        the_subscriber: SubscriberAsync,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| self.0.on_data_on_readers(Subscriber::new(the_subscriber)));
        std::future::ready(())
    }

    fn on_data_available(
        &mut self,
        the_reader: &dyn AnyDataReader,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| self.0.on_data_available(the_reader));
        std::future::ready(())
    }

    fn on_sample_rejected(
        &mut self,
        the_reader: &dyn AnyDataReader,
        status: SampleRejectedStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| self.0.on_sample_rejected(the_reader, status));
        std::future::ready(())
    }

    fn on_liveliness_changed(
        &mut self,
        the_reader: &dyn AnyDataReader,
        status: LivelinessChangedStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| self.0.on_liveliness_changed(the_reader, status));
        std::future::ready(())
    }

    fn on_requested_deadline_missed(
        &mut self,
        the_reader: &dyn AnyDataReader,
        status: RequestedDeadlineMissedStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| self.0.on_requested_deadline_missed(the_reader, status));
        std::future::ready(())
    }

    fn on_requested_incompatible_qos(
        &mut self,
        the_reader: &dyn AnyDataReader,
        status: RequestedIncompatibleQosStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| self.0.on_requested_incompatible_qos(the_reader, status));
        std::future::ready(())
    }

    fn on_subscription_matched(
        &mut self,
        the_reader: &dyn AnyDataReader,
        status: SubscriptionMatchedStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| self.0.on_subscription_matched(the_reader, status));
        std::future::ready(())
    }

    fn on_sample_lost(
        &mut self,
        the_reader: &dyn AnyDataReader,
        status: SampleLostStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| self.0.on_sample_lost(the_reader, status));
        std::future::ready(())
    }
}

impl<T> TopicListenerAsync for ListenerSyncToAsync<T>
where
    T: TopicListener,
{
    fn on_inconsistent_topic(
        &mut self,
        the_topic: TopicAsync,
        status: InconsistentTopicStatus,
    ) -> impl Future<Output = ()> + Send {
        tokio::task::block_in_place(|| self.0.on_inconsistent_topic(Topic::new(the_topic), status));
        std::future::ready(())
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
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        tokio::task::block_in_place(|| self.0.on_data_available(DataReader::new(the_reader)));
        Box::pin(std::future::ready(()))
    }

    fn on_sample_rejected(
        &mut self,
        the_reader: DataReaderAsync<Self::Foo>,
        status: SampleRejectedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        tokio::task::block_in_place(|| {
            self.0
                .on_sample_rejected(DataReader::new(the_reader), status)
        });
        Box::pin(std::future::ready(()))
    }

    fn on_liveliness_changed(
        &mut self,
        the_reader: DataReaderAsync<Self::Foo>,
        status: LivelinessChangedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        tokio::task::block_in_place(|| {
            self.0
                .on_liveliness_changed(DataReader::new(the_reader), status)
        });
        Box::pin(std::future::ready(()))
    }

    fn on_requested_deadline_missed(
        &mut self,
        the_reader: DataReaderAsync<Self::Foo>,
        status: RequestedDeadlineMissedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        tokio::task::block_in_place(|| {
            self.0
                .on_requested_deadline_missed(DataReader::new(the_reader), status)
        });
        Box::pin(std::future::ready(()))
    }

    fn on_requested_incompatible_qos(
        &mut self,
        the_reader: DataReaderAsync<Self::Foo>,
        status: RequestedIncompatibleQosStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        tokio::task::block_in_place(|| {
            self.0
                .on_requested_incompatible_qos(DataReader::new(the_reader), status)
        });
        Box::pin(std::future::ready(()))
    }

    fn on_subscription_matched(
        &mut self,
        the_reader: DataReaderAsync<Self::Foo>,
        status: SubscriptionMatchedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        tokio::task::block_in_place(|| {
            self.0
                .on_subscription_matched(DataReader::new(the_reader), status)
        });
        Box::pin(std::future::ready(()))
    }

    fn on_sample_lost(
        &mut self,
        the_reader: DataReaderAsync<Self::Foo>,
        status: SampleLostStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        tokio::task::block_in_place(|| self.0.on_sample_lost(DataReader::new(the_reader), status));
        Box::pin(std::future::ready(()))
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
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        tokio::task::block_in_place(|| {
            self.0
                .on_liveliness_lost(DataWriter::new(the_writer), status)
        });
        Box::pin(std::future::ready(()))
    }

    fn on_offered_deadline_missed(
        &mut self,
        the_writer: DataWriterAsync<Self::Foo>,
        status: OfferedDeadlineMissedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        tokio::task::block_in_place(|| {
            self.0
                .on_offered_deadline_missed(DataWriter::new(the_writer), status)
        });
        Box::pin(std::future::ready(()))
    }

    fn on_offered_incompatible_qos(
        &mut self,
        the_writer: DataWriterAsync<Self::Foo>,
        status: OfferedIncompatibleQosStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        tokio::task::block_in_place(|| {
            self.0
                .on_offered_incompatible_qos(DataWriter::new(the_writer), status)
        });
        Box::pin(std::future::ready(()))
    }

    fn on_publication_matched(
        &mut self,
        the_writer: DataWriterAsync<Self::Foo>,
        status: PublicationMatchedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        tokio::task::block_in_place(|| {
            self.0
                .on_publication_matched(DataWriter::new(the_writer), status)
        });
        Box::pin(std::future::ready(()))
    }
}
