pub mod codec;
pub use codec::Slice;

pub extern crate bitcode;
pub use bitcode::{decode, encode, Decode, Encode};
