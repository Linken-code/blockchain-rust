use crate::block::Block;
use crate::transaction::{TXOutput, Transaction};
use data_encoding::HEXLOWER;
use dotenv::dotenv;
use sled::transaction::TransactionResult;
use sled::{Db, Tree};
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, RwLock};
use utils::coder;

const TIP_BLOCK_HASH_KEY: &str = "tip_block_hash";
const BLOCKS_TREE: &str = "blocks";

#[derive(Clone, Debug)]
pub struct BlockChain {
    tip_hash: Arc<RwLock<String>>, // hash of last block
    db: Db,
}

impl BlockChain {
    // 生成创世块
    fn new_genesis_block(transaction: &Transaction) -> Block {
        let transaction = vec![transaction.clone()];
        let block = Block::new_block(&transaction, String::from("None"), 0);
        block
    }

    // 创建新的区块链
    pub fn create_blockchain(genesis_address: &str) -> BlockChain {
        dotenv().ok();
        let key = "DBName";
        let name = env::var(key).expect("获取环境变量失败");
        let db = sled::open(name).expect("数据库不存在");
        let blocks_tree = db.open_tree(BLOCKS_TREE).expect("block库不存在");
        let last_hash = blocks_tree
            .get(TIP_BLOCK_HASH_KEY)
            .expect("获取最后一个块的哈希失败");

        let tip_hash;
        if last_hash.is_none() {
            let coinbase_tx = Transaction::new_coinbase_tx(genesis_address); //新建coinbase交易
            let block = self::BlockChain::new_genesis_block(&coinbase_tx); //创世块
            self::BlockChain::update_blocks_tree(&blocks_tree, &block); //写入数据库
            tip_hash = String::from(block.get_hash());
        } else {
            tip_hash = String::from_utf8(last_hash.unwrap().to_vec()).unwrap();
        }

        BlockChain {
            tip_hash: Arc::new(RwLock::new(tip_hash)),
            db,
        }
    }

    /// 创建区块链实例
    pub fn new_blockchain() -> BlockChain {
        dotenv().ok();
        let key = "DBName";
        let name = env::var(key).expect("获取环境变量失败");
        let db = sled::open(name).expect("数据库不存在");
        let blocks_tree = db.open_tree(BLOCKS_TREE).unwrap();
        let tip_bytes = blocks_tree
            .get(TIP_BLOCK_HASH_KEY)
            .unwrap()
            .expect("No existing blockchain found. Create one first.");
        let tip_hash = String::from_utf8(tip_bytes.to_vec()).unwrap();
        BlockChain {
            tip_hash: Arc::new(RwLock::new(tip_hash)),
            db,
        }
    }

    // 添加一个区块到区块链
    pub fn add_block(&self, block: &Block) {
        let block_tree = self.db.open_tree(BLOCKS_TREE).unwrap();
        let _: TransactionResult<(), ()> = block_tree.transaction(|tx_db| {
            // 获取上一个区块字节数据
            let tip_block_bytes = tx_db
                .get(self.get_tip_hash())
                .unwrap()
                .expect("The tip hash is not valid");

            // 上一个区块字节数据转换为区块
            let tip_block: Block = coder::deserialized(tip_block_bytes.as_ref());

            // 对比区块高度
            if block.get_height() > tip_block.get_height() {
                //当前区块写入数据库
                tx_db
                    .insert(block.get_hash(), coder::serialized(&block))
                    .expect("插入新区块失败");

                //数据库插入当前区块hash
                tx_db
                    .insert(TIP_BLOCK_HASH_KEY, block.get_hash())
                    .expect("数据库插入tip_Hash失败");
                //替换链上的tip_hash
                self.set_tip_hash(block.get_hash());
            }
            Ok(())
        });
    }

    // 更新区块树
    fn update_blocks_tree(blocks_tree: &Tree, block: &Block) {
        let block_hash = block.get_hash(); //区块hash
        let _: TransactionResult<(), ()> = blocks_tree.transaction(|tx_db| {
            let _ = tx_db.insert(block_hash, block.clone()); //插入当前区块hash和区块数据
            let _ = tx_db.insert(TIP_BLOCK_HASH_KEY, block_hash); //将当前区块哈希值作为尾巴写入数据库
            Ok(())
        });
    }

    /// 挖矿新区块
    pub fn mine_block(&self, transactions: &[Transaction]) -> Block {
        for transaction in transactions {
            //验证签名
            if transaction.verify(self) == false {
                panic!("ERROR: Invalid transaction")
            }
        }
        let best_height = self.get_best_height();
        let block = Block::new_block(transactions, self.get_tip_hash(), best_height);
        let block_hash = block.get_hash();

        let blocks_tree = self.db.open_tree(BLOCKS_TREE).expect("无法找到区块树");
        Self::update_blocks_tree(&blocks_tree, &block);
        self.set_tip_hash(block_hash);
        block
    }

