use clap::Parser;
use serde::{Deserialize, Serialize};

use persist_core::error::Error;
use persist_core::protocol::PruneRequest;

use crate::format;
use crate::{daemon, GlobalOpts};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Parser)]
pub struct Opts {
    /// Also prune file of stopped, but still managed, processes.
    #[arg(long)]
    pub stopped: bool,
}

pub async fn handle(_: GlobalOpts, opts: Opts) -> Result<(), Error> {
    let request = PruneRequest {
        stopped: opts.stopped,
    };

    let mut daemon = daemon::connect().await?;
    let response = daemon.prune(request).await?;

    if !response.pruned_files.is_empty() {
        for pruned_file in response.pruned_files {
            let msg = format!(
                "'{}' successfully pruned",
                format::format_path(&pruned_file)
            );
            format::success(msg);
        }
    } else {
        format::success("nothing to prune");
    }

    Ok(())
}
