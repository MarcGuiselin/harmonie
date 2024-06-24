use bincode::serde::{borrow_decode_from_slice, encode_to_vec};
use serde::{Deserialize, Serialize};

pub fn decode<'de, D: Deserialize<'de>>(slice: &'de [u8]) -> D {
    let config = bincode::config::standard();
    borrow_decode_from_slice(slice, config)
        .map(|(deserialized, _)| deserialized)
        .unwrap()
}

pub fn encode<S: Serialize>(data: &S) -> Vec<u8> {
    let config = bincode::config::standard();
    encode_to_vec(data, config).unwrap()
}
