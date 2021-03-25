pub mod reader;
// pub mod stateful_reader;
// pub mod stateful_writer;
// pub mod stateless_reader;
pub mod stateless_writer;
pub mod types;
pub mod writer;

pub use reader::RTPSReader;
// pub use stateful_reader::{RTPSStatefulReader, RTPSWriterProxy};
// pub use stateful_writer::{RTPSReaderProxy, RTPSStatefulWriter};
// pub use stateless_reader::RTPSStatelessReader;
// pub use stateless_writer::RTPSStatelessWriter;
pub use writer::RTPSWriter;

use crate::{
    messages::{
        self,
        submessages::{self, submessage_elements},
    },
    structure::RTPSCacheChange,
    types::{ChangeKind, GUID},
};

// use crate::{
//     messages::{
//         submessages::{self, submessage_elements::SerializedData},
//         types::StatusInfo,
//     },
//     structure::RTPSCacheChange,
//     types::{ChangeKind, EntityId, GUID},
// };

// fn _cache_change_from_data<'a, T, C>(mut message: Data<'a>, guid_prefix: &GuidPrefix) -> C
// where
//     T: From<SerializedData<'a>>,
//     C: RTPSCacheChange<Data = T>,
// {
//     let change_kind = change_kind(&message);
//     let key_hash = key_hash(&message).unwrap();

//     message
//         .inline_qos
//         .parameter
//         .retain(|x| x.parameter_id() != PID_KEY_HASH);
//     message
//         .inline_qos
//         .parameter
//         .retain(|x| x.parameter_id() != PID_STATUS_INFO);

//     C::new(
//         change_kind,
//         GUID::new(*guid_prefix, message.writer_id).into(),
//         key_hash.0,
//         message.writer_sn,
//         message.serialized_payload.into(),
//         message.inline_qos,
//     )
// }
// type EntityId: types::EntityId;
// type SequenceNumber: types::SequenceNumber;
// type ParameterId: messages::types::ParameterId;
// type ParameterValue: AsRef<[u8]>;
// type ParameterList: IntoIterator<
//     Item = submessage_elements::Parameter<Self::ParameterId, Self::ParameterValue>,
// >;
// type SerializedData: AsRef<[u8]>;

pub fn data_submessage_from_cache_change<
    'a,
    DataSubmessage,
    GuidPrefix: crate::types::GuidPrefix,
    InstanceHandle: crate::types::InstanceHandle,
    Data: AsRef<[u8]>,
    // ParameterId: messages::types::ParameterId,
    // ParameterValue: AsRef<[u8]>,
    // ParameterList: IntoIterator<Item = submessage_elements::Parameter<ParameterId, ParameterValue>>,
>(
    cache_change: &'a RTPSCacheChange<
        GuidPrefix,
        DataSubmessage::EntityId,
        InstanceHandle,
        DataSubmessage::SequenceNumber,
        Data,
        // ParameterId,
        // ParameterValue,
        // ParameterList,
    >,
    reader_id: DataSubmessage::EntityId,
) -> DataSubmessage
where
    DataSubmessage: submessages::data_submessage::Data<
        SerializedData = &'a [u8],
        // ParameterId = ParameterId,
        // ParameterValue = ParameterValue,
        // ParameterList = &'a ParameterList,
    >,
    // &'a ParameterList:
        // IntoIterator<Item = submessage_elements::Parameter<ParameterId, ParameterValue>>,
{
    let endianness_flag = true.into();
    let non_standard_payload_flag = false.into();
    let reader_id = submessage_elements::EntityId { value: reader_id };
    let writer_id = submessage_elements::EntityId {
        value: cache_change.writer_guid.entity_id,
    };
    let writer_sn = submessage_elements::SequenceNumber {
        value: cache_change.sequence_number,
    };

    // let inline_qos = submessage_elements::ParameterList {
    //     parameter: &cache_change.inline_qos.parameter,
    // };
    let serialized_payload = submessage_elements::SerializedData {
        value: cache_change.data_value.as_ref(),
    };

    match cache_change.kind {
        crate::types::ChangeKind::Alive => {
            let data_flag = true.into();
            let key_flag = false.into();
            let inline_qos_flag = true.into();
            DataSubmessage::new(
                endianness_flag,
                inline_qos_flag,
                data_flag,
                key_flag,
                non_standard_payload_flag,
                reader_id,
                writer_id,
                writer_sn,
                // inline_qos,
                serialized_payload,
            )
        }
        ChangeKind::NotAliveDisposed
        | ChangeKind::NotAliveUnregistered
        | ChangeKind::AliveFiltered => {
            let data_flag = false.into();
            let key_flag = true.into();
            let inline_qos_flag = true.into();
            DataSubmessage::new(
                endianness_flag,
                inline_qos_flag,
                data_flag,
                key_flag,
                non_standard_payload_flag,
                reader_id,
                writer_id,
                writer_sn,
                // inline_qos,
                serialized_payload,
            )
        }
    }
}

