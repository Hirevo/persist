use colored::Colorize;
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

use persist_core::error::Error;
use persist_core::protocol::{LogStreamSource, LogsRequest, LogsResponse};

use crate::daemon;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, StructOpt)]
pub struct Opts {
    /// The names of the processes to get logs for
    #[structopt(name = "process-name")]
    pub processes: Vec<String>,
    /// Get logs from all the processes
    #[structopt(long)]
    pub all: bool,
    /// The number of previous log lines to initially output
    #[structopt(long, short = "n", default_value = "10")]
    pub lines: usize,
    /// Disable log streaming
    #[structopt(long = "no-stream")]
    pub no_stream: bool,
    /// Only show logs from the process' stdout
    #[structopt(long)]
    pub out: bool,
    /// Only show logs from the process' stderr
    #[structopt(long)]
    pub err: bool,
}

pub async fn handle(opts: Opts) -> Result<(), Error> {
    let filters = match (opts.all, opts.processes) {
        (false, processes) if processes.is_empty() => {
            return Err(Error::from(String::from(
                "you must specify at least one process name or --all",
            )));
        }
        (false, processes) => Some(processes),
        (true, _) => None,
    };

    let source_filter = match (opts.out, opts.err) {
        (false, false) => None,
        (true, false) => Some(LogStreamSource::Stdout),
        (false, true) => Some(LogStreamSource::Stderr),
        (true, true) => {
            return Err(Error::from(String::from(
                "only one of `--out` or `--err` is expected (none of those means both).",
            )));
        }
    };

    let request = LogsRequest {
        filters,
        stream: !opts.no_stream,
        lines: opts.lines,
        source_filter,
    };

    let mut daemon = daemon::connect().await?;
    let mut logs = daemon.logs(request).await?;

    while let Some(response) = logs.next().await.transpose()? {
        let entry = match response {
            LogsResponse::Entry(entry) => entry,
            LogsResponse::Unsubscribed => break,
            LogsResponse::Subscribed => {
                // TODO(error): received `Subscribed` twice ??
                unreachable!()
            }
        };

        let source = match entry.source {
            LogStreamSource::Stdout => "(out)",
            LogStreamSource::Stderr => "(err)",
        };

        println!(
            " {} {} {} {}",
            entry.name.bright_blue().bold(),
            source.bold(),
            "|".bold(),
            entry.msg,
        );
    }

    Ok(())
}
