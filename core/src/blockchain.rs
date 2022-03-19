use crate::block::Block;
use dotenv::dotenv;
use sled::transaction::TransactionResult;
use sled::{Db, Tree};
use std::env;
use std::sync::{Arc, RwLock};
use utils::coder;

const TIP_BLOCK_HASH_KEY: &str = "tip_block_hash";
const BLOCKS_TREE: &str = "blocks";

#[derive(Clone)]
pub struct BlockChain {
    tip_hash: Arc<RwLock<String>>, // hash of last block
    db: Db,
}

impl BlockChain {
    // 生成创世块
    fn new_genesis_block() -> Block {
        let block = Block::new_block("This is the first block".to_string(), String::from(""));
        block
    }

    // 创建新的区块链
    pub fn create_blockchain() -> BlockChain {
        dotenv().ok();
        let key = "DBName";
        let name = env::var(key).expect("获取环境变量失败");
        let db = sled::open(name).expect("数据库不存在");
        let blocks_tree = db.open_tree(BLOCKS_TREE).expect("block库不存在");
        let data = blocks_tree
            .get(TIP_BLOCK_HASH_KEY)
            .expect("获取最后一个块的哈希失败");

        let tip_hash;
        if data.is_none() {
            // let coinbase_tx = Transaction::new_coinbase_tx(genesis_address);
            let block = self::BlockChain::new_genesis_block(); //创世块
            self::BlockChain::update_blocks_tree(&blocks_tree, &block);
            tip_hash = String::from(block.get_hash());
        } else {
            tip_hash = String::from_utf8(data.unwrap().to_vec()).unwrap();
        }

        BlockChain {
            tip_hash: Arc::new(RwLock::new(tip_hash)),
            db,
        }
    }

    // 添加一个区块到区块链
    pub fn add_block(&mut self, data: String) {
        let block_tree = self.db.open_tree(BLOCKS_TREE).unwrap();
        let db_data = block_tree
            .get(TIP_BLOCK_HASH_KEY)
            .expect("获取最后一个块的哈希失败");
        //最后一个块的哈希
        let last_hash = String::from_utf8(db_data.unwrap().to_vec()).unwrap();
        //新区块
        let block = Block::new_block(data, last_hash);

        let _: TransactionResult<(), ()> = block_tree.transaction(|tx_db| {
            //数据库插入当前区块
            tx_db
                .insert(block.get_hash(), coder::serialized(&block))
                .expect("插入新区块失败");

            // 获取当前tip_hash
            // let tip_block_bytes = tx_db
            //     .get(self.get_tip_hash())
            //     .unwrap()
            //     .expect("The tip hash is not valid");

            // let tip_block =
            //     coder::deserialized(tip_block_bytes.as_ref()).expect("反序列化区块失败");

            // if block.get_height() > tip_block.get_height() {
            //     let _ = tx_db.insert(TIP_BLOCK_HASH_KEY, block.get_hash()).unwrap();
            //     self.set_tip_hash(block.get_hash());
            // }

            //数据库插入当前区块hash
            tx_db
                .insert(TIP_BLOCK_HASH_KEY, block.get_hash())
                .expect("数据库插入tipHash失败");

            //替换链上的tip_hash
            self.set_tip_hash(block.get_hash());

            Ok(())
        });
    }

    // 更新区块树
    fn update_blocks_tree(blocks_tree: &Tree, block: &Block) {
        let block_hash = block.get_hash(); //区块hash
        let _: TransactionResult<(), ()> = blocks_tree.transaction(|tx_db| {
            let _ = tx_db.insert(block_hash, block.clone()); //插入当前区块hash和区块数据
            let _ = tx_db.insert(TIP_BLOCK_HASH_KEY, block_hash); //插入tip和当前区块hash
            Ok(())
        });
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

    //区块链迭代器
    pub fn iterator(&self) -> BlockchainIterator {
        BlockchainIterator::new(self.get_tip_hash(), self.db.clone())
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
