use clap::ArgMatches;
use mize::error::MizeResult;
use mize::instance::Instance;

pub fn show(sub_matches: &ArgMatches) -> MizeResult<()> {

    let instance = Instance::new()?;

    println!("0/config/test is: {}", instance.get("0/config/test")?);

    Ok(())
}
