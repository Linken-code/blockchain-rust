use bincode::{self, ErrorKind};
use crypto::digest::Digest;
use crypto::sha3::Sha3;
use serde::{Deserialize, Serialize};

pub fn serialized<T: ?Sized>(value: &T) -> Result<Vec<u8>, Box<ErrorKind>>
where
    T: Serialize,
{
    let serialized = bincode::serialize(value)?;
    Ok(serialized)
}

pub fn deserialized<'a, T>(bytes: &'a [u8]) -> Result<T, Box<ErrorKind>>
where
    T: Deserialize<'a>,
{
    let deserialized = bincode::deserialize(bytes)?;
    Ok(deserialized)
}

pub fn get_hash(value: &[u8]) -> String {
    let mut hash = Sha3::sha3_256();
    hash.input(value);
    hash.result_str()
}