// // fn change_kind(data_submessage: &Data) -> ChangeKind {
// //     if data_submessage.data_flag && !data_submessage.key_flag {
// //         ChangeKind::Alive
// //     } else if !data_submessage.data_flag && data_submessage.key_flag {
// //         // let endianness = Endianness::from(data_submessage.endianness_flag()).into();
// //         let status_info = data_submessage
// //             .inline_qos
// //             .parameter
// //             .iter()
// //             .find(|&x| x.parameter_id() == PID_STATUS_INFO)
// //             .unwrap()
// //             .clone();

// //         status_info_to_change_kind(status_info.try_into().unwrap()).unwrap()
// //     } else {
// //         panic!("Invalid change kind combination")
// //     }
// // }

// // fn key_hash(data_submessage: &Data) -> Option<KeyHash> {
// //     if data_submessage.data_flag && !data_submessage.key_flag {
// //         Some(
// //             data_submessage
// //                 .inline_qos
// //                 .parameter
// //                 .iter()
// //                 .find(|&x| x.parameter_id() == PID_KEY_HASH)
// //                 .unwrap()
// //                 .clone()
// //                 .try_into()
// //                 .unwrap(),
// //         )
// //     } else if !data_submessage.data_flag && data_submessage.key_flag {
// //         let payload = &data_submessage.serialized_payload;
// //         Some(KeyHash(payload[0..16].try_into().ok()?))
// //     } else {
// //         None
// //     }
// // }

// // fn status_info_to_change_kind(status_info: StatusInfo) -> Option<ChangeKind> {
// //     if status_info.disposed_flag()
// //         && !status_info.unregistered_flag()
// //         && !status_info.filtered_flag()
// //     {
// //         Some(ChangeKind::NotAliveDisposed)
// //     } else if !status_info.disposed_flag()
// //         && status_info.unregistered_flag()
// //         && !status_info.filtered_flag()
// //     {
// //         Some(ChangeKind::NotAliveUnregistered)
// //     } else if !status_info.disposed_flag()
// //         && !status_info.unregistered_flag()
// //         && status_info.filtered_flag()
// //     {
// //         Some(ChangeKind::AliveFiltered)
// //     } else if !status_info.disposed_flag()
// //         && !status_info.unregistered_flag()
// //         && !status_info.filtered_flag()
// //     {
// //         Some(ChangeKind::Alive)
// //     } else {
// //         None
// //     }
// // }

// pub fn change_kind_to_status_info(change_kind: ChangeKind) -> StatusInfo {
//     match change_kind {
//         ChangeKind::Alive => StatusInfo([0, 0, 0, 0]),
//         ChangeKind::NotAliveDisposed => StatusInfo([0, 0, 0, StatusInfo::DISPOSED_FLAG_MASK]),
//         ChangeKind::NotAliveUnregistered => {
//             StatusInfo([0, 0, 0, StatusInfo::UNREGISTERED_FLAG_MASK])
//         }
//         ChangeKind::AliveFiltered => StatusInfo([0, 0, 0, StatusInfo::FILTERED_FLAG_MASK]),
//     }
// }

