use std::io::Cursor;

use binrw::{meta::WriteEndian, BinRead, BinWrite};

use crate::{
    command::Command,
    header::{ChecksumError, Header},
    message_preparable::MessagePreparable,
    version_payload::VersionPayload,
};

#[derive(Debug)]
pub enum MessageType {
    Verack,
    Version(VersionPayload),
}

pub fn prepare_message<P>(payload: P) -> Result<Vec<u8>, binrw::error::Error>
where
    P: MessagePreparable,
    P: BinWrite + WriteEndian,
    for<'a> <P as BinWrite>::Args<'a>: Default,
{
    let buf = vec![0u8; Header::HEADER_BYTE_SIZE];
    let mut cursor = Cursor::new(buf);
    cursor.set_position(Header::HEADER_BYTE_SIZE as u64);

    payload.write(&mut cursor)?;

    let buf = cursor.into_inner();
    let header = Header::create(P::COMMAND_TYPE, &buf[Header::HEADER_BYTE_SIZE..]);

    let mut cursor = Cursor::new(buf);
    header.write(&mut cursor)?;
    assert_eq!(cursor.position(), Header::HEADER_BYTE_SIZE as u64);

    Ok(cursor.into_inner())
}

pub fn parse_message(data: &[u8]) -> Result<(MessageType, usize), MessageParseError> {
    if data.len() < Header::HEADER_BYTE_SIZE {
        return Err(MessageParseError::NotEnoughData);
    }

    let mut cursor = Cursor::new(data);

    // Read the header first
    let header = Header::read(&mut cursor)?;

    // Ensure that the payload checksum is valid before even trying to parse the payload
    header.validate_checksum(&data[(cursor.position() as usize)..])?;

    // Introspect on the header type to determine which parsing should be applied
    let message = match header.command_type() {
        Ok(Command::Verack) => MessageType::Verack,
        Ok(Command::Version) => {
            let version_payload = VersionPayload::read(&mut cursor)?;
            MessageType::Version(version_payload)
        }
        Err(_) => return Err(MessageParseError::UnknownMessageType(header.payload_size())),
    };
    let bytes_read = cursor.position() as usize;
    Ok((message, bytes_read))
}

#[derive(Debug)]
pub enum MessageParseError {
    NotEnoughData,
    MissingMagicNumber,
    IncorrectChecksum,
    MalformedData,
    UnknownMessageType(u32),
}

impl std::fmt::Display for MessageParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::NotEnoughData => write!(f, "not enough data"),
            Self::MissingMagicNumber => write!(f, "missing magic number"),
            Self::IncorrectChecksum => write!(f, "incorrect payload checksum"),
            Self::MalformedData => write!(f, "malformed data"),
            Self::UnknownMessageType(_) => write!(f, "unknown or unimplemented message type"),
        }
    }
}

impl std::error::Error for MessageParseError {}

impl From<binrw::Error> for MessageParseError {
    fn from(e: binrw::Error) -> Self {
        match e {
            binrw::Error::BadMagic { .. } => Self::MissingMagicNumber,
            _ => Self::MalformedData,
        }
    }
}

impl From<ChecksumError> for MessageParseError {
    fn from(e: ChecksumError) -> Self {
        match e {
            ChecksumError::InsufficientPayload(_, _) => Self::NotEnoughData,
            ChecksumError::IncorrectChecksum => Self::IncorrectChecksum,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        net::IpAddr,
        time::{Duration, SystemTime},
    };

    use crate::verack_payload::VerackPayload;

    use super::*;

    #[test]
    fn test_prepare_verack_message() {
        let verack_payload = VerackPayload;

        let verack_message = prepare_message(verack_payload).unwrap();
        assert_eq!(
            verack_message,
            hex::decode("F9BEB4D976657261636B000000000000000000005DF6E0E2").unwrap(),
        );
    }

    #[test]
    fn test_prepare_version_message() {
        let timestamp = SystemTime::UNIX_EPOCH + Duration::from_secs(1640961477);
        let version_payload =
            VersionPayload::create(timestamp, "46.19.137.74".parse::<IpAddr>().unwrap(), 8333);

        let version_message = prepare_message(version_payload).unwrap();
        assert_eq!(
            version_message,
            hex::decode("F9BEB4D976657273696F6E0000000000550000002C2F86F37E1101000000000000000000C515CF6100000000000000000000000000000000000000000000FFFF2E13894A208D000000000000000000000000000000000000FFFF7F000001208D00000000000000000000000000").unwrap(),
        );
    }

    #[test]
    fn test_parse_verack_message() {
        let raw_binary = hex::decode("F9BEB4D976657261636B000000000000000000005DF6E0E2").unwrap();

        let (message, bytes_read) = parse_message(&raw_binary).unwrap();
        assert!(matches!(message, MessageType::Verack));
        assert_eq!(raw_binary.len(), bytes_read);
    }

    #[test]
    fn test_parse_version_message() {
        let raw_binary = hex::decode("F9BEB4D976657273696F6E0000000000550000002C2F86F37E1101000000000000000000C515CF6100000000000000000000000000000000000000000000FFFF2E13894A208D000000000000000000000000000000000000FFFF7F000001208D00000000000000000000000000").unwrap();

        let (message, bytes_read) = parse_message(&raw_binary).unwrap();
        assert!(matches!(message, MessageType::Version(_)));
        assert_eq!(raw_binary.len(), bytes_read);
    }
}
