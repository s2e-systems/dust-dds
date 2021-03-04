use std::ops::{Deref, DerefMut};

use crate::behavior::Reader;

pub trait StatelessReader<T: Reader>: Deref<Target = T> + DerefMut {}
