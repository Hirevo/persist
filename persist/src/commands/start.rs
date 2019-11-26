use std::env;

use serde::{Deserialize, Serialize};
use structopt::StructOpt;

use persist_core::error::Error;
use persist_core::protocol::NewProcess;

use crate::daemon;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, StructOpt)]
pub struct Opts {
    /// The name to give the process, to refer to it later
    #[structopt(long)]
    name: String,
    /// The command to launch
    command: Vec<String>,
}

pub async fn handle(opts: Opts) -> Result<(), Error> {
    println!("opts: {:?}", opts);

    let name = opts.name;
    let cmd = opts.command;
    let cwd = env::current_dir()?;
    let cwd = cwd.canonicalize()?;
    let env = env::vars().collect();

    let spec = NewProcess {
        name,
        cmd,
        cwd,
        env,
    };

    let mut daemon = daemon::connect().await?;
    daemon.start(spec).await?;

    Ok(())
}
