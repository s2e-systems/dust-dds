use crate::{
    dcps::runtime::DdsRuntime,
    domain::domain_participant_listener::DomainParticipantListener,
    publication::{
        data_writer_listener::DataWriterListener, publisher_listener::PublisherListener,
    },
    subscription::{
        data_reader_listener::DataReaderListener, subscriber_listener::SubscriberListener,
    },
    topic_definition::topic_listener::TopicListener,
};

/// Convinience constant to be used when the user does not want to install a listener.
pub const NO_LISTENER: Option<()> = None;

impl<R: DdsRuntime> DomainParticipantListener<R> for () {}

impl<R: DdsRuntime> PublisherListener<R> for () {}

impl<R: DdsRuntime> SubscriberListener<R> for () {}

impl<R: DdsRuntime> TopicListener<R> for () {}

impl<R: DdsRuntime, Foo> DataWriterListener<R, Foo> for () {}

impl<R: DdsRuntime, Foo> DataReaderListener<R, Foo> for () {}