#[cfg(test)]
mod tests {
    use messages::submessages::submessage_elements::ParameterList;

    use crate::messages::submessages::Submessage;

    use super::*;

    //     #[derive(Clone, Copy)]
    //     struct InstanceHandle;
    //     impl crate::types::InstanceHandle for InstanceHandle {}

    //     #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    //     struct SequenceNumber(i64);
    //     impl crate::types::SequenceNumber for SequenceNumber {
    //         const SEQUENCE_NUMBER_UNKNOWN: Self = Self(0);
    //     }

    //     impl From<i64> for SequenceNumber {
    //         fn from(_: i64) -> Self {
    //             todo!()
    //         }
    //     }
    //     impl Into<i64> for SequenceNumber {
    //         fn into(self) -> i64 {
    //             todo!()
    //         }
    //     }

    //     #[derive(Clone, Copy)]
    //     struct GuidPrefix([u8; 12]);
    //     impl crate::types::GuidPrefix for GuidPrefix {
    //         const GUIDPREFIX_UNKNOWN: Self = Self([0; 12]);
    //     }

    //     impl From<[u8; 12]> for GuidPrefix {
    //         fn from(_: [u8; 12]) -> Self {
    //             todo!()
    //         }
    //     }

    //     impl Into<[u8; 12]> for GuidPrefix {
    //         fn into(self) -> [u8; 12] {
    //             todo!()
    //         }
    //     }

    //     #[derive(Clone, Copy)]
    //     struct EntityId([u8; 4]);
    //     impl crate::types::EntityId for EntityId {
    //         const ENTITYID_UNKNOWN: Self = Self([0; 4]);
    //     }

    //     impl From<[u8; 4]> for EntityId {
    //         fn from(_: [u8; 4]) -> Self {
    //             todo!()
    //         }
    //     }

    //     impl Into<[u8; 4]> for EntityId {
    //         fn into(self) -> [u8; 4] {
    //             todo!()
    //         }
    //     }

    //     #[derive(Clone, Copy)]
    //     struct GUID {
    //         prefix: GuidPrefix,
    //         entity_id: EntityId,
    //     }

    //     impl crate::types::GUID for GUID {
    //         type GuidPrefix = GuidPrefix;
    //         type EntityId = EntityId;

    //         fn guid_prefix(&self) -> Self::GuidPrefix {
    //             self.prefix
    //         }

    //         fn entity_id(&self) -> Self::EntityId {
    //             self.entity_id
    //         }

    //         const GUID_UNKNOWN: Self = Self {
    //             prefix: GuidPrefix([0; 12]),
    //             entity_id: EntityId([0; 4]),
    //         };
    //     }

    //     impl From<[u8; 16]> for GUID {
    //         fn from(_: [u8; 16]) -> Self {
    //             todo!()
    //         }
    //     }

    //     impl Into<[u8; 16]> for GUID {
    //         fn into(self) -> [u8; 16] {
    //             todo!()
    //         }
    //     }

    //     #[derive(Clone, Copy)]
    //     struct ParameterId(u16);

    //     impl crate::messages::types::ParameterId for ParameterId {}
    //     struct Parameter;

    //     impl submessage_elements::Parameter for Parameter {
    //         type ParameterId = ParameterId;

    //         fn parameter_id(&self) -> Self::ParameterId {
    //             todo!()
    //         }

    //         fn value(&self) -> &[u8] {
    //             todo!()
    //         }
    //     }
    //     #[derive(Clone, Copy)]
    //     struct ParameterList;

    //     impl submessage_elements::SubmessageElement for ParameterList {}
    //     impl submessage_elements::ParameterList for ParameterList {
    //         type Parameter = Parameter;

    //         fn parameter(&self) -> &[Self::Parameter] {
    //             todo!()
    //         }
    //     }

    //     struct MockCacheChange {
    //         kind: ChangeKind,
    //         writer_guid: GUID,
    //         instance_handle: InstanceHandle,
    //         sequence_number: SequenceNumber,
    //         data_value: [u8; 4],
    //         inline_qos: ParameterList,
    //     }

