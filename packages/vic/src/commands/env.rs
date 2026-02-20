use clap::ArgMatches;

use crate::error::VicResult;

pub fn main(matches: &ArgMatches) -> VicResult<()> {
    println!("This subcommand is not yet implemented");
    Ok(())
}

pub fn get(matches: &ArgMatches) -> VicResult<()> {

    println!("system: {}", current_platform::CURRENT_PLATFORM);

    Ok(())
}
