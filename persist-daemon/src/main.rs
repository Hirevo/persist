use std::env;

use nix::unistd::ForkResult;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use tokio::runtime::Runtime;

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
            // 1st fork, done to transform into an orphaned process (by making the parent to exit immediately afterwards).
            // Orphaned processes makes the `init` process responsible for their cleanup.
            if let ForkResult::Parent { .. } = nix::unistd::fork()? {
                std::process::exit(0);
            }

            // Leave current session into our own separate session, to clear the controlling TTY.
            let _ = nix::unistd::setsid()?;

            // 2nd fork, performed because, since `setsid`, we were a session leader
            // that could potentially acquire a controlling TTY.
            // This new fork is so that the child will no longer be a session leader.
            if let ForkResult::Parent { .. } = nix::unistd::fork()? {
                std::process::exit(0);
            }

            let home_dir = persist_core::daemon::home_dir()?;
            let _ = std::fs::create_dir(&home_dir);
            env::set_current_dir(home_dir)?;

            let mut runtime = Runtime::new()?;
            let outcome = runtime.block_on(server::start());
            if let Err(err) = outcome {
                eprintln!("error: {}", err);
            }
        }
    };

    Ok(())
}
