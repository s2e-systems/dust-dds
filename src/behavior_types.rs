/// 
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// Table 8.46 - Types definitions for the Behavior Module
///  

use std::convert::{TryInto, TryFrom, From};

pub mod constants {
    use super::Duration;

    pub const DURATION_ZERO: Duration = Duration {
        seconds: 0,
        fraction: 0,
    };

    pub const DURATION_INFINITE: Duration = Duration {
        seconds: 0x7fffffff,
        fraction: 0xffffffff,
    };
}


#[derive(PartialEq, Eq, PartialOrd, Hash, Debug, Clone, Copy)]
pub struct Duration {
    seconds: i32,
    fraction: u32,
}

impl Duration {
    pub fn from_millis(millis: u64) -> Self { 
        std::time::Duration::from_millis(millis).try_into().unwrap()
    }
}

impl TryFrom<std::time::Duration> for Duration {
    type Error = core::num::TryFromIntError;

    fn try_from(value: std::time::Duration) -> Result<Self, Self::Error> {
        let seconds: i32 = value.as_secs().try_into()?;
        let fraction = ((value.as_secs_f64() - value.as_secs() as f64) * 2_f64.powi(32)) as u32;
        Ok(Duration {seconds, fraction})
    }
}

impl From<Duration> for std::time::Duration {
    fn from(value: Duration) -> Self {
        let nanoseconds = (value.fraction as f64 * 1_000_000_000_f64 / 2_f64.powi(32)) as u32;
        std::time::Duration::new(value.seconds as u64, nanoseconds)
    }
}



#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum ChangeForReaderStatusKind {
    Unsent,
    Unacknowledged,
    Requested,
    Acknowledged,
    Underway,
}



#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum ChangeFromWriterStatusKind {
    Lost,
    Missing,
    Received,
    Unknown,
}


// InstanceHandle should already be defined in structre types



// todo: ParticipantMessageData



#[cfg(test)]
mod tests {
    use super::*;
     
    #[test]
    fn duration_construction() {
        let result = Duration::try_from(std::time::Duration::new(2, 1)).unwrap();
        assert_eq!(result.seconds, 2);
        assert_eq!(result.fraction, 4);

        let result = Duration::try_from(std::time::Duration::from_millis(500)).unwrap();
        assert_eq!(result.seconds, 0);
        assert_eq!(result.fraction, 2_u32.pow(31));

        let result = Duration::try_from(std::time::Duration::from_secs(2_u64.pow(40)));
        assert!(result.is_err());  
    }

    #[test] #[should_panic]
    fn duration_from_invalid() {
        Duration::from_millis(2_u64.pow(32) * 1000 + 1);
    }

}