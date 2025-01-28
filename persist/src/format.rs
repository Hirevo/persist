use std::fmt::Display;
use std::path::Path;

use clap::ValueEnum;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use tableau::Style;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ValueEnum)]
pub enum TableStyle {
    #[default]
    Ascii,
    Unicode,
}

impl Into<Style> for TableStyle {
    fn into(self) -> Style {
        match self {
            TableStyle::Ascii => Style::ascii(),
            TableStyle::Unicode => Style::rounded(),
        }
    }
}

pub fn error(err: impl Display) {
    eprintln!("{} {}", "error:".red().bold(), err);
}

pub fn success(msg: impl Display) {
    println!("{} {}", "success:".green().bold(), msg);
}

pub fn info(msg: impl Display) {
    println!("{} {}", "info:".blue().bold(), msg);
}

pub fn format_path(path: impl AsRef<Path>) -> String {
    let path = path.as_ref();
    if let Some(home_dir) = dirs_next::home_dir() {
        let mut components = path.components();
        let matches = home_dir
            .components()
            .zip(components.by_ref())
            .all(|(a, b)| a == b);

        if matches {
            let path = components.as_path();
            if path.file_name().is_none() {
                "~".to_string()
            } else {
                format!("~/{}", path.display())
            }
        } else {
            path.display().to_string()
        }
    } else {
        path.display().to_string()
    }
}