    //     impl RTPSCacheChange for MockCacheChange {
    //         type InstanceHandle = InstanceHandle;
    //         type SequenceNumber = SequenceNumber;
    //         type GuidPrefix = GuidPrefix;
    //         type EntityId = EntityId;
    //         type GUID = GUID;
    //         type Data = [u8; 4];
    //         type ParameterList = ParameterList;

    //         fn new(
    //             kind: ChangeKind,
    //             writer_guid: Self::GUID,
    //             instance_handle: Self::InstanceHandle,
    //             sequence_number: Self::SequenceNumber,
    //             data_value: Self::Data,
    //             inline_qos: Self::ParameterList,
    //         ) -> Self {
    //             Self {
    //                 kind,
    //                 writer_guid,
    //                 instance_handle,
    //                 sequence_number,
    //                 data_value,
    //                 inline_qos,
    //             }
    //         }

    //         fn kind(&self) -> ChangeKind {
    //             self.kind
    //         }

    //         fn writer_guid(&self) -> Self::GUID {
    //             self.writer_guid
    //         }

    //         fn instance_handle(&self) -> Self::InstanceHandle {
    //             self.instance_handle
    //         }

    //         fn sequence_number(&self) -> Self::SequenceNumber {
    //             self.sequence_number
    //         }

    //         fn data_value(&self) -> &Self::Data {
    //             &self.data_value
    //         }

    //         fn inline_qos(&self) -> &Self::ParameterList {
    //             &self.inline_qos
    //         }
    //     }

    #[derive(Clone, Copy)]
    struct MockSubmessageKind(u8);

    impl crate::messages::types::SubmessageKind for MockSubmessageKind {
        const DATA: Self = Self(0);
        const GAP: Self = Self(0);
        const HEARTBEAT: Self = Self(0);
        const ACKNACK: Self = Self(0);
        const PAD: Self = Self(0);
        const INFO_TS: Self = Self(0);
        const INFO_REPLY: Self = Self(0);
        const INFO_DST: Self = Self(0);
        const INFO_SRC: Self = Self(0);
        const DATA_FRAG: Self = Self(0);
        const NACK_FRAG: Self = Self(0);
        const HEARTBEAT_FRAG: Self = Self(0);
    }

    #[derive(Clone, Copy)]
    struct MockSubmessageFlag(bool);
    impl crate::messages::types::SubmessageFlag for MockSubmessageFlag {}

    impl From<bool> for MockSubmessageFlag {
        fn from(value: bool) -> Self {
            Self(value)
        }
    }

    impl Into<bool> for MockSubmessageFlag {
        fn into(self) -> bool {
            todo!()
        }
    }

    //     struct MockSubmessageHeader;

    //     impl crate::messages::submessages::SubmessageHeader for MockSubmessageHeader {
    //         type SubmessageKind = MockSubmessageKind;
    //         type SubmessageFlag = MockSubmessageFlag;

    //         fn submessage_id(&self) -> Self::SubmessageKind {
    //             todo!()
    //         }

    //         fn flags(&self) -> [Self::SubmessageFlag; 8] {
    //             todo!()
    //         }

    //         fn submessage_length(&self) -> u16 {
    //             todo!()
    //         }
    //     }

    //     impl<'a> crate::messages::submessages::Submessage for MockDataSubmessage<'a> {
    //         type SubmessageHeader = MockSubmessageHeader;

    //         fn submessage_header(&self) -> Self::SubmessageHeader {
    //             todo!()
    //         }
    //     }

    //     impl submessage_elements::SubmessageElement for EntityId {}

    //     impl submessage_elements::EntityId for EntityId {
    //         type EntityId = Self;

    //         fn new(value: Self::EntityId) -> Self {
    //             value
    //         }

    //         fn value(&self) -> Self::EntityId {
    //             *self
    //         }
    //     }

    //     impl submessage_elements::SubmessageElement for SequenceNumber {}

    //     impl submessage_elements::SequenceNumber for SequenceNumber {
    //         type SequenceNumber = Self;

    //         fn new(value: Self::SequenceNumber) -> Self {
    //             value
    //         }

