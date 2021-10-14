use super::reader::RtpsReader;

pub struct RtpsStatelessReader<L, C>(pub RtpsReader<L, C>);
