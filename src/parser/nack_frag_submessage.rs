
use super::{Result};

#[derive(PartialEq, Debug)]
pub struct NackFrag {
}

pub fn parse_nack_frag_submessage(_submessage: &[u8], _submessage_flags: &u8) -> Result<NackFrag> {
    unimplemented!()
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_parse_nack_frag_submessage() {
        parse_nack_frag_submessage(&[0,0], &0).unwrap();
    }

}