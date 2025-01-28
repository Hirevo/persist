use clap::Parser;
use colored::Colorize;
use humansize::{format_size, DECIMAL};
use serde::{Deserialize, Serialize};
use tableau::{Alignment, Cell, Row, Table};

use persist_core::error::Error;
use persist_core::protocol::{ListRequest, ProcessStatus};

use crate::{daemon, GlobalOpts};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Parser)]
pub struct Opts {}

pub async fn handle(global_opts: GlobalOpts, _: Opts) -> Result<(), Error> {
    let mut daemon = daemon::connect().await?;
    let metrics = daemon.list(ListRequest { filters: None }).await?;

    let columns = ["NAME", "STATUS", "PID", "CPU", "MEMORY"];
    let mut table = Table::new()
        .with_style(global_opts.table_style.into())
        .with_row(Row::new().with_cells(columns.iter().map(|name| Cell::new(name.bold()))));

    if metrics.is_empty() {
        table.rows.push(
            Row::new().with_cell(
                Cell::new("Empty list.".bold())
                    .with_alignment(Alignment::Center)
                    .with_column_span(columns.len()),
            ),
        );
    } else {
        for (index, metric) in metrics.into_iter().enumerate() {
            let name = metric.name;
            let status = match metric.status {
                ProcessStatus::Running => "running".green().bold().to_string(),
                ProcessStatus::Stopped => "stopped".red().bold().to_string(),
            };
            let cpu_usage = match metric.status {
                ProcessStatus::Running => format!("{} %", metric.cpu_usage),
                ProcessStatus::Stopped => "".to_string(),
            };
            let mem_usage = match metric.status {
                ProcessStatus::Running => format_size(metric.mem_usage, DECIMAL),
                ProcessStatus::Stopped => "".to_string(),
            };
            let pid = match metric.pid {
                Some(pid) => pid.to_string(),
                None => "".to_string(),
            };

            let row = Row::new().with_cells(
                [name, status, pid, cpu_usage, mem_usage].map(|value| Cell::new(value)),
            );

            table.rows.push(if index == 0 {
                row
            } else {
                row.without_top_border()
            });
        }
    }

    println!("{}", table.render());

    Ok(())
}
