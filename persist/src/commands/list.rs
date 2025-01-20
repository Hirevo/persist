use clap::Parser;
use colored::Colorize;
use humansize::{format_size, DECIMAL};
use serde::{Deserialize, Serialize};
use term_table::row::Row;
use term_table::table_cell::{Alignment, TableCell};
use term_table::Table;

use persist_core::error::Error;
use persist_core::protocol::{ListRequest, ProcessStatus};

use crate::{daemon, GlobalOpts};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Parser)]
pub struct Opts {}

pub async fn handle(global_opts: GlobalOpts, _: Opts) -> Result<(), Error> {
    let mut daemon = daemon::connect().await?;
    let metrics = daemon.list(ListRequest { filters: None }).await?;

    let mut table = Table::builder()
        .style(global_opts.table_style.into())
        .build();

    let columns = ["NAME", "STATUS", "PID", "CPU", "MEMORY"];
    table.add_row(Row::new(columns.iter().map(|name| {
        TableCell::builder(name.bold())
            .alignment(Alignment::Left)
            .build()
    })));

    if metrics.is_empty() {
        table.set_max_width_for_column(0, 6);
        table.add_row(Row::new([TableCell::builder("Empty list.".bold())
            .alignment(Alignment::Center)
            .col_span(columns.len())
            .build()]));
    } else {
        let mut first = true;
        for metric in metrics {
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
            let new_row = if first {
                Row::new
            } else {
                Row::without_separator
            };
            table.add_row(new_row([name, status, pid, cpu_usage, mem_usage].map(
                |value| TableCell::builder(value).alignment(Alignment::Left).build(),
            )));
            first = false;
        }
    }

    println!("{}", table.render().trim());

    Ok(())
}
