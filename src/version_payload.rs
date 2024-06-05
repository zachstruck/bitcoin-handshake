use std::{
    io::SeekFrom,
    net::{IpAddr, Ipv4Addr},
    time::{SystemTime, UNIX_EPOCH},
};

use binrw::{binrw, BinRead, BinResult, BinWrite};

use crate::{command::Command, message_preparable::MessagePreparable};

#[derive(Debug)]
#[binrw]
#[brw(little)]
struct NetworkAddress {
    services: u64,
    #[brw(big)]
    #[br(parse_with = read_ip_addr)]
    #[bw(write_with = write_ip_addr)]
    ip_address: IpAddr,
    #[brw(big)]
    port: u16,
}

#[binrw::parser(reader, endian)]
fn read_ip_addr() -> BinResult<IpAddr> {
    Ok(IpAddr::from(<[u8; 16]>::read_options(reader, endian, ())?))
}

#[binrw::writer(writer, endian)]
fn write_ip_addr(ip_address: &IpAddr) -> BinResult<()> {
    let ip_address = match ip_address {
        IpAddr::V4(ip) => ip.to_ipv6_mapped().octets(),
        IpAddr::V6(ip) => ip.octets(),
    };
    ip_address.write_options(writer, endian, ())
}

#[derive(Debug)]
#[binrw]
#[brw(little)]
pub struct VersionPayload {
    version: i32,
    services: u64,
    pub timestamp: i64,
    addr_recv: NetworkAddress,
    addr_from: NetworkAddress,
    nonce: u64,
    #[br(parse_with = read_string)]
    #[bw(write_with = write_string)]
    user_agent: Vec<u8>,
    last_block: i32,
    #[br(parse_with = read_optional_bool)]
    #[bw(write_with = write_optional_bool)]
    relay: Option<bool>,
}

#[binrw::parser(reader, endian)]
fn read_string() -> BinResult<Vec<u8>> {
    let b = u8::read_options(reader, endian, ())?;
    let len = match b {
        len @ 0..=0xFC => len as u64,
        0xFD => u16::read_options(reader, endian, ())? as u64,
        0xFE => u32::read_options(reader, endian, ())? as u64,
        0xFF => u64::read_options(reader, endian, ())? as u64,
    };

    let mut s = Vec::with_capacity(len as usize);

    for _ in 0..len {
        // How to read an array of data?
        s.push(u8::read_options(reader, endian, ())?);
    }

    Ok(s)
}

#[binrw::writer(writer, endian)]
fn write_string(s: &Vec<u8>) -> BinResult<()> {
    let len = s.len() as u64;
    match len {
        0..=0xFC => {
            (len as u8).write_options(writer, endian, ())?;
        }
        0xFD..=0xFFFF => {
            0xFDu8.write_options(writer, endian, ())?;
            (len as u16).write_options(writer, endian, ())?;
        }
        0x1_0000..=0xFFFF_FFFF => {
            0xFEu8.write_options(writer, endian, ())?;
            (len as u32).write_options(writer, endian, ())?;
        }
        0x1_0000_0000..=0xFFFF_FFFF_FFFF_FFFF => {
            0xFFu8.write_options(writer, endian, ())?;
            (len as u64).write_options(writer, endian, ())?;
        }
    };

    s.write_options(writer, endian, ())
}

#[binrw::parser(reader, endian)]
fn read_optional_bool() -> BinResult<Option<bool>> {
    let b = match u8::read_options(reader, endian, ()) {
        Ok(1) => Some(true),
        Ok(0) => Some(false),
        Ok(_) => {
            // Move back one byte
            reader.seek(SeekFrom::Current(-1))?;
            None
        }
        Err(binrw::error::Error::Io(io_error)) => match io_error.kind() {
            std::io::ErrorKind::UnexpectedEof => {
                // We've run out of data in the stream,
                // so just assume the optional value was not set.
                None
            }
            _ => return Err(binrw::error::Error::Io(io_error)),
        },
        Err(e @ _) => return Err(e),
    };
    Ok(b)
}

#[binrw::writer(writer, endian)]
fn write_optional_bool(s: &Option<bool>) -> BinResult<()> {
    match &s {
        Some(true) => 1u8.write_options(writer, endian, ()),
        Some(false) => 0u8.write_options(writer, endian, ()),
        None => Ok(()),
    }
}

impl VersionPayload {
    pub fn create(timestamp: SystemTime, remote_ip_address: IpAddr, remote_port: u16) -> Self {
        Self {
            version: 70014,
            services: 0,
            timestamp: timestamp.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
            addr_recv: NetworkAddress {
                services: 0,
                ip_address: remote_ip_address,
                port: remote_port,
            },
            addr_from: NetworkAddress {
                services: 0,
                ip_address: IpAddr::V4(Ipv4Addr::LOCALHOST),
                port: 8333,
            },
            nonce: 0,
            user_agent: Vec::new(),
            last_block: 0,
            relay: None,
        }
    }
}

impl MessagePreparable for VersionPayload {
    const COMMAND_TYPE: Command = Command::Version;
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_serialize_deserialize_network_address() {
        let raw_binary =
            hex::decode("010000000000000000000000000000000000FFFF0A000001208D").unwrap();

        let network_address = NetworkAddress::read(&mut Cursor::new(&raw_binary)).unwrap();

        let mut encoded = Cursor::new(Vec::new());
        network_address.write(&mut encoded).unwrap();

        assert_eq!(encoded.into_inner(), raw_binary);
    }

    #[test]
    fn test_serialize_deserialize_version_payload() {
        let raw_binary = hex::decode("7E1101000000000000000000C515CF6100000000000000000000000000000000000000000000FFFF2E13894A208D000000000000000000000000000000000000FFFF7F000001208D00000000000000000000000000").unwrap();

        let version_payload = VersionPayload::read(&mut Cursor::new(&raw_binary)).unwrap();

        let mut encoded = Cursor::new(Vec::new());
        version_payload.write(&mut encoded).unwrap();

        assert_eq!(encoded.into_inner(), raw_binary);
    }

    #[test]
    fn test_serialize_deserialize_version_payload_2() {
        let raw_binary = hex::decode("62EA0000010000000000000011B2D05000000000010000000000000000000000000000000000FFFF000000000000010000000000000000000000000000000000FFFF0000000000003B2EB35D8CE617650F2F5361746F7368693A302E372E322FC03E0300").unwrap();

        let version_payload = VersionPayload::read(&mut Cursor::new(&raw_binary)).unwrap();

        assert_eq!(
            &String::from_utf8(version_payload.user_agent.clone()).unwrap(),
            "/Satoshi:0.7.2/",
        );
        assert_eq!(version_payload.last_block, 212672);

        let mut encoded = Cursor::new(Vec::new());
        version_payload.write(&mut encoded).unwrap();

        assert_eq!(encoded.into_inner(), raw_binary);
    }
}
