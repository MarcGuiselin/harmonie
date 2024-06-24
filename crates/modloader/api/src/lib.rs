use bitcode::{Decode, Encode};

#[derive(Encode, Decode, PartialEq, Debug)]
pub enum Command {
    Init,
    RunSystem(u32),
}

pub type CommandResult = Result<Response, Error>;

#[derive(Encode, Decode, PartialEq, Debug)]
pub enum Response {
    Success,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub enum Error {
    DecodeCommand,
}
