use log::warn;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::env;
use std::sync::RwLock;

pub static GLOBAL_CONFIG: Lazy<Config> = Lazy::new(|| Config::new(None));

/// 默认的中心节点ip地址
static DEFAULT_NODE_ADDR: &str = "127.0.0.1:2001";
///节点地址
const NODE_ADDRESS_KEY: &str = "NODE_ADDRESS";
///矿工地址
const MINING_ADDRESS_KEY: &str = "MINING_ADDRESS";

/// Node 配置
pub struct Config {
    inner: RwLock<HashMap<String, String>>,
}

impl Config {
    pub fn new(port: Option<String>) -> Config {
        let node_addr;
        match port {
            Some(port) => node_addr = String::from("127.0.0.1:") + &port,
            // 从环境变量获取节点地址
            None => match env::var("NODE_ADDRESS") {
                Ok(val) => node_addr = val,
                Err(e) => {
                    warn!(
                        "无法解析环境变量NODE_ADDRESS，将按照默认端口运行。 couldn't interpret NODE_ADDRESS,{}",
                        e
                    );
                    node_addr = String::from(DEFAULT_NODE_ADDR)
                }
            },
        }
        let mut map = HashMap::new();
        map.insert(String::from(NODE_ADDRESS_KEY), node_addr);

        Config {
            inner: RwLock::new(map),
        }
    }

    /// 设置节点运行IP地址
    pub fn set_node_addr(&self, port: String) {
        let node_addr = String::from("127.0.0.1:") + &port;
        let mut inner = self.inner.write().unwrap();
        let _ = inner.insert(String::from(NODE_ADDRESS_KEY), node_addr);
    }

    /// 获取节点运行IP地址
    pub fn get_node_addr(&self) -> String {
        let inner = self.inner.read().unwrap();
        let addr = if let Some(addr) = inner.get(NODE_ADDRESS_KEY) {
            addr.to_string().clone()
        } else {
            "none".to_string()
        };
        addr
    }

    /// 设置矿工钱包地址
    pub fn set_mining_addr(&self, addr: String) {
        let mut inner = self.inner.write().unwrap();
        let _ = inner.insert(String::from(MINING_ADDRESS_KEY), addr);
    }

    /// 获取矿工钱包地址
    pub fn get_mining_addr(&self) -> Option<String> {
        let inner = self.inner.read().expect("获取矿工节点失败");
        if let Some(addr) = inner.get(MINING_ADDRESS_KEY) {
            return Some(addr.clone());
        }
        None
    }

    /// 检查矿工节点
    pub fn is_miner(&self) -> bool {
        let inner = self.inner.read().unwrap();
        inner.contains_key(MINING_ADDRESS_KEY)
    }
}
