use colored::Colorize;
use prettytable::format::{FormatBuilder, LinePosition, LineSeparator};
use prettytable::Table;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

use persist_core::error::Error;
use persist_core::protocol::{InfoRequest, InfoResponse, ProcessStatus};

use crate::daemon;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, StructOpt)]
pub struct Opts {
    /// The name of the process to get information about
    #[structopt(name = "process-name")]
    name: String,
}

pub async fn handle(opts: Opts) -> Result<(), Error> {
    let mut daemon = daemon::connect().await?;
    let InfoResponse { info } = daemon.info(InfoRequest { name: opts.name }).await?;

    let mut table = Table::new();
    let sep_pos = [LinePosition::Top, LinePosition::Bottom];
    let sep_chars = LineSeparator::new('-', '+', '+', '+');
    let table_fmt = FormatBuilder::new()
        .column_separator('|')
        .borders('|')
        .separators(&sep_pos, sep_chars)
        .padding(1, 1)
        .build();
    table.set_format(table_fmt);

    table.add_row(row![b -> "Name", info.name]);
    let status = match info.status {
        ProcessStatus::Running => "running".green().bold(),
        ProcessStatus::Stopped => "stopped".red().bold(),
    };
    table.add_row(row![b -> "Status", status]);
    let pid = match info.pid {
        Some(pid) => pid.to_string(),
        None => "none".to_string(),
    };
    table.add_row(row![b -> "PID", pid]);
    let (cmd, args) = info.cmd.split_first().unwrap();
    table.add_row(row![b -> "Command", format!("{:?}", cmd)]);
    table.add_row(row![b -> "Args", format!("{:?}", args)]);
    table.add_row(row![b -> "Created at", info.created_at.format("%Y-%m-%d %H:%M:%S")]);
    table.add_row(row![b -> "Working dir", info.cwd.display()]);
    table.add_row(row![b -> "PID file", info.pid_path.display()]);
    table.add_row(row![b -> "Output log file", info.stdout_path.display()]);
    table.add_row(row![b -> "Error log file", info.stderr_path.display()]);

    table.printstd();

    Ok(())
}
