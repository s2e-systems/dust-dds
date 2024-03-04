use std::marker::PhantomData;

use crate::{
    dds_async::{
        data_reader_listener::DataReaderListenerAsync,
        data_writer_listener::DataWriterListenerAsync, publisher_listener::PublisherListenerAsync,
    },
    domain::domain_participant_listener::DomainParticipantListener,
    publication::{
        data_writer_listener::DataWriterListener, publisher_listener::PublisherListener,
    },
    subscription::{
        data_reader_listener::DataReaderListener, subscriber_listener::SubscriberListener,
    },
    topic_definition::topic_listener::TopicListener,
};

/// This struct implements the listener traits for all entities and can be used as a convinience when the entity
/// does not need the listener operations.
pub struct NoOpListener<Foo = ()>(PhantomData<Foo>);

impl<Foo> NoOpListener<Foo> {
    /// NoOpListener constructor
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<Foo> Default for NoOpListener<Foo> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl DomainParticipantListener for NoOpListener {}
impl PublisherListener for NoOpListener {}
impl SubscriberListener for NoOpListener {}
impl TopicListener for NoOpListener {}

impl<Foo> DataReaderListener for NoOpListener<Foo> {
    type Foo = Foo;
}

impl<Foo> DataWriterListener for NoOpListener<Foo> {
    type Foo = Foo;
}

impl PublisherListenerAsync for NoOpListener {}

impl<Foo> DataReaderListenerAsync for NoOpListener<Foo> {
    type Foo = Foo;
}

impl<Foo> DataWriterListenerAsync for NoOpListener<Foo> {
    type Foo = Foo;
}
