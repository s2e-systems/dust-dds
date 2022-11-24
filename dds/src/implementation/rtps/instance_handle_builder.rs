use crate::{
    infrastructure::{error::DdsResult, instance::InstanceHandle},
    topic_definition::type_support::{DdsDeserialize, DdsType},
};

pub struct InstanceHandleBuilder(fn(&[u8]) -> DdsResult<Vec<u8>>);

impl InstanceHandleBuilder {
    pub fn new<Foo>() -> Self
    where
        Foo: for<'de> DdsDeserialize<'de> + DdsType,
    {
        Self(Foo::deserialize_key)
    }

    pub fn build_instance_handle(&self, data: &[u8]) -> DdsResult<InstanceHandle> {
        Ok((self.0)(data)?.as_slice().into())
    }
}
