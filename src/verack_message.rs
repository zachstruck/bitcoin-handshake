use std::io::Cursor;

use binrw::BinWrite;

use crate::{command::Command, header::Header, verack_payload::VerackPayload};

pub fn prepare_verack_message(payload: &VerackPayload) -> Result<Vec<u8>, binrw::error::Error> {
    let buf = vec![0u8; Header::HEADER_BYTE_SIZE];
    let mut cursor = Cursor::new(buf);
    cursor.set_position(Header::HEADER_BYTE_SIZE as u64);

    payload.write(&mut cursor)?;

    let buf = cursor.into_inner();
    let header = Header::create(Command::Verack, &buf[Header::HEADER_BYTE_SIZE..]);

    let mut cursor = Cursor::new(buf);
    header.write(&mut cursor)?;
    assert_eq!(cursor.position(), Header::HEADER_BYTE_SIZE as u64);

    Ok(cursor.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prepare_verack_message() {
        let verack_payload = VerackPayload;

        let verack_message = prepare_verack_message(&verack_payload).unwrap();
        assert_eq!(
            verack_message,
            hex::decode("F9BEB4D976657261636B000000000000000000005DF6E0E2").unwrap(),
        );
    }
}
