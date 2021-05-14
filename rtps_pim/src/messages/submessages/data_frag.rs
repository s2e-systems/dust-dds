use crate::{messages::submessage_elements, PIM};

pub struct DataFrag<PSM: PIM, SerializedDataFragment: AsRef<[u8]>> {
    pub endianness_flag: PSM::SubmessageFlag,
    pub inline_qos_flag: PSM::SubmessageFlag,
    pub non_standard_payload_flag: PSM::SubmessageFlag,
    pub key_flag: PSM::SubmessageFlag,
    pub reader_id: submessage_elements::EntityId<PSM>,
    pub writer_id: submessage_elements::EntityId<PSM>,
    pub writer_sn: submessage_elements::SequenceNumber<PSM>,
    pub fragment_starting_num: submessage_elements::FragmentNumber<PSM>,
    pub fragments_in_submessage: submessage_elements::UShort,
    pub data_size: submessage_elements::ULong,
    pub fragment_size: submessage_elements::UShort,
    pub inline_qos: submessage_elements::ParameterList<PSM>,
    pub serialized_payload: SerializedDataFragment,
}
