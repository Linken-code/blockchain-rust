use crate::block::Block;
use data_encoding::HEXLOWER;
use num_bigint::{BigInt, Sign};
use std::borrow::Borrow;
use std::ops::ShlAssign;
use utils::coder;

// 难度值，这里表示哈希的前20位必须是0
const TARGET_BITS: i32 = 20;

// nonce 最大值,限制 nonce 避免整型溢出
const MAX_NONCE: i64 = i64::MAX;

pub struct ProofOfWork {
    block: Block,
    target: BigInt,
}

//工作量证明
impl ProofOfWork {
    //新建工作量证明
    pub fn new_proof_of_work(block: Block) -> ProofOfWork {
        //bigInt 初始化为 1
        let mut target = BigInt::from(1);
        // target 等于 1 左移 256 - TARGET_BITS 位
        target.shl_assign(256 - TARGET_BITS);
        ProofOfWork { block, target }
    }

    // 工作量证明用到的数据
    fn prepare_data(&mut self, nonce: i64) -> Vec<u8> {
        let pre_block_hash = self.block.get_pre_block_hash();
        let transactions_hash = self.block.hash_transactions();
        let timestamp = self.block.get_timestamp();
        let mut data_bytes = vec![];
        data_bytes.extend(pre_block_hash.as_bytes());
        data_bytes.extend(transactions_hash.as_bytes());
        data_bytes.extend(timestamp.to_be_bytes());
        data_bytes.extend(TARGET_BITS.to_be_bytes());
        data_bytes.extend(nonce.to_be_bytes());
        data_bytes
    }

    // 工作量证明的核心就是寻找有效的哈希
    pub fn run(&mut self) -> (i64, String) {
        // 1.在比特币中，当一个块被挖出来以后，“target bits” 代表了区块头里存储的难度，也就是开头有多少个 0。
        // 2.这里的 20 指的是算出来的哈希前 20 位必须是 0，如果用 16 进制表示，就是前 5 位必须是 0，这一点从
        //   最后的输出可以看出来。
        //   例如：target 16进制输出是 0000100000000000000000000000000000000000000000000000000000000000
        //   目前我们并不会实现一个动态调整目标的算法，所以将难度定义为一个全局的常量即可。
        // 3.将哈希与目标数 target 进行比较：先把哈希转换成一个大整数，然后检测它是否小于目标，小就是有效的，反之无效。
        let mut nonce = 0;
        let mut hash = Vec::new();
        println!("Mining the block");
        while nonce <= MAX_NONCE {
            let data = self.prepare_data(nonce); //用来哈希的数据
            hash = coder::sha256_digest(data.as_slice()); //hash函数
            let hash_int = BigInt::from_bytes_be(Sign::Plus, hash.as_slice()); //将hash转换为大整数

            if hash_int.lt(self.target.borrow()) {
                // 将hash整数与目标比较
                println!("当前hash={}", HEXLOWER.encode(hash.as_slice()));
                break;
            } else {
                nonce += 1;
            }
        }
        (nonce, HEXLOWER.encode(hash.as_slice()))
    }
}
