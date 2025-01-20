use std::fmt::Display;
use std::path::Path;

use clap::ValueEnum;
use colored::Colorize;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ValueEnum)]
pub enum TableStyle {
    #[default]
    Ascii,
    Unicode,
}

impl Into<term_table::TableStyle> for TableStyle {
    fn into(self) -> term_table::TableStyle {
        match self {
            TableStyle::Ascii => term_table::TableStyle {
                top_left_corner: '+',
                top_right_corner: '+',
                bottom_left_corner: '+',
                bottom_right_corner: '+',
                outer_left_vertical: '+',
                outer_right_vertical: '+',
                outer_bottom_horizontal: '+',
                outer_top_horizontal: '+',
                intersection: '+',
                vertical: '|',
                horizontal: '-',
            },
            TableStyle::Unicode => term_table::TableStyle {
                top_left_corner: '╭',
                top_right_corner: '╮',
                bottom_left_corner: '╰',
                bottom_right_corner: '╯',
                outer_left_vertical: '├',
                outer_right_vertical: '┤',
                outer_bottom_horizontal: '┴',
                outer_top_horizontal: '┬',
                intersection: '┼',
                vertical: '│',
                horizontal: '─',
            },
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
