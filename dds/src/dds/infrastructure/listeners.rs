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

pub struct NoListener();

impl NoListener {
    pub fn new() -> Self {
        Self()
    }
}

impl DomainParticipantListener for NoListener {}
impl PublisherListener for NoListener {}
impl SubscriberListener for NoListener {}
impl TopicListener for NoListener {}

pub struct NoFooListener<Foo>(PhantomData<Foo>);

impl<Foo> NoFooListener<Foo> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<Foo> DataReaderListener for NoFooListener<Foo> {
    type Foo = Foo;
}
impl<Foo> DataWriterListener for NoFooListener<Foo> {
    type Foo = Foo;
}
