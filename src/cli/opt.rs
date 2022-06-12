use clap::Parser;
use log::info;
use std::error::Error;

#[derive(Parser, Debug)]
pub struct Opts {
    #[clap(short, long, help = "配置项，config")]
    pub config: Option<String>,

    #[clap(
        short,
        long,
        parse(try_from_str),
        help = "运行端口",
        default_value_t = 8080
    )]
    pub port: usize,

    #[clap(short, long, help = "数据库目录位置")]
    pub data_dir: Option<String>,
}

pub struct Config {
    pub config: String,
    pub port: String,
    pub data_dir: String,
}

impl Opts {
    pub fn to_config(&self) -> Result<Config, Box<dyn Error>> {
        //记录配置项
        info!("配置项，config opt is {:#?}", self);

        let port = self.port.to_string();
        let data_dir;
        let config;

        match &self.data_dir {
            Some(dir) => data_dir = dir.to_owned(),
            None => data_dir = String::from("dir"),
        }
        match &self.config {
            Some(cfg) => config = cfg.to_owned(),
            None => config = String::from("config"),
        }
        let cfg = Config {
            config,
            port,
            data_dir,
        };
        Ok(cfg)
    }
}