    //         fn value(&self) -> Self::SequenceNumber {
    //             *self
    //         }

    //         const SEQUENCENUMBER_UNKNOWN: Self = Self(0);
    //     }

    //     struct SerializedData<'a>(&'a [u8; 4]);

    //     impl<'a> submessage_elements::SubmessageElement for SerializedData<'a> {}

    //     impl<'a> submessage_elements::SerializedData for SerializedData<'a> {
    //         type SerializedData = &'a [u8; 4];
    //         fn new(value: Self::SerializedData) -> Self {
    //             Self(value)
    //         }

    //         fn value(&self) -> &[u8] {
    //             todo!()
    //         }
    //     }

    //     impl<'a> submessage_elements::SubmessageElement for &'a ParameterList {}

    //     impl<'a> submessage_elements::ParameterList for &'a ParameterList {
    //         type Parameter = <ParameterList as submessage_elements::ParameterList>::Parameter;

    //         fn parameter(&self) -> &[Self::Parameter] {
    //             todo!()
    //         }
    //     }

    impl crate::types::EntityId for [u8; 4] {
        const ENTITYID_UNKNOWN: Self = [0; 4];
    }

    impl crate::types::SequenceNumber for i64 {
        const SEQUENCE_NUMBER_UNKNOWN: Self = 0;
    }

    impl crate::messages::types::ParameterId for u16 {}

    impl crate::types::GuidPrefix for [u8; 12] {
        const GUIDPREFIX_UNKNOWN: Self = [0; 12];
    }

    impl crate::types::InstanceHandle for i32 {}

    impl crate::messages::types::ParameterId for i16 {}

