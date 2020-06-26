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
    /// The number of previous log lines to initially output
    #[structopt(long, short = "n", default_value = "10")]
    pub lines: usize,
    /// Disable log streaming
    #[structopt(long = "no-stream")]
    pub no_stream: bool,
}

pub async fn handle(opts: Opts) -> Result<(), Error> {
    let request = LogsRequest {
        processes: opts.processes,
        stream: !opts.no_stream,
        lines: opts.lines,
        source_filter: None,
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
