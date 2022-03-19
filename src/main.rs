use core::blockchain;

fn main() {
    let mut blockchain = blockchain::BlockChain::create_blockchain();
    blockchain.add_block("a->b:5btc".to_string());
    blockchain.add_block("c->d:5btc".to_string());
    blockchain.add_block("d->e:5btc".to_string());
    // for i in blockchain.tip_hash {
    //     println!("digging block");
    //     println!("{:#?}", i);
    // }
}
