use super::opt::Opts;
use super::subcommand::Commands;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "MyBlockchain")]
#[clap(author = "Linken. <294258474@qq.com>")]
#[clap(version = "1.0")]
#[clap(about = "Does awesome things", long_about = None)]
pub struct Cli {
    #[clap(flatten)]
    pub cli_opts: Opts,

    #[clap(subcommand)]
    pub command: Option<Commands>,
}
