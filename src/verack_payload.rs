use binrw::binrw;

use crate::{command::Command, message_preparable::MessagePreparable};

#[derive(Debug)]
#[binrw]
#[brw(little)]
pub struct VerackPayload;

impl MessagePreparable for VerackPayload {
    const COMMAND_TYPE: Command = Command::Verack;
}
