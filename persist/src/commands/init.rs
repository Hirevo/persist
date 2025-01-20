use clap::Parser;
use serde::{Deserialize, Serialize};

use persist_core::error::Error;

use crate::{daemon, GlobalOpts};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Parser)]
pub struct Opts {}

pub async fn handle(_: GlobalOpts, _: Opts) -> Result<(), Error> {
    daemon::init().await?;
    Ok(())
}
