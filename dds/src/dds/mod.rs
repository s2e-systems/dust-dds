/// Contains the built-in topics used by the service to propagate information needed for discovery and other data.
pub mod builtin_topics;

/// Contains the [`DomainParticipantFactory`](crate::domain::domain_participant_factory::DomainParticipantFactory) which is responsible for creating the
/// [`DomainParticipant`](crate::domain::domain_participant::DomainParticipant). The [`DomainParticipant`](crate::domain::domain_participant::DomainParticipant)
/// acts as an entry-point of the Service and a factory and contained for many of the classes that make up the Service.
pub mod domain;

/// Contains all the basic types used in the other modules including e.g. qos policies and communication statuses.
pub mod infrastructure;

/// Contains the [`Publisher`](crate::publication::publisher::Publisher) and [`DataWriter`](crate::publication::data_writer::DataWriter) classes as well as its
/// listener traits, and more generally, all that is needed on the publication side.
pub mod publication;

/// Contains the [`Subscriber`](crate::subscription::subscriber::Subscriber) and [`DataReader`](crate::subscription::data_reader::DataReader) classes as well as
/// its listener traits, and more generally, all that is needed on the subscription side.
pub mod subscription;

/// Contains the [`Topic`](crate::topic_definition::topic::Topic) class as well as its listener trait, and more generally, all that is needed
/// by the application to define topics and attach qos policies.
pub mod topic_definition;
