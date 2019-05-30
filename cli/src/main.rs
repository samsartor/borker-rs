use structopt::StructOpt;
use serde::{Deserialize, Serialize};
use dirs::config_dir;

#[derive(StructOpt)]
pub enum Command { }

#[derive(Deserialize, Serialize)]
pub struct Config {

}

impl Default {
    fn default() -> Config {

    }
}

pub fn main() {
    let opt = Opt::from_args();
}
