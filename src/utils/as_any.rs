use std::any::Any;

pub trait AsAny{
    fn as_any(&self) -> &dyn Any;
}