    struct MockDataSubmessage<'a> {
        endianness_flag: MockSubmessageFlag,
        inline_qos_flag: MockSubmessageFlag,
        data_flag: MockSubmessageFlag,
        key_flag: MockSubmessageFlag,
        non_standard_payload_flag: MockSubmessageFlag,
        reader_id: submessage_elements::EntityId<[u8; 4]>,
        writer_id: submessage_elements::EntityId<[u8; 4]>,
        writer_sn: submessage_elements::SequenceNumber<i64>,
        // inline_qos: submessage_elements::ParameterList<
        //     u16,
        //     Vec<u8>,
        //     &'a Vec<submessage_elements::Parameter<u16, Vec<u8>>>,
        // >,
        serialized_payload: submessage_elements::SerializedData<&'a [u8]>,
    }

    impl<'a> Submessage for MockDataSubmessage<'a> {
        type SubmessageKind = MockSubmessageKind;
        type SubmessageFlag = MockSubmessageFlag;

        fn submessage_header(
            &self,
        ) -> submessages::SubmessageHeader<Self::SubmessageKind, Self::SubmessageFlag> {
            todo!()
        }
    }

    impl<'a> crate::messages::submessages::data_submessage::Data for MockDataSubmessage<'a> {
        type EntityId = [u8; 4];
        type SequenceNumber = i64;
        type ParameterId = u16;
        type ParameterValue = [u8; 4];
        type ParameterList =
            Vec<submessage_elements::Parameter<Self::ParameterId, Self::ParameterValue>>;
        type SerializedData = &'a [u8];

        fn new(
            endianness_flag: MockSubmessageFlag,
            inline_qos_flag: MockSubmessageFlag,
            data_flag: MockSubmessageFlag,
            key_flag: MockSubmessageFlag,
            non_standard_payload_flag: MockSubmessageFlag,
            reader_id: submessage_elements::EntityId<Self::EntityId>,
            writer_id: submessage_elements::EntityId<Self::EntityId>,
            writer_sn: submessage_elements::SequenceNumber<Self::SequenceNumber>,
            // inline_qos: submessage_elements::ParameterList<
            //     Self::ParameterId,
            //     Self::ParameterValue,
            //     Self::ParameterList,
            // >,
            serialized_payload: submessage_elements::SerializedData<Self::SerializedData>,
        ) -> Self {
            Self {
                endianness_flag,
                inline_qos_flag,
                data_flag,
                key_flag,
                non_standard_payload_flag,
                reader_id,
                writer_id,
                writer_sn,
                serialized_payload,

            }
        }

        fn endianness_flag(&self) -> MockSubmessageFlag {
            todo!()
        }

        fn inline_qos_flag(&self) -> MockSubmessageFlag {
            todo!()
        }

        fn data_flag(&self) -> MockSubmessageFlag {
            todo!()
        }

        fn key_flag(&self) -> MockSubmessageFlag {
            todo!()
        }

        fn non_standard_payload_flag(&self) -> MockSubmessageFlag {
            todo!()
        }

        fn reader_id(&self) -> &submessage_elements::EntityId<Self::EntityId> {
            todo!()
        }

        fn writer_id(&self) -> &submessage_elements::EntityId<Self::EntityId> {
            todo!()
        }

        fn writer_sn(&self) -> &submessage_elements::SequenceNumber<Self::SequenceNumber> {
            todo!()
        }

        // fn inline_qos(
        //     &self,
        // ) -> &submessage_elements::ParameterList<
        //     Self::ParameterId,
        //     Self::ParameterValue,
        //     Self::ParameterList,
        // > {
        //     todo!()
        // }

        fn serialized_payload(&self) -> &submessage_elements::SerializedData<Self::SerializedData> {
            todo!()
        }
    }

    #[test]
    fn create_data_submessage_from_cache_change() {
        let change_kind = ChangeKind::Alive;
        let writer_id = [1, 2, 4, 5];
        let writer_guid = GUID {
            guid_prefix: [1; 12],
            entity_id: writer_id,
        };
        let instance_handle = 0;
        let sequence_number = 2;
        let data_value = vec![1, 2, 3, 4];
        let inline_qos: submessage_elements::ParameterList<
            i16,
            Vec<u8>,
            Vec<submessage_elements::Parameter<i16, Vec<u8>>>,
        > = submessage_elements::ParameterList { parameter: vec![] };
        let cache_change = RTPSCacheChange {
            kind: change_kind,
            writer_guid,
            instance_handle,
            sequence_number,
            data_value: data_value.clone(),
            // inline_qos,
        };

        let reader_id = [5; 4];

        let data_submessage: MockDataSubmessage =
            data_submessage_from_cache_change(&cache_change, reader_id);

        assert_eq!(data_submessage.endianness_flag.0, true);
        assert_eq!(data_submessage.inline_qos_flag.0, true);
        assert_eq!(data_submessage.data_flag.0, true);
        assert_eq!(data_submessage.key_flag.0, false);
        assert_eq!(data_submessage.non_standard_payload_flag.0, false);
        assert_eq!(data_submessage.reader_id.value, reader_id);
        assert_eq!(data_submessage.writer_id.value, writer_id);
        assert_eq!(data_submessage.writer_sn.value, sequence_number);
        assert_eq!(data_submessage.serialized_payload.value, &data_value);
    }

    //     //     #[test]
    //     //     fn status_info_change_kind_conversions() {
    //     //         assert_eq!(
    //     //             status_info_to_change_kind(change_kind_to_status_info(ChangeKind::Alive)).unwrap(),
    //     //             ChangeKind::Alive
    //     //         );
    //     //         assert_eq!(
    //     //             status_info_to_change_kind(change_kind_to_status_info(ChangeKind::AliveFiltered))
    //     //                 .unwrap(),
    //     //             ChangeKind::AliveFiltered
    //     //         );
    //     //         assert_eq!(
    //     //             status_info_to_change_kind(change_kind_to_status_info(
    //     //                 ChangeKind::NotAliveUnregistered
    //     //             ))
    //     //             .unwrap(),
    //     //             ChangeKind::NotAliveUnregistered
    //     //         );
    //     //         assert_eq!(
    //     //             status_info_to_change_kind(change_kind_to_status_info(ChangeKind::NotAliveDisposed))
    //     //                 .unwrap(),
    //     //             ChangeKind::NotAliveDisposed
    //     //         );
    //     //     }
}
