use crate::discovery::types::DomainId;

pub trait SedpParticipant {
    type BuiltinPublicationsWriter;
    type BuiltinPublicationsReader;
    type BuiltinSubscriptionsWriter;
    type BuiltinSubscriptionsReader;
    type BuiltinTopicsWriter;
    type BuiltinTopicsReader;

    fn domain_id(&self) -> DomainId;
    fn domain_tag(&self) -> &str;

    fn sedp_builtin_publications_writer(&mut self) -> Option<&mut Self::BuiltinPublicationsWriter>;
    fn sedp_builtin_publications_reader(&mut self) -> Option<&mut Self::BuiltinPublicationsReader>;
    fn sedp_builtin_subscriptions_writer(
        &mut self,
    ) -> Option<&mut Self::BuiltinSubscriptionsWriter>;
    fn sedp_builtin_subscriptions_reader(
        &mut self,
    ) -> Option<&mut Self::BuiltinSubscriptionsReader>;
    fn sedp_builtin_topics_writer(&mut self) -> Option<&mut Self::BuiltinTopicsWriter>;
    fn sedp_builtin_topics_reader(&mut self) -> Option<&mut Self::BuiltinTopicsReader>;
}
