mod cli;
use clap::Parser;
use cli::{cli_error_and_die, process, Cli};
use env_logger::Env;
use log::info;

///命令：
///配置运行端口 cargo run -- -p port

///新建钱包 cargo run -- new wallet
///新建区块链 cargo run -- new center address
///运行新节点 cargo run -- new miner

///向钱包地址发送币 cargo run -- send  from to amount mine

///新建矿工节点cargo run --  miner address
///检查地址余额 cargo run -- wallet address
///创建新区块链 cargo run -- center address

///查看钱包地址 cargo run -- check wallet-list
///查看区块链 cargo run -- check chain
///重置utxo集 cargo run -- check utxo

fn main() {
    // 注意，env_logger 必须尽可能早的初始化
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    //日志开始
    info!("日志开始，Process start");

    let Cli { cli_opts, command } = Cli::parse();

    match cli_opts.to_config() {
        Ok(cfg) => match command {
            Some(command) => process(command, cfg),
            None => println!("None"),
        },
        Err(e) => cli_error_and_die(e.to_string().as_str(), 123),
    }
}
