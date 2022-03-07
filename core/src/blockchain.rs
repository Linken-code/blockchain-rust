use crate::block::Block;

pub struct BlockChain {
    pub blocks: Vec<Block>,
}

impl BlockChain {
    pub fn add_block(&mut self, data: String) {
        let pre_block = &self.blocks[self.blocks.len() - 1];
        let new_block = Block::new_block(data, pre_block.hash.clone()).expect("block error");
        self.blocks.push(new_block);
    }

    fn new_genesis_block() -> Result<Block, Box<dyn std::error::Error>> {
        let block = Block::new_block("This is the first block".to_string(), String::from(""))?;
        Ok(block)
    }

    pub fn new_blockchain() -> BlockChain {
        BlockChain {
            blocks: vec![BlockChain::new_genesis_block().unwrap()],
        }
    }
}
