use core::ops::Deref;
use std::ops::DerefMut;

use crate::behavior::RTPSReader;

pub trait RTPSStatelessReader<T: RTPSReader>: Deref<Target = T> + DerefMut {}
