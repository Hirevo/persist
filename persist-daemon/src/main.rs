use std::env;

use nix::unistd::ForkResult;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

pub mod server;

use persist_core::error::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, StructOpt)]
#[structopt(about, author)]
pub enum Opts {
    /// Start the daemon
    Start,
}

fn main() -> Result<(), Error> {
    let config = Opts::from_args();

    match config {
        Opts::Start => {
            if let ForkResult::Parent { .. } = nix::unistd::fork()? {
                std::process::exit(0);
            }
            let _ = nix::unistd::setsid()?;
            if let ForkResult::Parent { .. } = nix::unistd::fork()? {
                std::process::exit(0);
            }

            let home_dir = persist_core::daemon::home_dir()?;
            let _ = std::fs::create_dir(&home_dir);
            env::set_current_dir(home_dir)?;

            let runtime = tokio::runtime::Runtime::new()?;
            let outcome = runtime.block_on(server::start());
            if let Err(err) = outcome {
                eprintln!("error: {}", err);
            }
        }
    };

    Ok(())
}
