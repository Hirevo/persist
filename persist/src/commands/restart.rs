use serde::{Deserialize, Serialize};
use structopt::StructOpt;

use persist_core::error::Error;

use crate::daemon;
use crate::format;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, StructOpt)]
pub struct Opts {
    /// The name of the process to restart
    #[structopt(name = "process-name")]
    name: String,
}

pub async fn handle(opts: Opts) -> Result<(), Error> {
    let mut daemon = daemon::connect().await?;
    let msg = format!("process '{}' successfully restarted.", opts.name);
    daemon.restart(opts.name).await?;
    format::success(msg);

    Ok(())
}
