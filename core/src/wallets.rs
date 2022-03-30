use crate::Wallet;
use std::collections::HashMap;
use std::env::current_dir;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Read, Write};
use utils::coder;

pub const WALLET_FILE: &str = "wallet.dat";

pub struct Wallets {
    wallets: HashMap<String, Wallet>,
}

impl Wallets {
    pub fn new() -> Wallets {
        let mut wallets = Wallets {
            wallets: HashMap::new(),
        };
        wallets.load_from_file();
        wallets
    }

    /// 创建一个钱包
    pub fn create_wallet(&mut self) -> String {
        let wallet = Wallet::new();
        let address = wallet.get_address();
        self.wallets.insert(address.clone(), wallet);
        self.save_to_file();
        return address;
    }

    //获取地址
    pub fn get_addresses(&self) -> Vec<String> {
        let mut addresses = vec![];
        for (address, _) in &self.wallets {
            addresses.push(address.clone())
        }
        return addresses;
    }

    /// 通过钱包地址查询钱包
    pub fn get_wallet(&self, address: &str) -> Option<&Wallet> {
        if let Some(wallet) = self.wallets.get(address) {
            return Some(wallet);
        }
        None
    }

    /// 从本地文件加载钱包
    pub fn load_from_file(&mut self) {
        let path = current_dir().unwrap().join(WALLET_FILE);
        if !path.exists() {
            return;
        }
        let mut file = File::open(path).unwrap();
        let metadata = file.metadata().expect("unable to read metadata");
        let mut buf = vec![0; metadata.len() as usize];
        let _ = file.read(&mut buf).expect("buffer overflow");
        let wallets = coder::deserialized(&buf[..]);
        self.wallets = wallets;
    }

    /// 钱包持久化到本地文件
    fn save_to_file(&self) {
        let path = current_dir().unwrap().join(WALLET_FILE);
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&path)
            .expect("unable to open wallet.dat");
        let mut writer = BufWriter::new(file);
        let wallets_bytes = coder::serialized(&self.wallets);
        writer.write(wallets_bytes.as_slice()).unwrap();
        let _ = writer.flush();
    }
}
