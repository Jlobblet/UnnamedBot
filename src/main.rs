use eyre::Result;
use try_traits::default::TryDefault;
use crate::config::Config;

mod config;

fn main() -> Result<()> {
    let cfg = Config::try_default()?;
    dbg!(cfg);
    Ok(())
}
