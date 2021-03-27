use rust_rtps_pim::messages::Submessage;

use crate::RtpsUdpPsm;

pub struct DataFrag<'a> {
    endianness_flag: <<Self as Submessage>::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
    inline_qos_flag: <<Self as Submessage>::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
    non_standard_payload_flag:
        <<Self as Submessage>::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
    key_flag: <<Self as Submessage>::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
    reader_id: rust_rtps_pim::messages::submessage_elements::EntityId<<Self as Submessage>::PSM>,
    writer_id: rust_rtps_pim::messages::submessage_elements::EntityId<<Self as Submessage>::PSM>,
    writer_sn:
        rust_rtps_pim::messages::submessage_elements::SequenceNumber<<Self as Submessage>::PSM>,
    fragment_starting_num:
        rust_rtps_pim::messages::submessage_elements::FragmentNumber<<Self as Submessage>::PSM>,
    fragments_in_submessage: rust_rtps_pim::messages::submessage_elements::UShort,
    data_size: rust_rtps_pim::messages::submessage_elements::ULong,
    fragment_size: rust_rtps_pim::messages::submessage_elements::UShort,
    inline_qos:
        rust_rtps_pim::messages::submessage_elements::ParameterList<<Self as Submessage>::PSM>,
    serialized_payload:
        rust_rtps_pim::messages::submessage_elements::SerializedDataFragment<&'a [u8]>,
}

impl<'a> Submessage for DataFrag<'a> {
    type PSM = RtpsUdpPsm;

    fn submessage_header(&self) -> rust_rtps_pim::messages::SubmessageHeader<Self::PSM> {
        todo!()
    }
}

impl<'a> rust_rtps_pim::messages::submessages::DataFrag for DataFrag<'a> {
    type SerializedDataFragment = &'a [u8];

    fn new(
        endianness_flag: <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
        inline_qos_flag: <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
        non_standard_payload_flag: <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
        key_flag: <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
        reader_id: rust_rtps_pim::messages::submessage_elements::EntityId<Self::PSM>,
        writer_id: rust_rtps_pim::messages::submessage_elements::EntityId<Self::PSM>,
        writer_sn: rust_rtps_pim::messages::submessage_elements::SequenceNumber<Self::PSM>,
        fragment_starting_num: rust_rtps_pim::messages::submessage_elements::FragmentNumber<
            Self::PSM,
        >,
        fragments_in_submessage: rust_rtps_pim::messages::submessage_elements::UShort,
        data_size: rust_rtps_pim::messages::submessage_elements::ULong,
        fragment_size: rust_rtps_pim::messages::submessage_elements::UShort,
        inline_qos: rust_rtps_pim::messages::submessage_elements::ParameterList<Self::PSM>,
        serialized_payload: rust_rtps_pim::messages::submessage_elements::SerializedDataFragment<
            Self::SerializedDataFragment,
        >,
    ) -> Self {
        Self {
            endianness_flag,
            inline_qos_flag,
            non_standard_payload_flag,
            key_flag,
            reader_id,
            writer_id,
            writer_sn,
            fragment_starting_num,
            fragments_in_submessage,
            data_size,
            fragment_size,
            inline_qos,
            serialized_payload,
        }
    }

    fn endianness_flag(&self) -> <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag {
        self.endianness_flag
    }

    fn inline_qos_flag(&self) -> <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag {
        self.inline_qos_flag
    }

    fn non_standard_payload_flag(
        &self,
    ) -> <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag {
        self.non_standard_payload_flag
    }

    fn key_flag(&self) -> <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag {
        self.key_flag
    }

    fn reader_id(&self) -> &rust_rtps_pim::messages::submessage_elements::EntityId<Self::PSM> {
        &self.reader_id
    }

    fn writer_id(&self) -> &rust_rtps_pim::messages::submessage_elements::EntityId<Self::PSM> {
        &self.writer_id
    }

    fn writer_sn(
        &self,
    ) -> &rust_rtps_pim::messages::submessage_elements::SequenceNumber<Self::PSM> {
        &self.writer_sn
    }

    fn fragment_starting_num(
        &self,
    ) -> &rust_rtps_pim::messages::submessage_elements::FragmentNumber<Self::PSM> {
        &self.fragment_starting_num
    }

    fn fragments_in_submessage(&self) -> &rust_rtps_pim::messages::submessage_elements::UShort {
        &self.fragments_in_submessage
    }

    fn data_size(&self) -> &rust_rtps_pim::messages::submessage_elements::ULong {
        &self.data_size
    }

    fn fragment_size(&self) -> &rust_rtps_pim::messages::submessage_elements::UShort {
        &self.fragment_size
    }

    fn inline_qos(
        &self,
    ) -> &rust_rtps_pim::messages::submessage_elements::ParameterList<Self::PSM> {
        &self.inline_qos
    }

    fn serialized_payload(
        &self,
    ) -> &rust_rtps_pim::messages::submessage_elements::SerializedDataFragment<
        Self::SerializedDataFragment,
    > {
        &self.serialized_payload
    }
}
