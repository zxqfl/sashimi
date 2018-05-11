#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate structopt;
extern crate rayon;
extern crate engine;
extern crate simplelog;
extern crate mcts;
extern crate chess;
extern crate slurp;
extern crate rand;

mod config;
mod playgame;
mod import;
mod self_play_state;
mod iternum;

use simplelog::{WriteLogger, CombinedLogger, LevelFilter, Config, TermLogger};
use structopt::StructOpt;
use import::*;

#[derive(StructOpt)]
#[structopt(name = "selfplay")]
struct Opt {
    #[structopt(short = "-l", long = "log-file-path", parse(from_os_str))]
    log_file_path: PathBuf,
    #[structopt(long = "config", parse(from_os_str))]
    config_path: PathBuf,
}


fn main() {
    let options = Opt::from_args();
    CombinedLogger::init(vec![
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            File::create(&options.log_file_path).unwrap()),
        TermLogger::new(
            LevelFilter::Debug,
            Config::default()).unwrap()
    ]).unwrap();
    let config = serde_json::from_reader(
        File::open(options.config_path)
            .expect("opening config file"))
        .expect("parsing config");
    SelfPlayState::loop_(&config);
}
