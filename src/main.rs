use core::{
    convert_address, hash_pub_key, send_tx, validate_address, BlockChain, Server, Transaction,
    UTXOSet, Wallets, ADDRESS_CHECK_SUM_LEN, CENTER_NODE, GLOBAL_CONFIG,
};
use data_encoding::HEXLOWER;
use utils::coder::base58_decode;
#[macro_use]
extern crate clap;
use clap::App;

/// mine 标志指的是块会立刻被同一节点挖出来。必须要有这个标志，因为初始状态时，网络中没有矿工节点。
const MINE_TRUE: i32 = 1;

fn new_blockchain(address: &str) {
    //创建新区块链
    let blockchain = BlockChain::create_blockchain(address);
    let utxo_set = UTXOSet::new(blockchain);
    utxo_set.reindex();
    println!("new_blockchain Done!");
}

fn new_wallet() {
    let mut wallet = Wallets::new();
    let address = wallet.create_wallet();
    println!("Your new address: {}", address);
}

fn new_node(miner: Option<&str>) {
    if let Some(addr) = miner {
        if validate_address(addr) == false {
            panic!("Wrong miner address!")
        }
        println!(
            "Mining is on. Address to receive rewards: {}",
            addr.to_string()
        );
        GLOBAL_CONFIG.set_mining_addr(addr.to_string());
    }
    let blockchain = BlockChain::new_blockchain();
    let socket_addr = GLOBAL_CONFIG.get_node_addr();
    Server::new(blockchain).start_server(socket_addr.as_str());
}

fn send_data(from: &str, to: &str, amount: i32, mine: i32) {
    if !validate_address(from) {
        panic!("ERROR: Sender address is not valid")
    }
    if !validate_address(to) {
        panic!("ERROR: Recipient address is not valid")
    }
    let blockchain = BlockChain::new_blockchain();
    let utxo_set = UTXOSet::new(blockchain.clone());
    // 创建 UTXO 交易
    let transaction = Transaction::new_utxo_transaction(from, to, amount, &utxo_set);

    if mine == MINE_TRUE {
        //  挖矿奖励
        let coinbase_tx = Transaction::new_coinbase_tx(from);
        // 挖新区块
        let block = blockchain.mine_block(&vec![transaction, coinbase_tx]);
        // 更新 UTXO 集
        utxo_set.update(&block);
    } else {
        send_tx(CENTER_NODE, &transaction);
    }
    println!("Success!")
}

fn get_balance(address: &str) {
    let address_valid = validate_address(address);
    if address_valid == false {
        panic!("ERROR: Address is not valid")
    }
    let payload = base58_decode(address);
    let pub_key_hash = &payload[1..payload.len() - ADDRESS_CHECK_SUM_LEN];
    let blockchain = BlockChain::new_blockchain();
    let utxo_set = UTXOSet::new(blockchain);
    let utxos = utxo_set.find_utxo(pub_key_hash);
    let mut balance = 0;
    for utxo in utxos {
        balance += utxo.get_value();
    }
    println!("Balance of {}: {}", address, balance);
}

fn println_wallet() {
    let wallets = Wallets::new();
    for address in wallets.get_addresses() {
        println!("{}", address)
    }
}

fn println_chain() {
    let mut block_iterator = BlockChain::new_blockchain().iterator();
    loop {
        let option = block_iterator.next();
        if option.is_none() {
            break;
        }
        let block = option.unwrap();
        println!("Pre block hash: {}", block.get_pre_block_hash());
        println!("Cur block hash: {}", block.get_hash());
        println!("Cur block Timestamp: {}", block.get_timestamp());
        for tx in block.get_transactions() {
            let cur_txid_hex = HEXLOWER.encode(tx.get_id());
            println!("- Transaction txid_hex: {}", cur_txid_hex);

            if tx.is_coinbase() == false {
                for input in tx.get_vin() {
                    let txid_hex = HEXLOWER.encode(input.get_txid());
                    let pub_key_hash = hash_pub_key(input.get_pub_key());
                    let address = convert_address(pub_key_hash.as_slice());
                    println!(
                        "-- Input txid = {}, vout = {}, from = {}",
                        txid_hex,
                        input.get_vout(),
                        address,
                    )
                }
            }
            for output in tx.get_vout() {
                let pub_key_hash = output.get_pub_key_hash();
                let address = convert_address(pub_key_hash);
                println!("-- Output value = {}, to = {}", output.get_value(), address,)
            }
        }
        println!()
    }
}

fn rest_utxo() {
    let blockchain = BlockChain::new_blockchain();
    let utxo_set = UTXOSet::new(blockchain);
    utxo_set.reindex();
    let count = utxo_set.count_transactions();
    println!("Done! There are {} transactions in the UTXO set.", count);
}

fn main() {
    println!("−−−−−−−−−−−−−−−−Mine Info −−−−−−−−−−−−−−−−−−−");
    let yml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yml).get_matches();
    //参数的值
    if let Some(mode) = matches.value_of("mode") {
        match mode {
            "wallet" => {
                if let Some(params) = matches.values_of("parameters") {
                    println!("{:?}", params);
                } else {
                    new_wallet();
                }
            }
            "center" => {
                if let Some(params) = matches.values_of("parameters") {
                    let list: Vec<&str> = params.collect();
                    let address = list[0];
                    new_blockchain(address);
                } else {
                    println!("请输入参数: -p [address] \n");
                }
            }
            "miner" => {
                if let Some(params) = matches.values_of("parameters") {
                    let list: Vec<&str> = params.collect();
                    let address = Some(list[0]);
                    new_node(address);
                } else {
                    new_node(None);
                }
            }
            "send" => {
                if let Some(params) = matches.values_of("parameters") {
                    let list: Vec<&str> = params.collect();
                    let from = list[0];
                    let to = list[1];
                    let amount = list[2].to_string().parse::<i32>().unwrap();
                    let mine = list[3].to_string().parse::<i32>().unwrap();
                    println!("{from}向{to}发送{amount}个币,{mine}");
                    send_data(from, to, amount, mine);
                } else {
                    println!("请输入参数: -p [address] \n");
                }
            }
            "balance" => {
                if let Some(params) = matches.values_of("parameters") {
                    let list: Vec<&str> = params.collect();
                    let address = list[0];
                    get_balance(address);
                } else {
                    println!("请输入参数: -p [address] \n");
                }
            }
            "other" => {
                if let Some(params) = matches.values_of("parameters") {
                    println!("{:?}", params);
                } else {
                    if let Some(matches) = matches.subcommand_matches("tx") {
                        if matches.is_present("check_list") {
                            println_wallet();
                        } else if matches.is_present("check_chain") {
                            println_chain();
                        } else if matches.is_present("rest_utxo") {
                            rest_utxo();
                        } else {
                            println!("Not printing testing ...");
                        }
                    }
                }
            }
            _ => unreachable!(),
        }
    }
    //   if let Some(matches) = matches.subcommand_matches("test") {
    //                 if matches.is_present("list") {
    //                     println!("Printing testing lists...");
    //                 } else if matches.is_present("wallet") {
    //                     println!("Printing wallet...");
    //                 } else {
    //                     println!("Not printing testing lists...");
    //                 }
    //             }
}

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
