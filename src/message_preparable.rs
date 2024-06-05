use crate::command::Command;

pub trait MessagePreparable {
    const COMMAND_TYPE: Command;
}
