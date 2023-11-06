use crate::{
    implementation::payload_serializer_deserializer::{
        cdr_serializer::ClassicCdrSerializer, endianness::CdrEndianness,
    },
    infrastructure::{error::DdsResult, instance::InstanceHandle},
    serialized_payload::cdr::serialize::CdrSerialize,
};

pub fn get_instance_handle_from_key(foo_key: &impl CdrSerialize) -> DdsResult<InstanceHandle> {
    let mut serialized_key = Vec::new();
    let mut serializer = ClassicCdrSerializer::new(&mut serialized_key, CdrEndianness::BigEndian);
    CdrSerialize::serialize(foo_key, &mut serializer)?;
    let handle = if serialized_key.len() <= 16 {
        let mut h = [0; 16];
        h[..serialized_key.len()].clone_from_slice(serialized_key.as_slice());
        h
    } else {
        <[u8; 16]>::from(md5::compute(serialized_key.as_slice()))
    };
    Ok(InstanceHandle::new(handle))
}
