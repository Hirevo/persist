use clap::Parser;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use tableau::{Cell, Row, Table};

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

    let status = match info.status {
        ProcessStatus::Running => "running".green().bold().to_string(),
        ProcessStatus::Stopped => "stopped".red().bold().to_string(),
    };
    let pid = match info.pid {
        Some(pid) => pid.to_string(),
        None => "none".to_string(),
    };
    let (cmd, args) = info.cmd.split_first().unwrap();

    let table = Table::new()
        .with_style(global_opts.table_style.into())
        .with_row(
            Row::new()
                .with_cell(Cell::new("Name".bold()))
                .with_cell(Cell::new(info.name)),
        )
        .with_row(
            Row::new()
                .without_top_border()
                .with_cell(Cell::new("Status".bold()))
                .with_cell(Cell::new(status)),
        )
        .with_row(
            Row::new()
                .without_top_border()
                .with_cell(Cell::new("PID".bold()))
                .with_cell(Cell::new(pid)),
        )
        .with_row(
            Row::new()
                .without_top_border()
                .with_cell(Cell::new("Command".bold()))
                .with_cell(Cell::new(format!("{:?}", cmd))),
        )
        .with_row(
            Row::new()
                .without_top_border()
                .with_cell(Cell::new("Args".bold()))
                .with_cell(Cell::new(format!("{:?}", args))),
        )
        .with_row(
            Row::new()
                .without_top_border()
                .with_cell(Cell::new("Created at".bold()))
                .with_cell(Cell::new(info.created_at.format("%Y-%m-%d %H:%M:%S"))),
        )
        .with_row(
            Row::new()
                .without_top_border()
                .with_cell(Cell::new("Working dir".bold()))
                .with_cell(Cell::new(info.cwd.display())),
        )
        .with_row(
            Row::new()
                .without_top_border()
                .with_cell(Cell::new("PID file".bold()))
                .with_cell(Cell::new(info.pid_path.display())),
        )
        .with_row(
            Row::new()
                .without_top_border()
                .with_cell(Cell::new("Output log file".bold()))
                .with_cell(Cell::new(info.stdout_path.display())),
        )
        .with_row(
            Row::new()
                .without_top_border()
                .with_cell(Cell::new("Error log file".bold()))
                .with_cell(Cell::new(info.stderr_path.display())),
        );

    println!("{}", table.render());

    Ok(())
}
