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

pub struct NoOpFooListener<Foo = ()>(PhantomData<Foo>);

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

impl DomainParticipantListener for NoOpFooListener {}
impl PublisherListener for NoOpFooListener {}
impl SubscriberListener for NoOpFooListener {}
impl TopicListener for NoOpFooListener {}

impl<Foo> DataReaderListener for NoOpFooListener<Foo> {
    type Foo = Foo;
}

impl<Foo> DataWriterListener for NoOpFooListener<Foo> {
    type Foo = Foo;
}

impl<Foo> DataWriterListener for fn(&Foo) {
    type Foo = Foo;
}
