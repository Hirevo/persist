use std::fmt::Display;

use colored::Colorize;

pub fn error(err: impl Display) {
    eprintln!("{} {}", "error:".red().bold(), err);
}

pub fn success(msg: impl Display) {
    println!("{} {}", "success:".green().bold(), msg);
}

pub fn info(msg: impl Display) {
    println!("{} {}", "info:".blue().bold(), msg);
}
