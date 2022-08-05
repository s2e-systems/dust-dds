use std::convert::TryFrom;

use dds_transport::messages::{
    submessage_elements::{
        EntityIdSubmessageElement, Parameter, ParameterListSubmessageElement,
        SequenceNumberSubmessageElement, SerializedDataSubmessageElement,
    },
    submessages::DataSubmessage,
    types::ParameterId,
};

use crate::{
    dcps_psm::{InstanceHandle, Time},
    implementation::{
        data_representation_inline_qos::{
            parameter_id_values::PID_STATUS_INFO,
            types::{STATUS_INFO_DISPOSED_FLAG, STATUS_INFO_UNREGISTERED_FLAG},
        },
        dds_impl::message_receiver::MessageReceiver,
    },
    return_type::DdsError,
};

use super::{
    history_cache::RtpsParameter,
    types::{ChangeKind, Guid, SequenceNumber, ENTITYID_UNKNOWN},
};

#[derive(Debug)]

pub struct RtpsReaderCacheChange {
    kind: ChangeKind,
    writer_guid: Guid,
    sequence_number: SequenceNumber,
    instance_handle: InstanceHandle,
    data: Vec<u8>,
    inline_qos: Vec<RtpsParameter>,
    source_timestamp: Option<Time>,
}

impl PartialEq for RtpsReaderCacheChange {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.writer_guid == other.writer_guid
            && self.sequence_number == other.sequence_number
            && self.instance_handle == other.instance_handle
    }
}

impl TryFrom<(&MessageReceiver, &DataSubmessage<'_>)> for RtpsReaderCacheChange {
    type Error = DdsError;

    fn try_from(value: (&MessageReceiver, &DataSubmessage<'_>)) -> Result<Self, Self::Error> {
        let (message_receiver, data) = value;
        let writer_guid = Guid::new(
            message_receiver.source_guid_prefix(),
            data.writer_id.value.into(),
        );

        let instance_handle = [0; 16];
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

        Ok(RtpsReaderCacheChange {
            kind,
            writer_guid,
            instance_handle,
            sequence_number,
            data: data_value,
            inline_qos,
            source_timestamp,
        })
    }
}

impl<'a> From<&'a RtpsReaderCacheChange> for DataSubmessage<'a> {
    fn from(val: &'a RtpsReaderCacheChange) -> Self {
        let endianness_flag = true;
        let inline_qos_flag = true;
        let (data_flag, key_flag) = match val.kind() {
            ChangeKind::Alive => (true, false),
            ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered => (false, true),
            _ => todo!(),
        };
        let non_standard_payload_flag = false;
        let reader_id = EntityIdSubmessageElement {
            value: ENTITYID_UNKNOWN.into(),
        };
        let writer_id = EntityIdSubmessageElement {
            value: val.writer_guid().entity_id().into(),
        };
        let writer_sn = SequenceNumberSubmessageElement {
            value: val.sequence_number(),
        };
        let inline_qos = ParameterListSubmessageElement {
            parameter: val
                .inline_qos()
                .iter()
                .map(|p| Parameter {
                    parameter_id: p.parameter_id().0,
                    length: p.value().len() as i16,
                    value: p.value(),
                })
                .collect(),
        };
        let serialized_payload = SerializedDataSubmessageElement {
            value: val.data_value(),
        };
        DataSubmessage {
            endianness_flag,
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        }
    }
}

impl RtpsReaderCacheChange {
    pub fn new(
        kind: ChangeKind,
        writer_guid: Guid,
        instance_handle: InstanceHandle,
        sequence_number: SequenceNumber,
        data_value: Vec<u8>,
        inline_qos: Vec<RtpsParameter>,
        source_timestamp: Option<Time>,
    ) -> Self {
        Self {
            kind,
            writer_guid,
            sequence_number,
            instance_handle,
            data: data_value,
            inline_qos,
            source_timestamp,
        }
    }

    pub fn kind(&self) -> ChangeKind {
        self.kind
    }

    pub fn writer_guid(&self) -> Guid {
        self.writer_guid
    }

    pub fn instance_handle(&self) -> InstanceHandle {
        self.instance_handle
    }

    pub fn sequence_number(&self) -> SequenceNumber {
        self.sequence_number
    }

    pub fn data_value(&self) -> &[u8] {
        self.data.as_ref()
    }

    pub fn inline_qos(&self) -> &[RtpsParameter] {
        &self.inline_qos
    }

    pub fn source_timestamp(&self) -> &Option<Time> {
        &self.source_timestamp
    }
}
