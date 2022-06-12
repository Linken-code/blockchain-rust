use super::{run_cmd, Config};
use clap::{ArgEnum, Args, Subcommand};
use core::GLOBAL_CONFIG;

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[clap(about = "钱包")]
    Wallet { opt: Option<String> },

    #[clap(arg_required_else_help = true, about = "节点")]
    Center { opt: Option<String> },

    #[clap(arg_required_else_help = true, about = "矿工")]
    Miner { opt: Option<String> },

    #[clap(arg_required_else_help = true, about = "查看")]
    Check {
        #[clap(arg_enum)]
        opt: CheckList,
    },

    #[clap(arg_required_else_help = true, about = "转账")]
    Send {
        #[clap(flatten)]
        opt: Transform,
    },

    #[clap(arg_required_else_help = true, about = "新建")]
    New {
        #[clap(subcommand)]
        opt: Mode,
    },
}

#[derive(Clone, Subcommand, Debug)]
pub enum Mode {
    Wallet { params: Option<String> },
    Miner { params: Option<String> },
    Center { params: Option<String> },
}

#[derive(Clone, ArgEnum, Debug)]
pub enum CheckList {
    WalletList,
    Chain,
    Utxo,
}

#[derive(Args, Debug)]
pub struct Transform {
    pub from: String,
    pub to: String,
    pub amount: i32,
    pub mine: i32,
}

pub fn process(command: Commands, cfg: Config) {
    GLOBAL_CONFIG.set_node_addr(cfg.port);
    run_cmd(command)
}
