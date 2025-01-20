use std::env;

use clap::Parser;
use persist_core::protocol::ProcessStatus;
use serde::{Deserialize, Serialize};

use persist_core::error::Error;
use persist_core::protocol::StartRequest;

use crate::format;
use crate::{daemon, GlobalOpts};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Parser)]
pub struct Opts {
    /// The name to give the process, to refer to it later
    #[arg(long)]
    pub name: Option<String>,
    /// Insert the process, but do not start it immediately
    #[arg(long)]
    pub stopped: bool,
    /// The command to launch
    pub command: Vec<String>,
}

pub async fn handle(_: GlobalOpts, opts: Opts) -> Result<(), Error> {
    if opts.command.is_empty() {
        return Err(Error::from(String::from("empty commands not permitted")));
    }

    let cmd = opts.command;
    let name = match opts.name {
        Some(name) => name,
        None => cmd[0].split('/').last().unwrap().to_string(),
    };
    let cwd = env::current_dir()?;
    let cwd = cwd.canonicalize()?;
    let env = env::vars().collect();
    let status = match opts.stopped {
        true => ProcessStatus::Stopped,
        false => ProcessStatus::Running,
    };

    let request = StartRequest {
        name,
        cmd,
        cwd,
        env,
        status,
    };

    let mut daemon = daemon::connect().await?;
    let msg = format!("process '{}' successfully started.", request.name);
    daemon.start(request).await?;
    format::success(msg);

    Ok(())
}
