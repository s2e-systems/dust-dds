use std::ops::{Deref, DerefMut};

use rust_rtps::behavior::StatefulWriter;

use super::rtps_datawriter_inner::RtpsDataWriterInner;

pub struct RtpsStatefulDataWriterInner {
    pub stateful_writer: StatefulWriter,
    pub inner: RtpsDataWriterInner,
}

impl Deref for RtpsStatefulDataWriterInner {
    type Target = StatefulWriter;

    fn deref(&self) -> &Self::Target {
        &self.stateful_writer
    }
}

impl DerefMut for RtpsStatefulDataWriterInner {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stateful_writer
    }
}