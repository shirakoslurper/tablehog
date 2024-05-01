use anyhow::Context;
use std::ops::Sub;
use tablehog::*;
use time::OffsetDateTime;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {    

    unsafe {
        time::util::local_offset::set_soundness(time::util::local_offset::Soundness::Unsound);
    }

    run().await?;

    Ok(())
}
