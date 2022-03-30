use crate::pow::ProofOfWork;
use crate::transaction::Transaction;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use sled::IVec;
use utils::coder;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockHeader {
    timestamp: i64,   // 区块时间戳
    tx_hash: String,  //交易数据hash
    pre_hash: String, // 上一区块的哈希值
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    header: BlockHeader,            //区块头部
    hash: String,                   // 当前区块的哈希值
    transactions: Vec<Transaction>, // 交易数据
    nonce: i64,                     // 计数器
    height: usize,                  // 区块链中节点的高度
}

impl Block {
    // 新建一个区块
    pub fn new_block(transactions: &[Transaction], pre_hash: String, height: usize) -> Block {
        let timestamp = Utc::now().timestamp();
        let tx_ser = coder::serialized(transactions);
        let tx_hash = coder::get_hash(&tx_ser);
        let mut block = Block {
            header: BlockHeader {
                timestamp,
                pre_hash,
                tx_hash,
            },
            hash: String::new(),
            transactions: transactions.to_vec(),
            nonce: 0,
            height,
        };
        // 挖矿计算哈希
        let mut pow = ProofOfWork::new_proof_of_work(block.clone());
        let (nonce, hash) = pow.run();
        block.nonce = nonce;
        block.hash = hash;
        block
    }

    //获取区块的hash
    pub fn get_hash(&self) -> &str {
        self.hash.as_str()
    }

    // 计算区块里所有交易的哈希
    pub fn hash_transactions(&mut self) -> String {
        let tx_ser = coder::serialized(&self.transactions);
        self.header.tx_hash = coder::get_hash(&tx_ser);
        self.header.tx_hash.clone()
    }

    //获取上一个区块的hash
    pub fn get_pre_block_hash(&self) -> String {
        self.header.pre_hash.clone()
    }

    //获取区块时间戳
    pub fn get_timestamp(&self) -> i64 {
        self.header.timestamp
    }

    //获取交易数据
    pub fn get_transactions(&self) -> &[Transaction] {
        self.transactions.as_slice()
    }

    //获取区块高度
    pub fn get_height(&self) -> usize {
        self.height
    }

    //将hash转为字节数组
    pub fn get_hash_bytes(&self) -> Vec<u8> {
        self.hash.as_bytes().to_vec()
    }
}

impl From<Block> for IVec {
    fn from(b: Block) -> Self {
        let bytes = coder::serialized(&b);
        Self::from(bytes)
    }
}
