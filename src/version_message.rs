use std::io::Cursor;

use binrw::BinWrite;

use crate::{command::Command, header::Header, version_payload::VersionPayload};

pub fn prepare_version_message(payload: &VersionPayload) -> Result<Vec<u8>, binrw::error::Error> {
    let buf = vec![0u8; Header::HEADER_BYTE_SIZE];
    let mut cursor = Cursor::new(buf);
    cursor.set_position(Header::HEADER_BYTE_SIZE as u64);

    payload.write(&mut cursor)?;

    let buf = cursor.into_inner();
    let header = Header::create(Command::Version, &buf[Header::HEADER_BYTE_SIZE..]);

    let mut cursor = Cursor::new(buf);
    header.write(&mut cursor)?;
    assert_eq!(cursor.position(), Header::HEADER_BYTE_SIZE as u64);

    Ok(cursor.into_inner())
}

#[cfg(test)]
mod tests {
    use std::{
        net::IpAddr,
        time::{Duration, SystemTime},
    };

    use super::*;

    #[test]
    fn test_prepare_version_message() {
        let timestamp = SystemTime::UNIX_EPOCH + Duration::from_secs(1640961477);
        let version_payload =
            VersionPayload::create(timestamp, "46.19.137.74".parse::<IpAddr>().unwrap(), 8333);

        let version_message = prepare_version_message(&version_payload).unwrap();
        assert_eq!(
            version_message,
            hex::decode("F9BEB4D976657273696F6E0000000000550000002C2F86F37E1101000000000000000000C515CF6100000000000000000000000000000000000000000000FFFF2E13894A208D000000000000000000000000000000000000FFFF7F000001208D00000000000000000000000000").unwrap(),
        );
    }
}
