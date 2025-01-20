use clap::Parser;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use term_table::row::Row;
use term_table::Table;

use persist_core::error::Error;
use persist_core::protocol::{InfoRequest, InfoResponse, ProcessStatus};

use crate::{daemon, GlobalOpts};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Parser)]
pub struct Opts {
    /// The name of the process to get information about
    #[arg(name = "PROCESS_NAME")]
    pub name: String,
}

pub async fn handle(global_opts: GlobalOpts, opts: Opts) -> Result<(), Error> {
    let mut daemon = daemon::connect().await?;
    let InfoResponse { info } = daemon.info(InfoRequest { name: opts.name }).await?;

    let mut table = Table::builder()
        .style(global_opts.table_style.into())
        .build();

    let status = match info.status {
        ProcessStatus::Running => "running".green().bold().to_string(),
        ProcessStatus::Stopped => "stopped".red().bold().to_string(),
    };
    let pid = match info.pid {
        Some(pid) => pid.to_string(),
        None => "none".to_string(),
    };
    let (cmd, args) = info.cmd.split_first().unwrap();

    table.add_row(Row::new(["Name".bold().to_string(), info.name]));
    table.add_row(Row::without_separator([
        "Status".bold().to_string(),
        status,
    ]));
    table.add_row(Row::without_separator(["PID".bold().to_string(), pid]));
    table.add_row(Row::without_separator([
        "Command".bold().to_string(),
        format!("{:?}", cmd),
    ]));
    table.add_row(Row::without_separator([
        "Args".bold().to_string(),
        format!("{:?}", args),
    ]));
    table.add_row(Row::without_separator([
        "Created at".bold().to_string(),
        info.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
    ]));
    table.add_row(Row::without_separator([
        "Working dir".bold().to_string(),
        info.cwd.display().to_string(),
    ]));
    table.add_row(Row::without_separator([
        "PID file".bold().to_string(),
        info.pid_path.display().to_string(),
    ]));
    table.add_row(Row::without_separator([
        "Output log file".bold().to_string(),
        info.stdout_path.display().to_string(),
    ]));
    table.add_row(Row::without_separator([
        "Error log file".bold().to_string(),
        info.stderr_path.display().to_string(),
    ]));

    println!("{}", table.render().trim());

    Ok(())
}
