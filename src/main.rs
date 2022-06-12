mod cli;
use clap::Parser;
use cli::{cli_error_and_die, process, Cli};
use env_logger::Env;
use log::info;

//命令：
//新建钱包 cargo run -- -m wallet
//新建区块链 cargo run -- -m center -p address
//新建节点 cargo run -- -m miner
//新增矿工地址 cargo run -- -m miner -p address

//向钱包地址发送币 cargo run -- -m send -p from to amount mine

//检查地址余额 cargo run -- -m balance -p address
//查看钱包地址 cargo run -- -m other tx -l
//查看区块链 cargo run -- -m other tx -c
//重置utxo集 cargo run -- -m other tx -r

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
