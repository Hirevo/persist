use colored::Colorize;
use humansize::file_size_opts::CONVENTIONAL;
use humansize::FileSize;
use prettytable::format::consts::FORMAT_NO_LINESEP_WITH_TITLE;
use prettytable::Table;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

use persist_core::error::Error;
use persist_core::protocol::{ListRequest, ProcessStatus};

use crate::daemon;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, StructOpt)]
pub struct Opts {}

pub async fn handle(_: Opts) -> Result<(), Error> {
    let mut daemon = daemon::connect().await?;
    let metrics = daemon.list(ListRequest { filters: None }).await?;

    let mut table = Table::new();
    table.set_format(*FORMAT_NO_LINESEP_WITH_TITLE);
    table.set_titles(row![b => "Name", "PID", "Status", "CPU", "Memory"]);
    if metrics.is_empty() {
        table.add_row(row![bcH5 => "Empty list."]);
    } else {
        for metric in metrics {
            let name = metric.name;
            let status = match metric.status {
                ProcessStatus::Running => "running".green().bold(),
                ProcessStatus::Stopped => "stopped".red().bold(),
            };
            let cpu_usage = match metric.status {
                ProcessStatus::Running => format!("{} %", metric.cpu_usage),
                ProcessStatus::Stopped => "N/A".to_string(),
            };
            let mem_usage = match metric.status {
                ProcessStatus::Running => metric.mem_usage.file_size(CONVENTIONAL).unwrap(),
                ProcessStatus::Stopped => "N/A".to_string(),
            };
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