    /// 查找所有未花费的交易输出 ( K -> txid_hex, V -> Vec<TXOutput )
    pub fn find_utxo(&self) -> HashMap<String, Vec<TXOutput>> {
        let mut utxo: HashMap<String, Vec<TXOutput>> = HashMap::new();
        let mut spent_txos: HashMap<String, Vec<usize>> = HashMap::new();

        let mut iterator = self.iterator();
        loop {
            let option = iterator.next();
            if option.is_none() {
                break;
            }
            let block = option.unwrap();
            'outer: for tx in block.get_transactions() {
                let txid_hex = HEXLOWER.encode(tx.get_id());
                for (idx, out) in tx.get_vout().iter().enumerate() {
                    // 过滤已花费的输出
                    if let Some(outs) = spent_txos.get(txid_hex.as_str()) {
                        for spend_out_idx in outs {
                            if idx.eq(spend_out_idx) {
                                continue 'outer;
                            }
                        }
                    }
                    if utxo.contains_key(txid_hex.as_str()) {
                        utxo.get_mut(txid_hex.as_str()).unwrap().push(out.clone());
                    } else {
                        utxo.insert(txid_hex.clone(), vec![out.clone()]);
                    }
                }
                if tx.is_coinbase() {
                    continue;
                }
                // 在输入中查找已花费输出
                for tx_input in tx.get_vin() {
                    let txid_hex = HEXLOWER.encode(tx_input.get_txid());
                    if spent_txos.contains_key(txid_hex.as_str()) {
                        spent_txos
                            .get_mut(txid_hex.as_str())
                            .unwrap()
                            .push(tx_input.get_vout());
                    } else {
                        spent_txos.insert(txid_hex, vec![tx_input.get_vout()]);
                    }
                }
            }
        }
        utxo
    }

    /// 从区块链中查找交易
    pub fn find_transaction(&self, txid: &[u8]) -> Option<Transaction> {
        let mut iterator = self.iterator();
        loop {
            let option = iterator.next();
            if option.is_none() {
                break;
            }
            if let Some(block) = option {
                for transaction in block.get_transactions() {
                    if txid.eq(transaction.get_id()) {
                        return Some(transaction.clone());
                    }
                }
            }
        }
        None
    }

    //获取当前tip_hash
    pub fn get_tip_hash(&self) -> String {
        self.tip_hash.read().unwrap().clone()
    }

    //设置tip_hash
    pub fn set_tip_hash(&self, new_tip_hash: &str) {
        let mut tip_hash = self.tip_hash.write().unwrap();
        *tip_hash = String::from(new_tip_hash)
    }

    pub fn get_db(&self) -> &Db {
        &self.db
    }

    /// 获取最新区块在链中的高度
    pub fn get_best_height(&self) -> usize {
        let block_tree = self.db.open_tree(BLOCKS_TREE).expect("获取区块集失败");
        match block_tree.get(self.get_tip_hash()) {
            Ok(block_tree) => {
                let tip_block_bytes = block_tree.expect("The tip hash is valid");
                let tip_block: Block = coder::deserialized(tip_block_bytes.as_ref());
                tip_block.get_height()
            }
            _ => 0,
        }
    }

    /// 通过区块哈希查询区块
    pub fn get_block(&self, block_hash: &[u8]) -> Option<Block> {
        let block_tree = self.db.open_tree(BLOCKS_TREE).unwrap();
        if let Some(block_bytes) = block_tree.get(block_hash).unwrap() {
            let block = coder::deserialized(block_bytes.as_ref());
            return Some(block);
        }
        return None;
    }

    /// 返回链中所有区块的哈希列表
    pub fn get_block_hashes(&self) -> Vec<Vec<u8>> {
        let mut iterator = self.iterator();
        let mut blocks = vec![];
        loop {
            let option = iterator.next();
            if option.is_none() {
                break;
            }
            let block = option.unwrap();
            blocks.push(block.get_hash_bytes());
        }
        return blocks;
    }

    //区块链迭代器
    pub fn iterator(&self) -> BlockchainIterator {
        BlockchainIterator::new(self.get_tip_hash(), self.db.clone())
    }

    //打印区块链信息
    pub fn block_info(&self) {
        println!("{:#?}", self);
    }
}

pub struct BlockchainIterator {
    db: Db,
    current_hash: String,
}

impl BlockchainIterator {
    fn new(tip_hash: String, db: Db) -> BlockchainIterator {
        BlockchainIterator {
            current_hash: tip_hash,
            db,
        }
    }

    pub fn next(&mut self) -> Option<Block> {
        let block_tree = self.db.open_tree(BLOCKS_TREE).unwrap();
        let data = block_tree.get(self.current_hash.clone()).unwrap();
        if data.is_none() {
            return None;
        }
        let block: Block = coder::deserialized(data.unwrap().to_vec().as_slice());
        self.current_hash = block.get_pre_block_hash().clone();
        return Some(block);
    }
}
