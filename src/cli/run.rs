use super::subcommand::{CheckList, Commands, Mode};
use core::{
    convert_address, hash_pub_key, send_tx, validate_address, BlockChain, Server, Transaction,
    UTXOSet, Wallets, ADDRESS_CHECK_SUM_LEN, CENTER_NODE, GLOBAL_CONFIG,
};
use data_encoding::HEXLOWER;
use log::info;
use utils::coder::base58_decode;

/// mine 标志指的是块会立刻被同一节点挖出来。必须要有这个标志，因为初始状态时，网络中没有矿工节点。
const MINE_TRUE: i32 = 1;
//运行子命令
pub fn run_cmd(command: Commands) {
    info!("子命令，cmd is {:#?}", command);
    match command {
        Commands::Wallet { opt } => match opt {
            Some(address) => {
                get_balance(&address);
            }
            None => {
                new_wallet();
            }
        },
        Commands::Center { opt } => match opt {
            Some(address) => {
                new_blockchain(&address);
            }
            None => {}
        },
        Commands::Miner { opt } => {
            if let Some(address) = opt {
                info!("新建矿工节点，钱包地址为 {}，new a miner", address);
                new_node(Some(address));
            }
        }
        Commands::Check { opt } => match opt {
            CheckList::Chain => {
                info!("查看区块链列表，check chain");
                println_chain();
            }
            CheckList::Utxo => {
                info!("查看未打包交易，check Utxo");
                rest_utxo();
            }
            CheckList::WalletList => {
                info!("查看钱包列表，check wallet-list");
                println_wallet();
            }
        },
        Commands::New { opt } => match opt {
            Mode::Wallet { params } => {
                if let None = params {
                    info!("新建钱包，new a wallet");
                    new_wallet();
                }
            }
            Mode::Miner { params } => {
                if let None = params {
                    info!("生成新矿工节点，new a miner");
                    new_node(None);
                }
            }
            Mode::Center { params } => {
                if let Some(address) = params {
                    info!("新建区块，new a blockchain");
                    new_blockchain(&address);
                }
            }
        },
        Commands::Send { opt } => {
            info!("发生转账！");
            send_data(&opt.from, &opt.to, opt.amount, opt.mine);
        }
    }
}

//创建新区块链
fn new_blockchain(address: &str) {
    let blockchain = BlockChain::create_blockchain(address);
    let utxo_set = UTXOSet::new(blockchain);
    utxo_set.reindex();
    println!("new_blockchain Done!");
}

//创建新钱包地址
fn new_wallet() {
    let mut wallet = Wallets::new();
    let address = wallet.create_wallet();
    println!("Your new address: {}", address);
}

//运行新节点
fn new_node(miner: Option<String>) {
    if let Some(addr) = miner {
        if validate_address(&addr) == false {
            panic!("Wrong miner address!")
        }
        println!("Mining is on. Address to receive rewards: {}", addr);
        GLOBAL_CONFIG.set_mining_addr(addr);
    }
    let blockchain = BlockChain::new_blockchain();
    //节点IP地址
    let socket_addr = GLOBAL_CONFIG.get_node_addr();
    Server::new(blockchain).start_server(socket_addr.as_str());
}

//转账交易
fn send_data(from: &str, to: &str, amount: i32, mine: i32) {
    println!("{from}向{to}发送{amount}个币,{mine}");
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

//获取钱包地址余额
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

//打印钱包列表
fn println_wallet() {
    let wallets = Wallets::new();
    for address in wallets.get_addresses() {
        println!("{}", address)
    }
}

//打印区块链列表
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

//查看未打包交易
fn rest_utxo() {
    let blockchain = BlockChain::new_blockchain();
    let utxo_set = UTXOSet::new(blockchain);
    utxo_set.reindex();
    let count = utxo_set.count_transactions();
    println!("Done! There are {} transactions in the UTXO set.", count);
}
