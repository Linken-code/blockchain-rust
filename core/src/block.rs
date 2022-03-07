use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use utils::coder;

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockHeader {
    pub time: i64,
    pub tx_hash: String,
    pub pre_hash: String,
}

#[derive(Debug)]
pub struct Block {
    pub header: BlockHeader,
    pub hash: String,
    pub data: String,
}

impl Block {
    fn set_hash(&mut self) {
        self.header.time = Utc::now().timestamp();
        let header = coder::serialized(&(self.header)).unwrap();
        self.hash = coder::get_hash(&header[..]);
    }

    pub fn new_block(data: String, pre_hash: String) -> Result<Block, Box<dyn std::error::Error>> {
        let transactions = coder::serialized(&data)?;
        let tx_hash: String = coder::get_hash(&transactions);
        let time = Utc::now().timestamp();
        let mut block = Block {
            header: BlockHeader {
                time,
                tx_hash,
                pre_hash,
            },
            hash: "".to_string(),
            data: data,
        };
        block.set_hash();
        Ok(block)
    }
}
