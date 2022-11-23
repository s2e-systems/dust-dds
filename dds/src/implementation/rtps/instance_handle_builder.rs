use crate::{
    infrastructure::{error::DdsResult, instance::InstanceHandle},
    topic_definition::type_support::{DdsDeserialize, DdsType, LittleEndian},
};

pub struct InstanceHandleBuilder(fn(&[u8]) -> DdsResult<Vec<u8>>);

impl InstanceHandleBuilder {
    pub fn new<Foo>() -> Self
    where
        Foo: for<'de> DdsDeserialize<'de> + DdsType,
    {
        // Create a function that deserializes the data and gets the key for the type
        // without having to store the actual type intermediatelly to avoid generics
        fn serialized_data_to_key_func<Foo>(mut buf: &[u8]) -> DdsResult<Vec<u8>>
        where
            Foo: for<'de> DdsDeserialize<'de> + DdsType,
        {
            Ok(Foo::deserialize(&mut buf)?.get_serialized_key::<LittleEndian>())
        }

        Self(serialized_data_to_key_func::<Foo>)
    }

    pub fn create_instance_handle(&self, mut data: &[u8]) -> DdsResult<InstanceHandle> {
        fn calculate_instance_handle(serialized_key: &[u8]) -> InstanceHandle {
            if serialized_key.len() <= 16 {
                let mut h = [0; 16];
                h[..serialized_key.len()].clone_from_slice(serialized_key);
                h.into()
            } else {
                <[u8; 16]>::from(md5::compute(serialized_key)).into()
            }
        }

        Ok(calculate_instance_handle(&(self.0)(&mut data)?))
    }
}
