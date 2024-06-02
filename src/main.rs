use std::{
    io::{Read, Write},
    net::{IpAddr, TcpStream},
    time::SystemTime,
};

use message::{parse_message, MessageParseError};
use verack::VerackPayload;
use verack_message::prepare_verack_message;
use version::VersionPayload;
use version_message::prepare_version_message;

mod command;
mod header;
mod message;
mod utils;
mod verack;
mod verack_message;
mod version;
mod version_message;

fn main() {
    let mut stream = TcpStream::connect("65.109.34.157:8333").unwrap();
    println!("connected");

    // Send my version message
    {
        let version_message = prepare_version_message(&VersionPayload::create(
            SystemTime::now(),
            "65.109.34.157".parse::<IpAddr>().unwrap(),
            8333,
        ))
        .expect("should be a valid version message");
        stream
            .write(&version_message)
            .expect("should be able to send version message");
    }

    let mut data = Vec::<u8>::new();

    // Wait to receive the remote version message (presumably)
    {
        'receiving: loop {
            match parse_message(&data) {
                Ok((message, bytes_read)) => {
                    println!("Received message: {message:?}");
                    data = data.split_off(bytes_read);
                    break 'receiving;
                }
                Err(MessageParseError::UnknownMessageType(bytes_read)) => {
                    let bytes_read = bytes_read as usize;
                    println!("Unknown message: {:?}", &data[..bytes_read]);
                    data = data.split_off(bytes_read);
                    break 'receiving;
                }
                Err(MessageParseError::NotEnoughData) => {
                    let mut buf = [0; 4096];
                    let bytes_read = stream
                        .read(&mut buf)
                        .expect("should be able to read receive data");
                    data.extend(&buf[..bytes_read]);
                    continue 'receiving;
                }
                Err(e @ MessageParseError::MissingMagicNumber)
                | Err(e @ MessageParseError::IncorrectChecksum)
                | Err(e @ MessageParseError::MalformedData) => panic!("{e:?}"),
            };
        }
    }

    // Wait to receive the remote verack message (presumably)
    {
        'receiving: loop {
            match parse_message(&data) {
                Ok((message, bytes_read)) => {
                    println!("Received message: {message:?}");
                    data = data.split_off(bytes_read);
                    break 'receiving;
                }
                Err(MessageParseError::UnknownMessageType(bytes_read)) => {
                    let bytes_read = bytes_read as usize;
                    println!("Unknown message: {:?}", &data[..bytes_read]);
                    data = data.split_off(bytes_read);
                    break 'receiving;
                }
                Err(MessageParseError::NotEnoughData) => {
                    let mut buf = [0; 4096];
                    let bytes_read = stream
                        .read(&mut buf)
                        .expect("should be able to read receive data");
                    data.extend(&buf[..bytes_read]);
                    continue 'receiving;
                }
                Err(e @ MessageParseError::MissingMagicNumber)
                | Err(e @ MessageParseError::IncorrectChecksum)
                | Err(e @ MessageParseError::MalformedData) => panic!("{e:?}"),
            };
        }
    }

    // Send my verack message
    {
        let version_message =
            prepare_verack_message(&VerackPayload).expect("should be a valid verack message");
        stream
            .write(&version_message)
            .expect("should be able to send version message");
    }
}
