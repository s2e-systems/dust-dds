use super::submessage_elements;
use super::SubmessageFlag;
use super::{Submessage, SubmessageHeader};
use crate::messages::types::constants;

#[derive(PartialEq, Debug)]
pub struct AckNack {
    pub endianness_flag: SubmessageFlag,
    pub final_flag: SubmessageFlag,
    pub reader_id: submessage_elements::EntityId,
    pub writer_id: submessage_elements::EntityId,
    pub reader_sn_state: submessage_elements::SequenceNumberSet,
    pub count: submessage_elements::Count,
}

impl Submessage for AckNack {
    fn submessage_header(&self) -> SubmessageHeader {
        const X: SubmessageFlag = false;
        let e = self.endianness_flag;
        let f = self.final_flag;
        let flags = [e, f, X, X, X, X, X, X];

        SubmessageHeader::new(constants::SUBMESSAGE_KIND_ACK_NACK, flags, 0)
    }

    fn is_valid(&self) -> bool {
        todo!()
        // self.reader_sn_state.is_valid()
    }
}

impl serde::Serialize for AckNack {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
        // let mut state = serializer.serialize_struct("AckNack", 4)?;
        // state.serialize_field("header", &self.submessage_header())?;
        // state.end()
    }
}

#[cfg(test)]
mod tests {
    use serde_test::{assert_ser_tokens, Token};

    use crate::{
        messages::submessages::submessage_elements::SequenceNumberSet,
        types::constants::ENTITYID_PARTICIPANT,
    };

    use super::*;

    #[test]
    fn serialize() {
        let acknack = AckNack {
            endianness_flag: false,
            final_flag: true,
            reader_id: ENTITYID_PARTICIPANT,
            writer_id: ENTITYID_PARTICIPANT,
            reader_sn_state: SequenceNumberSet::new(10.into(), [10; 8]),
            count: 10,
        };

        assert_ser_tokens(
            &acknack,
            &[
                Token::Struct {
                    name: "AckNack",
                    len: 4,
                },
                Token::StructEnd,
            ],
        )
    }
}
