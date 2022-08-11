use dds_transport::messages::{submessages::DataSubmessage, types::ParameterId};

use crate::{
    dcps_psm::InstanceHandle,
    dds_type::{DdsDeserialize, DdsType, LittleEndian},
    implementation::{
        data_representation_inline_qos::{
            parameter_id_values::PID_STATUS_INFO,
            types::{STATUS_INFO_DISPOSED_FLAG, STATUS_INFO_UNREGISTERED_FLAG},
        },
        rtps::{
            history_cache::RtpsParameter,
            reader_cache_change::RtpsReaderCacheChange,
            types::{ChangeKind, Guid},
        },
    },
    return_type::{DdsError, DdsResult},
};

use super::message_receiver::MessageReceiver;

fn calculate_instance_handle(serialized_key: &[u8]) -> InstanceHandle {
    if serialized_key.len() <= 16 {
        let mut h = [0; 16];
        h[..serialized_key.len()].clone_from_slice(serialized_key);
        h.into()
    } else {
        <[u8; 16]>::from(md5::compute(serialized_key)).into()
    }
}

pub struct DataSubmessageHandler {
    serialized_data_to_key_func: fn(&[u8]) -> DdsResult<Vec<u8>>,
}

impl DataSubmessageHandler {
    pub fn new<T>() -> Self
    where
        T: for<'de> DdsDeserialize<'de> + DdsType,
    {
        // Create a function that deserializes the data and gets the key for the type
        // without having to store the actual type intermediatelly to avoid generics
        fn serialized_data_to_key_func<T>(mut buf: &[u8]) -> DdsResult<Vec<u8>>
        where
            T: for<'de> DdsDeserialize<'de> + DdsType,
        {
            Ok(T::deserialize(&mut buf)?.get_serialized_key::<LittleEndian>())
        }

        Self {
            serialized_data_to_key_func: serialized_data_to_key_func::<T>,
        }
    }

    pub fn try_into_reader_cache_change(
        &self,
        message_receiver: &MessageReceiver,
        data: &DataSubmessage<'_>,
    ) -> DdsResult<RtpsReaderCacheChange> {
        let writer_guid = Guid::new(
            message_receiver.source_guid_prefix(),
            data.writer_id.value.into(),
        );

        let instance_handle = calculate_instance_handle(
            (self.serialized_data_to_key_func)(data.serialized_payload.value)?.as_ref(),
        );
        let sequence_number = data.writer_sn.value;
        let data_value = data.serialized_payload.value.to_vec();

        let inline_qos: Vec<RtpsParameter> = data
            .inline_qos
            .parameter
            .iter()
            .map(|p| RtpsParameter::new(ParameterId(p.parameter_id), p.value.to_vec()))
            .collect();

        let kind = match (data.data_flag, data.key_flag) {
            (true, false) => Ok(ChangeKind::Alive),
            (false, true) => {
                if let Some(p) = inline_qos
                    .iter()
                    .find(|&x| x.parameter_id() == ParameterId(PID_STATUS_INFO))
                {
                    let mut deserializer =
                        cdr::Deserializer::<_, _, cdr::LittleEndian>::new(p.value(), cdr::Infinite);
                    let status_info = serde::Deserialize::deserialize(&mut deserializer).unwrap();
                    match status_info {
                        STATUS_INFO_DISPOSED_FLAG => Ok(ChangeKind::NotAliveDisposed),
                        STATUS_INFO_UNREGISTERED_FLAG => Ok(ChangeKind::NotAliveUnregistered),
                        _ => Err(DdsError::PreconditionNotMet(
                            "Unknown status info value".to_string(),
                        )),
                    }
                } else {
                    Err(DdsError::PreconditionNotMet(
                        "Missing mandatory StatusInfo parameter".to_string(),
                    ))
                }
            }
            _ => Err(DdsError::PreconditionNotMet(
                "Invalid data submessage data and key flag combination".to_string(),
            )),
        }?;

        let source_timestamp = if message_receiver.have_timestamp() {
            Some(message_receiver.timestamp())
        } else {
            None
        };

        Ok(RtpsReaderCacheChange::new(
            kind,
            writer_guid,
            instance_handle,
            sequence_number,
            data_value,
            inline_qos,
            source_timestamp,
        ))
    }
}

#[cfg(test)]
mod tests {
    use dds_transport::messages::submessage_elements::{
        EntityIdSubmessageElement, ParameterListSubmessageElement, SequenceNumberSubmessageElement,
        SerializedDataSubmessageElement,
    };

    use crate::dds_type::DdsSerde;

    use super::*;

    #[derive(serde::Serialize, serde::Deserialize)]
    struct DummyData {
        key: [u8; 2],
        data: [u8; 2],
    }

    impl DdsType for DummyData {
        fn type_name() -> &'static str {
            "DummyData"
        }

        fn has_key() -> bool {
            true
        }

        fn get_serialized_key<E: crate::dds_type::Endianness>(&self) -> Vec<u8> {
            self.key.to_vec()
        }

        fn set_key_fields_from_serialized_key(&mut self, _key: &[u8]) -> DdsResult<()> {
            if Self::has_key() {
                unimplemented!("DdsType with key must provide an implementation for set_key_fields_from_serialized_key")
            }
            Ok(())
        }
    }

    impl DdsSerde for DummyData {}

    #[test]
    fn convert_data_submessage_to_alive_cache_change_with_key() {
        let data_submessage_handler = DataSubmessageHandler::new::<DummyData>();

        let message_receiver = MessageReceiver::new();
        let mut data = DataSubmessage {
            endianness_flag: false,
            inline_qos_flag: false,
            data_flag: true,
            key_flag: false,
            non_standard_payload_flag: false,
            reader_id: EntityIdSubmessageElement { value: [0; 4] },
            writer_id: EntityIdSubmessageElement { value: [1; 4] },
            writer_sn: SequenceNumberSubmessageElement { value: 1 },
            inline_qos: ParameterListSubmessageElement { parameter: vec![] },
            serialized_payload: SerializedDataSubmessageElement {
                value: &[0, 0, 0, 0, 1, 2, 3, 4],
            },
        };

        let expected_cache_change = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1].into(),
            [1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].into(),
            1,
            vec![1, 2, 3, 4],
            vec![],
            None,
        );

        let cache_change = data_submessage_handler
            .try_into_reader_cache_change(&message_receiver, &mut data)
            .unwrap();

        assert_eq!(cache_change, expected_cache_change);
    }
}
