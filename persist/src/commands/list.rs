use humansize::file_size_opts::CONVENTIONAL;
use humansize::FileSize;
use prettytable::format::consts::FORMAT_NO_LINESEP_WITH_TITLE;
use prettytable::Table;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

use persist_core::error::Error;

use crate::daemon;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, StructOpt)]
pub struct Opts {}

pub async fn handle(_: Opts) -> Result<(), Error> {
    let mut daemon = daemon::connect().await?;
    let metrics = daemon.list().await?;

    let mut table = Table::new();
    table.set_format(*FORMAT_NO_LINESEP_WITH_TITLE);
    table.set_titles(row![b => "Name", "PID", "Status", "CPU", "Memory"]);
    if metrics.is_empty() {
        table.add_row(row![bcH5 => "Empty list."]);
    } else {
        for metric in metrics {
            let name = metric.name;
            let status = metric.status;
            let cpu_usage = format!("{} %", metric.cpu_usage);
            let mem_usage = metric.mem_usage.file_size(CONVENTIONAL).unwrap();
            let pid = match metric.pid {
                Some(pid) => pid.to_string(),
                None => "none".to_string(),
            };
            table.add_row(row![name, pid, status, cpu_usage, mem_usage]);
        }
    }
    table.printstd();

    Ok(())
}
