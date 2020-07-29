use std::fmt;
use std::marker::PhantomData;

use serde::ser::{Serialize,Serializer, SerializeStruct};
use serde::de::{Deserialize, Deserializer, Visitor, Error, MapAccess, SeqAccess};

use cdr::{deserialize_from, Infinite, BigEndian, LittleEndian};

use crate::messages::types::ParameterId;
use crate::types::{VendorId, Locator};

type DomainId = u32;

#[derive(Debug)]
struct CdrParameter<'a, T: Serialize> {
    parameter_id: ParameterId,
    value: &'a T,
    default: Option<T>,
}

impl<'a, T> CdrParameter<'a, T>
where T: Serialize + Deserialize<'a> {
    fn new(parameter_id: ParameterId, value: &'a T, default: Option<T>) -> Self {
        Self {
            parameter_id,
            value,
            default,
        }
    }
}

impl<'a, T> Serialize for CdrParameter<'a, T> 
    where T: Serialize + PartialEq + std::fmt::Debug{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {

        // Only serialize the value if it is different than the default (in case it exists), otherwise just do an empty
        // serialization (means serialize () type)
        if self.default.is_none() || self.value != self.default.as_ref().unwrap() {

            let mut state = serializer.serialize_struct("", 3)?;

            let serialized_data_size = cdr::size::calc_serialized_data_size(&self.value) as i16;
            // rounded up to multple of 4 (that is besides the length of the value may not be a multiple of 4)
            let length = (serialized_data_size + 3 & !3) as i16;
            let padding = length - serialized_data_size;

            assert!(padding >= 0);

            
            state.serialize_field("", &self.parameter_id)?;
            state.serialize_field("", &length)?;
            state.serialize_field("", self.value)?;
            for _ in 0..padding {
                state.serialize_field("", &(0 as u8))?;
            }

            state.end()
        } else {
            serializer.serialize_unit()
        }
    }
}

#[derive(Debug)]
struct GenericCdrParameter {
    parameter_id: ParameterId,
    value: Vec<u8>,
}

impl<'de> GenericCdrParameter {
    fn parameter_id(&self) -> ParameterId {
        self.parameter_id
    }

    fn value_as<T: Deserialize<'de>, E: ByteOrder>(&self) -> T {
        let mut deserializer = cdr::Deserializer::<_, Infinite, BigEndian>::new(self.value.as_slice(), Infinite);
        serde::Deserialize::deserialize(&mut deserializer).unwrap()
    }
}

impl<'a> Deserialize<'a> for GenericCdrParameter 
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
            D: Deserializer<'a> {

        struct GenericCdrParameterVisitor;

        impl<'a> Visitor<'a> for GenericCdrParameterVisitor {
            type Value = GenericCdrParameter;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct CdrParameter")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'a>,
            {
                let parameter_id : ParameterId = seq.next_element()?.unwrap();
                let length : i16 = seq.next_element()?.unwrap();

                let value = match length {
                    0 => [].to_vec(),
                    4 => seq.next_element::<[u8;4]>()?.unwrap().to_vec(),
                    8 => seq.next_element::<[u8;8]>()?.unwrap().to_vec(),
                    12 => seq.next_element::<[u8;12]>()?.unwrap().to_vec(),
                    16 => seq.next_element::<[u8;16]>()?.unwrap().to_vec(),
                    _ => panic!("Invalid length"),
                };

                Ok(GenericCdrParameter{parameter_id, value})
            }

        }
                
        const FIELDS: &'static [&'static str] = &["", "", ""];
        deserializer.deserialize_struct("", FIELDS, GenericCdrParameterVisitor)
    }
}

#[derive(Debug)]
pub struct SpdpParticipantData{
    domain_id: DomainId,
    domain_tag: String,
    vendor_id: VendorId,
}



impl Serialize for SpdpParticipantData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        let sentinel = CdrParameter::new(0x0001, &(), None);

        let mut state = serializer.serialize_struct("SpdpParticipantData", 3)?;
        state.serialize_field("", &CdrParameter::new(0x0031, &self.domain_id, None))?;
        state.serialize_field("", &CdrParameter::new(0x0022, &self.domain_tag, Some("".to_string())))?;
        state.serialize_field("", &CdrParameter::new(0x0055, &self.vendor_id, None))?;

        state.serialize_field("", &sentinel)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for SpdpParticipantData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        struct SpdpParticipantDataVisitor;

        impl<'de> Visitor<'de> for SpdpParticipantDataVisitor {
            // The type that our Visitor is going to produce.
            type Value = SpdpParticipantData;

            // Format a message stating what data this Visitor expects to receive.
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("SPDP participant data")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let value : GenericCdrParameter = seq.next_element()?.unwrap();
                let domain_id = value.value_as::<DomainId>();
                println!("Found: {:?}", domain_id);
                todo!()
            }
        }

        deserializer.deserialize_tuple(1024 /*just a very high number to make sure we go through all fields*/, SpdpParticipantDataVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cdr::{Infinite, PlCdrBe, PlCdrLe};

    #[test]
    fn serialize_vec_u8() {
        let a: [u8; 2] = [100, 200];
        let param1 = CdrParameter::new(0x0001, &a, None);

        let bytes = cdr::serialize::<_,_,PlCdrBe>(&param1, Infinite).unwrap();
        println!("Result: {:?}", bytes);

        let deserialized_param1 : GenericCdrParameter = cdr::deserialize(&bytes).unwrap();
        println!("Deserialized: {:?}", deserialized_param1);
    }


    #[test]
    fn serialize_single_parameter() {
        let a: i16 = 100;
        let param1 = CdrParameter::new(0x0001, &a, None);

        let bytes = cdr::serialize::<_,_,PlCdrBe>(&param1, Infinite).unwrap();
        println!("Result: {:?}", bytes);

        let deserialized_param1 : GenericCdrParameter = cdr::deserialize(&bytes).unwrap();
        println!("Deserialized: {:?}", deserialized_param1);
    }

    #[test]
    fn serialize_spdp_data() {
        let spdp_participant_data = SpdpParticipantData{
            domain_id: 1,
            domain_tag: "abcd".to_string(),
            vendor_id: [99,99],
        };

        let bytes = cdr::serialize::<_,_,PlCdrBe>(&spdp_participant_data, Infinite).unwrap();
        println!("Result: {:?}", bytes);

        let deserialized_spdp = cdr::deserialize::<SpdpParticipantData>(&bytes).unwrap();
        println!("Deserialized Result: {:?}", deserialized_spdp);
    }

}