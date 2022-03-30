use crate::wallet::{hash_pub_key, ADDRESS_CHECK_SUM_LEN};
use crate::wallets::Wallets;
use crate::BlockChain;
use crate::UTXOSet;
use data_encoding::HEXLOWER;
use serde::{Deserialize, Serialize};
use utils::coder;

// 挖矿奖励金
const SUBSIDY: i32 = 10;

//交易
#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct Transaction {
    id: Vec<u8>,         //整个交易的哈希值转换为交易id
    vin: Vec<TXInput>,   //输入
    vout: Vec<TXOutput>, //输出
}

//交易输出
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TXOutput {
    value: i32,            //币的数量
    pub_key_hash: Vec<u8>, // 地址公钥哈希
}

impl TXOutput {
    pub fn new(value: i32, address: &str) -> TXOutput {
        let mut output = TXOutput {
            value,
            pub_key_hash: vec![],
        };
        output.lock(address);
        output
    }

    //获取输出币值
    pub fn get_value(&self) -> i32 {
        self.value
    }

    //获取公钥
    pub fn get_pub_key_hash(&self) -> &[u8] {
        self.pub_key_hash.as_slice()
    }

    //通过地址锁定输出的公钥hash
    fn lock(&mut self, address: &str) {
        let payload = coder::base58_decode(address);
        let pub_key_hash = payload[1..payload.len() - ADDRESS_CHECK_SUM_LEN].to_vec();
        self.pub_key_hash = pub_key_hash;
    }

    //检查是否提供的公钥哈希被用于锁定输出
    pub fn is_locked_with_key(&self, pub_key_hash: &[u8]) -> bool {
        self.pub_key_hash.eq(pub_key_hash)
    }
}

//交易输入
#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct TXInput {
    tx_id: Vec<u8>,     // 一个交易输入引用了前一笔交易的一个输出，ID表明是之前的哪一笔交易
    vout: usize,        // 交易中所有输出的索引
    signature: Vec<u8>, // 签名
    pub_key: Vec<u8>,   // 原生的公钥
}

impl TXInput {
    //新建输入
    pub fn new(txid: &[u8], vout: usize) -> TXInput {
        TXInput {
            tx_id: txid.to_vec(),
            vout,
            signature: vec![],
            pub_key: vec![],
        }
    }

    // 检查输入使用了指定密钥来解锁一个输出
    pub fn uses_key(&self, pub_key_hash: &[u8]) -> bool {
        let locking_hash = hash_pub_key(self.pub_key.as_slice());
        return locking_hash.eq(pub_key_hash);
    }

    pub fn get_txid(&self) -> &[u8] {
        self.tx_id.as_slice()
    }

    pub fn get_vout(&self) -> usize {
        self.vout
    }

    pub fn get_pub_key(&self) -> &[u8] {
        self.pub_key.as_slice()
    }
}

impl Transaction {
    // 创建一个 coinbase 交易，该没有输入，只有一个输出
    pub fn new_coinbase_tx(to: &str) -> Transaction {
        let tx_out = TXOutput::new(SUBSIDY, to);
        let tx_input = TXInput::default();
        let mut tx = Transaction {
            id: vec![],
            vin: vec![tx_input],
            vout: vec![tx_out],
        };
        tx.id = tx.hash();
        tx
    }

    // 创建一笔 UTXO 的交易
    pub fn new_utxo_transaction(
        from: &str,
        to: &str,
        amount: i32,
        utxo_set: &UTXOSet,
    ) -> Transaction {
        // 1.查找钱包
        let wallets = Wallets::new();
        let wallet = wallets.get_wallet(from).expect("unable to found wallet");
        let public_key_hash = hash_pub_key(wallet.get_public_key());
        // 2.找到足够的未花费输出
        let (accumulated, valid_outputs) =
            utxo_set.find_spendable_outputs(public_key_hash.as_slice(), amount);
        if accumulated < amount {
            panic!("Error: Not enough funds")
        };
        // 3.交易数据
        // 3.1.交易的输入
        let mut inputs = vec![];
        for (txid_hex, outs) in valid_outputs {
            let txid = HEXLOWER.decode(txid_hex.as_bytes()).unwrap();
            for out in outs {
                let input = TXInput {
                    tx_id: txid.clone(), // 上一笔交易的ID
                    vout: out,           // 输出的索引
                    signature: vec![],
                    pub_key: wallet.get_public_key().to_vec(),
                };
                inputs.push(input);
            }
        }
        // 3.2.交易的输出
        let mut outputs = vec![TXOutput::new(amount, to)];
        // 如果 UTXO 总数超过所需，则产生找零
        if accumulated > amount {
            outputs.push(TXOutput::new(accumulated - amount, from)) // to: 币收入
        };
        // 4.生成交易
        let mut tx = Transaction {
            id: vec![],
            vin: inputs,
            vout: outputs,
        };
        // 生成交易ID
        tx.id = tx.hash();
        // 5.交易中的 TXInput 签名
        tx.sign(utxo_set.get_blockchain(), wallet.get_pkcs8());
        tx
    }

