use binrw::binrw;
use sha2::{Digest, Sha256};

use crate::{
    command::{Command, CommandError},
    utils::double_sha256_hash,
};

#[derive(Debug)]
#[binrw]
#[brw(magic = b"\xF9\xBE\xB4\xD9")]
#[brw(little)]
pub struct Header {
    command: [u8; 12],
    length: u32,
    checksum: u32,
}

impl Header {
    pub const HEADER_BYTE_SIZE: usize = 4 + 12 + 4 + 4;

    pub fn create(command: Command, payload: &[u8]) -> Self {
        let checksum = double_sha256_hash(payload);
        let checksum = u32::from_le_bytes([checksum[0], checksum[1], checksum[2], checksum[3]]);

        Self {
            command: command.into(),
            length: payload.len() as u32, // FIXME: Should I handle payloads greater than 4 GiB?
            checksum,
        }
    }

    pub fn command_type(&self) -> Result<Command, CommandError> {
        self.command.try_into()
    }

    pub fn payload_size(&self) -> u32 {
        self.length
    }

    pub fn validate_checksum(&self, payload: &[u8]) -> Result<(), ChecksumError> {
        if payload.len() < self.length as usize {
            return Err(ChecksumError::InsufficientPayload(
                payload.len(),
                self.length,
            ));
        }

        let mut hasher = Sha256::new();
        hasher.update(&payload[..(self.length as usize)]);
        let hash = hasher.finalize();

        let mut hasher = Sha256::new();
        hasher.update(hash);
        let hash = hasher.finalize();

        let hash = u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]]);
        if self.checksum == hash {
            Ok(())
        } else {
            Err(ChecksumError::IncorrectChecksum)
        }
    }
}

#[derive(Debug)]
pub enum ChecksumError {
    InsufficientPayload(usize, u32),
    IncorrectChecksum,
}

impl std::fmt::Display for ChecksumError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ChecksumError::InsufficientPayload(actual_length, expected_length) => {
                write!(
                    f,
                    "expected {expected_length} but received only {actual_length} byte(s) for payload",
                )
            }
            ChecksumError::IncorrectChecksum => write!(f, "incorrect checksum for payload"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use binrw::{BinRead, BinWrite};

    use super::*;

    #[test]
    fn test_header_byte_size() {
        let raw_binary = hex::decode("F9BEB4D976657273696F6E000000000064000000358d4932").unwrap();

        let version_header = Header::read(&mut Cursor::new(&raw_binary)).unwrap();

        let mut encoded = Cursor::new(Vec::new());
        version_header.write(&mut encoded).unwrap();

        assert_eq!(encoded.into_inner().len(), Header::HEADER_BYTE_SIZE);
    }

    #[test]
    fn test_serialize_deserialize_header_version() {
        let raw_binary = hex::decode("F9BEB4D976657273696F6E000000000064000000358d4932").unwrap();

        let version_header = Header::read(&mut Cursor::new(&raw_binary)).unwrap();

        let mut encoded = Cursor::new(Vec::new());
        version_header.write(&mut encoded).unwrap();

        assert_eq!(encoded.into_inner(), raw_binary);
    }

    #[test]
    fn test_serialize_deserialize_header_verack() {
        let raw_binary = hex::decode("F9BEB4D976657261636B000000000000000000005DF6E0E2").unwrap();

        let verack_header = Header::read(&mut Cursor::new(&raw_binary)).unwrap();

        assert!(matches!(verack_header.validate_checksum(&[]), Ok(())));

        let mut encoded = Cursor::new(Vec::new());
        verack_header.write(&mut encoded).unwrap();

        assert_eq!(encoded.into_inner(), raw_binary);
    }
}
