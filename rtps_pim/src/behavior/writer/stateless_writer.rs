use crate::{
    behavior::types::Duration,
    structure::types::{Guid, Locator, ReliabilityKind, TopicKind},
};

use super::writer::RtpsWriter;

pub struct RtpsStatelessWriter<L, C, R> {
    pub writer: RtpsWriter<L, C>,
    pub reader_locators: R,
}

pub trait RtpsStatelessWriterOperations {
    fn reader_locator_add(&mut self, a_locator: Locator);

    fn reader_locator_remove(&mut self, a_locator: &Locator);

    fn unsent_changes_reset(&mut self);
}
