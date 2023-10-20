use std::marker::PhantomData;

use crate::{
    domain::domain_participant_listener::DomainParticipantListener,
    publication::{
        data_writer_listener::DataWriterListener, publisher_listener::PublisherListener,
    },
    subscription::{
        data_reader_listener::DataReaderListener, subscriber_listener::SubscriberListener,
    },
    topic_definition::topic_listener::TopicListener,
};

pub struct NoOpListener<Foo = ()>(PhantomData<Foo>);

impl<Foo> NoOpListener<Foo> {
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
