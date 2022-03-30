use crate::block::Block;
use crate::blockchain::BlockChain;
use crate::transaction::TXOutput;
use data_encoding::HEXLOWER;
use std::collections::HashMap;
use utils::coder;

const UTXO_TREE: &str = "chainstate";
//未花费交易输出
pub struct UTXOSet {
    blockchain: BlockChain,
}

impl UTXOSet {
    /// 创建 UTXO 集
    pub fn new(blockchain: BlockChain) -> UTXOSet {
        UTXOSet { blockchain }
    }

    //获取UTXO中的区块链
    pub fn get_blockchain(&self) -> &BlockChain {
        &self.blockchain
    }

    // 找到未花费的输出
    pub fn find_spendable_outputs(
        &self,
        pub_key_hash: &[u8],
        amount: i32,
    ) -> (i32, HashMap<String, Vec<usize>>) {
        let mut unspent_outputs: HashMap<String, Vec<usize>> = HashMap::new();
        let mut accmulated = 0;
        let db = self.blockchain.get_db(); //获取UTXO数据库
        let utxo_tree = db.open_tree(UTXO_TREE).expect("无法找到UTXO集");
        for item in utxo_tree.iter() {
            let (k, v) = item.expect("迭代失败");
            let txid_hex = HEXLOWER.encode(k.to_vec().as_slice());
            let outs: Vec<TXOutput> = coder::deserialized(v.to_vec().as_slice());
            for (idx, out) in outs.iter().enumerate() {
                if out.is_locked_with_key(pub_key_hash) && accmulated < amount {
                    accmulated += out.get_value();
                    if unspent_outputs.contains_key(txid_hex.as_str()) {
                        unspent_outputs
                            .get_mut(txid_hex.as_str())
                            .unwrap()
                            .push(idx);
                    } else {
                        unspent_outputs.insert(txid_hex.clone(), vec![idx]);
                    }
                }
            }
        }
        (accmulated, unspent_outputs)
    }

    // 通过公钥哈希查找 UTXO 集
    pub fn find_utxo(&self, pub_key_hash: &[u8]) -> Vec<TXOutput> {
        let db = self.blockchain.get_db();
        let utxo_tree = db.open_tree(UTXO_TREE).expect("无法找到UTXO集");
        let mut utxos = vec![];
        for item in utxo_tree.iter() {
            let (_, v) = item.expect("迭代失败");
            let outs: Vec<TXOutput> = coder::deserialized(v.to_vec().as_slice());
            for out in outs.iter() {
                if out.is_locked_with_key(pub_key_hash) {
                    utxos.push(out.clone())
                }
            }
        }
        utxos
    }

    // 统计 UTXO 集合中的交易数量
    pub fn count_transactions(&self) -> i32 {
        let db = self.blockchain.get_db();
        let utxo_tree = db.open_tree(UTXO_TREE).expect("无法找到UTXO集");
        let mut counter = 0;
        for _ in utxo_tree.iter() {
            counter += 1;
        }
        counter
    }

    // 重建 UTXO 集
    pub fn reindex(&self) {
        let db = self.blockchain.get_db();
        let utxo_tree = db.open_tree(UTXO_TREE).expect("无法找到UTXO集");
        utxo_tree.clear().expect("清空utxo数据集失败"); //清空utxo数据集

        let utxo_map = self.blockchain.find_utxo();
        for (txid_hex, outs) in &utxo_map {
            let txid = HEXLOWER.decode(txid_hex.as_bytes()).unwrap();
            let value = coder::serialized(outs);
            let _ = utxo_tree.insert(txid.as_slice(), value).unwrap();
        }
    }

    /// 使用来自区块的交易更新 UTXO 集
    pub fn update(&self, block: &Block) {
        let db = self.blockchain.get_db();
        let utxo_tree = db.open_tree(UTXO_TREE).unwrap();
        for tx in block.get_transactions() {
            if tx.is_coinbase() == false {
                for vin in tx.get_vin() {
                    let mut updated_outs = vec![];
                    let outs_bytes = utxo_tree.get(vin.get_txid()).unwrap().unwrap();
                    let outs: Vec<TXOutput> = coder::deserialized(outs_bytes.as_ref());
                    for (idx, out) in outs.iter().enumerate() {
                        if idx != vin.get_vout() {
                            updated_outs.push(out.clone())
                        }
                    }
                    if updated_outs.len() == 0 {
                        let _ = utxo_tree.remove(vin.get_txid()).unwrap();
                    } else {
                        let outs_bytes = coder::serialized(&updated_outs);
                        utxo_tree.insert(vin.get_txid(), outs_bytes).unwrap();
                    }
                }
            }
            let mut new_outputs = vec![];
            for out in tx.get_vout() {
                new_outputs.push(out.clone())
            }
            let outs_bytes = coder::serialized(&new_outputs);
            let _ = utxo_tree.insert(tx.get_id(), outs_bytes).unwrap();
        }
    }
}
