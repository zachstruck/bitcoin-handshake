use std::{
    net::{IpAddr, SocketAddr},
    time::SystemTime,
};

use clap::Parser;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use command::Command;
use message::{parse_message, prepare_message, MessageParseError, MessageType};
use verack_payload::VerackPayload;
use version_payload::VersionPayload;

mod command;
mod header;
mod message;
mod message_preparable;
mod utils;
mod verack_payload;
mod version_payload;

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long)]
    ip_address: IpAddr,
    #[arg(short, long, default_value_t = 8333)]
    port: u16,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let mut messaging_system =
        MessagingSystem::try_new(SocketAddr::new(args.ip_address, args.port))
            .await
            .expect("IP address and port should point to an available node");

    // Send my version message
    messaging_system
        .send_message(Command::Version)
        .await
        .expect("should be able to send version message");

    // Receive the version message
    let message = messaging_system
        .receive_message()
        .await
        .expect("should be able to receive message");
    match message {
        MessageType::Verack => panic!("unexpectedly received verack message"),
        MessageType::Version(_) => {}
    };

    // Receive the verack message
    let message = messaging_system
        .receive_message()
        .await
        .expect("should be able to receive message");
    match message {
        MessageType::Verack => {}
        MessageType::Version(_) => panic!("unexpectedly received version message"),
    };

    // Send the verack message
    messaging_system
        .send_message(Command::Verack)
        .await
        .expect("should be able to send verack message");

    println!("successful handshake");
}

pub struct MessagingSystem {
    stream: tokio::net::TcpStream,
    data: Vec<u8>,
    buf: [u8; 4096],
    socket_address: SocketAddr,
}

impl MessagingSystem {
    pub async fn try_new(socket_address: SocketAddr) -> std::io::Result<Self> {
        let stream = TcpStream::connect(&socket_address).await?;

        Ok(Self {
            stream,
            data: Vec::new(),
            buf: [0; 4096],
            socket_address,
        })
    }

    pub async fn send_message(&mut self, command: Command) -> Result<(), MessageSendError> {
        let message_packet = match command {
            Command::Verack => prepare_message(VerackPayload)?,
            Command::Version => prepare_message(VersionPayload::create(
                SystemTime::now(),
                self.socket_address.ip(),
                self.socket_address.port(),
            ))?,
        };

        Ok(self.stream.write_all(&message_packet).await?)
    }

    pub async fn receive_message(&mut self) -> Result<MessageType, MessageReceiveError> {
        'receiving: loop {
            match parse_message(&self.data) {
                Ok((message, bytes_read)) => {
                    self.data = self.data.split_off(bytes_read);
                    return Ok(message);
                }
                Err(MessageParseError::UnknownMessageType(bytes_read)) => {
                    let bytes_read = bytes_read as usize;
                    self.data = self.data.split_off(bytes_read);
                    return Err(MessageReceiveError::UnknownMessage);
                }
                Err(MessageParseError::NotEnoughData) => {
                    let bytes_read = self.stream.read(&mut self.buf).await?;
                    self.data.extend(&self.buf[..bytes_read]);
                    continue 'receiving;
                }
                Err(e @ MessageParseError::MissingMagicNumber)
                | Err(e @ MessageParseError::IncorrectChecksum)
                | Err(e @ MessageParseError::MalformedData) => return Err(e.into()),
            };
        }
    }
}

#[derive(Debug)]
pub enum MessageSendError {
    Creation(binrw::Error),
    Io(std::io::Error),
}

impl std::fmt::Display for MessageSendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Creation(e) => e.fmt(f),
            Self::Io(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for MessageSendError {}

impl From<binrw::Error> for MessageSendError {
    fn from(value: binrw::Error) -> Self {
        Self::Creation(value)
    }
}

impl From<std::io::Error> for MessageSendError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug)]
pub enum MessageReceiveError {
    Parsing(MessageParseError),
    UnknownMessage,
    Io(std::io::Error),
}

impl std::fmt::Display for MessageReceiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parsing(e) => e.fmt(f),
            Self::UnknownMessage => write!(f, "unknown message"),
            Self::Io(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for MessageReceiveError {}

impl From<MessageParseError> for MessageReceiveError {
    fn from(value: MessageParseError) -> Self {
        Self::Parsing(value)
    }
}

impl From<std::io::Error> for MessageReceiveError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
