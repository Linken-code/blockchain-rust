use crate::block::Block;

pub struct BlockChain {
    pub blocks: Vec<Block>,
}

impl BlockChain {
    // 添加一个区块到区块链
    pub fn add_block(&mut self, data: String) {
        let pre_block = &self.blocks[self.blocks.len() - 1];
        let new_block = Block::new_block(data, pre_block.get_hash()).expect("block error");
        self.blocks.push(new_block);
    }
    // 生成创世块
    fn new_genesis_block() -> Result<Block, Box<dyn std::error::Error>> {
        let block = Block::new_block("This is the first block".to_string(), String::from(""))?;
        Ok(block)
    }
    // 创建新的区块链
    pub fn create_blockchain() -> BlockChain {
        BlockChain {
            blocks: vec![BlockChain::new_genesis_block().unwrap()],
        }
    }
}
