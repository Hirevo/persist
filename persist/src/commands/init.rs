use serde::{Deserialize, Serialize};
use structopt::StructOpt;

use persist_core::error::Error;

use crate::daemon;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, StructOpt)]
pub struct Opts {}

pub async fn handle(_: Opts) -> Result<(), Error> {
    daemon::init().await?;
    Ok(())
}
