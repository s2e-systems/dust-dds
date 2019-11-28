use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::process;
use rand;
use crate::types::GuidPrefix;


pub fn generate_guid_prefix() -> GuidPrefix {
    let process_id = process::id().to_be_bytes();
    let random = rand::random::<u32>().to_be_bytes();
    let time_now = SystemTime::now();

    // TODO: Print a log message if result Error
    let time_unix_epoch = time_now.duration_since(UNIX_EPOCH).unwrap_or_else(|_| Duration::new(0,0));

    let timestamp = ((time_unix_epoch.as_secs() & 0xFFFF_FFFF) as u32).to_be_bytes();

    [process_id[0], process_id[1], process_id[2], process_id[3],
     timestamp[0], timestamp[1], timestamp[2], timestamp[3],
     random[0], random[1], random[2], random[3]]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_guid_prefix() {
        let process_id = process::id().to_be_bytes();

        let guid1 = generate_guid_prefix();
        let guid2 = generate_guid_prefix();
        let guid3 = generate_guid_prefix();

        // Check the process ID bytes for one of the GUIDs
        assert_eq!(process_id, guid1[0..=3]);

        // Check that the process ID bytes are the same for all GUIDs
        assert_eq!(guid1[0..=3], guid2[0..=3]);
        assert_eq!(guid1[0..=3], guid3[0..=3]);

        // Check that the random number is different for all GUIDs
        assert_ne!(guid1[8..=11], guid2[8..=11]);
        assert_ne!(guid1[8..=11], guid3[8..=11]);
    }
}