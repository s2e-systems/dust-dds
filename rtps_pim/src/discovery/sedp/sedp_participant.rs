pub trait SedpParticipant {
    type BuiltinPublicationsWriter;

    fn sedp_builtin_publications_writer(&mut self) -> &mut Self::BuiltinPublicationsWriter;
}