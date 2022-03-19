use core::blockchain;

fn main() {
    let mut bc = blockchain::BlockChain::create_blockchain();
    bc.add_block("a->b:5btc".to_string());
    bc.add_block("c->d:5btc".to_string());
    bc.add_block("d->e:5btc".to_string());
    for i in bc.blocks {
        println!("digging block");
        println!("{:#?}", i);
    }
}
