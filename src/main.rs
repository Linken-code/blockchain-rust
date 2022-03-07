use core::blockchain;

fn main() {
    let mut bc = blockchain::BlockChain::new_blockchain();
    bc.add_block("a->b:5btc".to_string());
    bc.add_block("c->d:5btc".to_string());
    for i in bc.blocks {
        println!("digging block");
        println!("{:#?}", i);
    }
}
