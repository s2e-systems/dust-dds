use super::reader::RtpsReader;

pub struct RtpsStatelessReader<'a, L, C> {
    pub reader: &'a mut RtpsReader<L, C>,
}
