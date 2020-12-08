use crate::types::DDSType;

pub struct RtpsTopic<T: DDSType>{
    marker: std::marker::PhantomData<T>
}
