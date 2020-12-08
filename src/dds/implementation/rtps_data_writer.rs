use crate::types::DDSType;
pub struct RtpsDataWriter<T: DDSType> {
    phantom: std::marker::PhantomData<T>
}