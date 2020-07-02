use serde::{Deserialize, Serialize};
use structopt::StructOpt;

use persist_core::error::Error;
use persist_core::protocol::PruneRequest;

use crate::daemon;
use crate::format;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, StructOpt)]
pub struct Opts {
    /// Also prune file of stopped, but still managed, processes.
    #[structopt(long)]
    pub stopped: bool,
}

pub async fn handle(opts: Opts) -> Result<(), Error> {
    let request = PruneRequest {
        stopped: opts.stopped,
    };

    let mut daemon = daemon::connect().await?;
    let response = daemon.prune(request).await?;

    if !response.pruned_files.is_empty() {
        for pruned_file in response.pruned_files {
            let msg = format!("'{}' successfully pruned", pruned_file);
            format::success(msg);
        }
    } else {
        format::success("nothing to prune");
    }

    Ok(())
}
