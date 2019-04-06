#[macro_use]
extern crate mysql;
#[macro_use]
extern crate serde_derive;

mod config;
mod cron;
mod roosterteeth;

use config::Config;
use cron::Cron;
use std::env;

fn main() {
    for argument in env::args() {
        let config = Config::load();
        let mut job = Cron::init(&config.address, &config.userpass);

        if argument == "--cron" {
            job.run();
        } else if argument == "--dump" {
            job.dump();
        }
    }
}
