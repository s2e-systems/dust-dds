use crate::xtypes::{dynamic_type::DynamicData, serialize::XTypesSerialize};

impl XTypesSerialize for DynamicData {
    fn serialize(
        &self,
        serializer: impl super::serialize::XTypesSerializer,
    ) -> Result<(), super::error::XTypesError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        infrastructure::type_support::TypeSupport, xtypes::xcdr_serializer::Xcdr1LeSerializer,
    };

    use super::*;

    #[derive(TypeSupport)]
    struct FinalStruct {
        id: u8,
        x: i64,
        y: i64,
    }

    #[test]
    fn serialize_final_struct_cdr1() {
        let data = FinalStruct {
            id: 1,
            x: 100,
            y: 200,
        };
        let dynamic_sample = data.create_dynamic_sample();
        let mut buffer = vec![];
        let mut serializer = Xcdr1LeSerializer::new(&mut buffer);
        dynamic_sample.serialize(&mut serializer).unwrap();

        assert_eq!(&buffer, &[1, 0, 0, 0, 0, 0, 0, 100, 0, 0, 0, 200])
    }
}
