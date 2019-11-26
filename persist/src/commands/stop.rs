use serde::{Deserialize, Serialize};
use structopt::StructOpt;

use persist_core::error::Error;

use crate::daemon;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, StructOpt)]
pub struct Opts {
    /// The name of the process to stop
    #[structopt(name = "process-name")]
    name: String,
}

pub async fn handle(opts: Opts) -> Result<(), Error> {
    println!("opts: {:?}", opts);

    let mut daemon = daemon::connect().await?;
    daemon.stop(opts.name).await?;

    Ok(())
}
