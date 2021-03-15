use core::ops::Deref;

use crate::behavior::RTPSReader;

pub trait RTPSStatelessReader<T: RTPSReader>: Deref<Target = T> {}