    /// 创建一个修剪后的交易副本
    fn trimmed_copy(&self) -> Transaction {
        let mut inputs = vec![];
        let mut outputs = vec![];
        for input in &self.vin {
            let tx_input = TXInput::new(input.get_txid(), input.get_vout());
            inputs.push(tx_input);
        }
        for output in &self.vout {
            outputs.push(output.clone());
        }
        Transaction {
            id: self.id.clone(),
            vin: inputs,
            vout: outputs,
        }
    }

    /// 对交易的每个输入进行签名
    fn sign(&mut self, blockchain: &BlockChain, pkcs8: &[u8]) {
        let mut tx_copy = self.trimmed_copy();

        for (idx, vin) in self.vin.iter_mut().enumerate() {
            // 查找输入引用的交易
            let prev_tx_option = blockchain.find_transaction(vin.get_txid());
            if prev_tx_option.is_none() {
                panic!("ERROR: Previous transaction is not correct")
            }
            if let Some(prev_tx) = prev_tx_option {
                tx_copy.vin[idx].pub_key = prev_tx.vout[vin.vout].pub_key_hash.clone();
            }
            tx_copy.vin[idx].signature = vec![];
            tx_copy.id = tx_copy.hash();
            tx_copy.vin[idx].pub_key = vec![];

            // 使用私钥对数据签名
            let signature = coder::ecdsa_p256_sha256_sign_digest(pkcs8, tx_copy.get_id());
            vin.signature = signature;
        }
    }

    /// 对交易的每个输入进行签名验证
    pub fn verify(&self, blockchain: &BlockChain) -> bool {
        if self.is_coinbase() {
            return true;
        }
        let mut tx_copy = self.trimmed_copy();
        for (idx, vin) in self.vin.iter().enumerate() {
            let prev_tx_option = blockchain.find_transaction(vin.get_txid());
            if prev_tx_option.is_none() {
                panic!("ERROR: Previous transaction is not correct")
            }
            if let Some(prev_tx) = prev_tx_option {
                tx_copy.vin[idx].pub_key = prev_tx.vout[vin.vout].pub_key_hash.clone();
            }
            tx_copy.vin[idx].signature = vec![];
            tx_copy.id = tx_copy.hash();
            tx_copy.vin[idx].pub_key = vec![];

            // 使用公钥验证签名
            let verify = coder::ecdsa_p256_sha256_sign_verify(
                vin.pub_key.as_slice(),
                vin.signature.as_slice(),
                tx_copy.get_id(),
            );
            if !verify {
                return false;
            }
        }
        true
    }

    // 生成交易的哈希
    fn hash(&self) -> Vec<u8> {
        let tx_copy = Transaction {
            id: vec![],
            vin: self.vin.clone(),
            vout: self.vout.clone(),
        };
        let tx_ser = coder::serialized(&tx_copy);
        coder::sha256_digest(tx_ser.as_slice())
    }

    /// 判断是否是 coinbase 交易
    pub fn is_coinbase(&self) -> bool {
        return self.vin.len() == 1 && self.vin[0].pub_key.len() == 0;
    }

    pub fn get_id(&self) -> &[u8] {
        self.id.as_slice()
    }

    pub fn get_id_bytes(&self) -> Vec<u8> {
        self.id.clone()
    }

    pub fn get_vin(&self) -> &[TXInput] {
        self.vin.as_slice()
    }

    pub fn get_vout(&self) -> &[TXOutput] {
        self.vout.as_slice()
    }
}
