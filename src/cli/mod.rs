use log::error;

mod subcommand;
pub use subcommand::{process, CheckList, Commands, Transform};

mod opt;
pub use opt::{Config, Opts};

mod run;
pub use run::run_cmd;

mod entry;
pub use entry::Cli;

///打印错误消息并退出程序
pub fn cli_error_and_die(msg: &str, code: i32) {
    error!("Error parsing CLI,Error was: {}", msg);
    std::process::exit(code);
}
