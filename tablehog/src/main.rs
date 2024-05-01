use anyhow::Context;
use std::ops::Sub;
use tablehog::*;
use time::OffsetDateTime;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {    

    run().await?;

    Ok(())
}
