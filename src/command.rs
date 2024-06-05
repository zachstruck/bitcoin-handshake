const VERACK_COMMAND: [u8; 12] = *b"verack\0\0\0\0\0\0";
const VERSION_COMMAND: [u8; 12] = *b"version\0\0\0\0\0";

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Verack,
    Version,
}

impl TryFrom<[u8; 12]> for Command {
    type Error = CommandError;

    fn try_from(value: [u8; 12]) -> Result<Self, Self::Error> {
        let command = match value {
            VERACK_COMMAND => Self::Verack,
            VERSION_COMMAND => Self::Version,
            _ => return Err(Self::Error::UnknownCommand),
        };
        Ok(command)
    }
}

impl From<Command> for [u8; 12] {
    fn from(value: Command) -> Self {
        match value {
            Command::Verack => VERACK_COMMAND,
            Command::Version => VERSION_COMMAND,
        }
    }
}

#[derive(Debug)]
pub enum CommandError {
    UnknownCommand,
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::UnknownCommand => write!(f, "unknown command"),
        }
    }
}

impl std::error::Error for CommandError {}
