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

#[derive(Default)]
pub struct NoOpListener();

impl NoOpListener {
    pub fn new() -> Self {
        Self()
    }
}

impl DomainParticipantListener for NoOpListener {}
impl PublisherListener for NoOpListener {}
impl SubscriberListener for NoOpListener {}
impl TopicListener for NoOpListener {}

pub struct NoOpFooListener<Foo>(PhantomData<Foo>);

impl<Foo> NoOpFooListener<Foo> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<Foo> Default for NoOpFooListener<Foo> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<Foo> DataReaderListener for NoOpFooListener<Foo> {
    type Foo = Foo;
}
impl<Foo> DataWriterListener for NoOpFooListener<Foo> {
    type Foo = Foo;
}
