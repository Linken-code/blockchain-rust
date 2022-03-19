use crate::proof_of_work::ProofOfWork;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use utils::coder;
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockHeader {
    timestamp: i64,   // 区块时间戳
    tx_hash: String,  //交易数据hash
    pre_hash: String, // 上一区块的哈希值
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    header: BlockHeader,  //区块头部
    hash: String,         // 当前区块的哈希值
    transactions: String, // 交易数据
    nonce: i64,           // 计数器
                          // height: usize,        // 区块链中节点的高度
}

impl Block {
    // 新建一个区块
    pub fn new_block(data: String, pre_hash: String) -> Result<Block, Box<dyn std::error::Error>> {
        let transactions = coder::serialized(&data)?;
        let tx_hash: String = coder::get_hash(&transactions);
        let timestamp = Utc::now().timestamp();
        let mut block = Block {
            header: BlockHeader {
                timestamp,
                tx_hash,
                pre_hash,
            },
            hash: String::new(),
            transactions: data,
            nonce: 0,
        };
        // 挖矿计算哈希
        let pow = ProofOfWork::new_proof_of_work(block.clone());
        let (nonce, hash) = pow.run();
        block.nonce = nonce;
        block.hash = hash;
        Ok(block)
    }
    //获取区块的hash
    pub fn get_hash(&self) -> String {
        self.hash.clone()
    }

    //获取上一个区块的hash
    pub fn get_pre_block_hash(&self) -> String {
        self.header.pre_hash.clone()
    }

    //获取区块时间戳
    pub fn get_timestamp(&self) -> i64 {
        self.header.timestamp
    }

    //获取区块高度
    // pub fn get_height(&self) -> usize {
    //     self.height
    // }
}
