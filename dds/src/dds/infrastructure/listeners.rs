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

pub struct NoListener<Foo>(PhantomData<Foo>);

impl<Foo> NoListener<Foo> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<Foo> DomainParticipantListener for NoListener<Foo> {}
impl<Foo> PublisherListener for NoListener<Foo> {}
impl<Foo> SubscriberListener for NoListener<Foo> {}
impl<Foo> TopicListener for NoListener<Foo> {}
impl<Foo> DataReaderListener<Foo> for NoListener<Foo> {}
impl<Foo> DataWriterListener for NoListener<Foo> {
    type Foo = Foo;
}
