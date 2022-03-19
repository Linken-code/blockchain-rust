use bincode::{self, ErrorKind};
use crypto::digest::Digest;
use crypto::sha3::Sha3;
use ring::digest::{Context, SHA256};
use serde::{Deserialize, Serialize};

/// 区块序列化
pub fn serialized<T: ?Sized>(value: &T) -> Result<Vec<u8>, Box<ErrorKind>>
where
    T: Serialize,
{
    let serialized = bincode::serialize(value)?;
    Ok(serialized)
}

/// 从字节数组反序列化
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

/// 计算 sha256 哈希值
pub fn sha256_digest(data: &[u8]) -> Vec<u8> {
    let mut context = Context::new(&SHA256);
    context.update(data);
    let digest = context.finish();
    digest.as_ref().to_vec()
}
