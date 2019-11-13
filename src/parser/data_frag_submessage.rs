use super::{DataFrag, Result};

pub fn parse_data_frag_submessage(_submessage: &[u8], _submessage_flags: &u8) -> Result<DataFrag> {
    unimplemented!()
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_parse_data_frag_submessage() {
        parse_data_frag_submessage(&[0,0], &0);
    }
}