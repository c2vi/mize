use clap::ArgMatches;
use crossbeam::channel::{bounded, unbounded};
use mize::error::{MizeResult, MizeError};
use mize::instance::subscription::{Subscription, Update};
use mize::instance::Instance;
use mize::platform::os::config_from_cli_args;

pub fn show(sub_matches: &ArgMatches) -> MizeResult<()> {

    let instance = Instance::with_config(config_from_cli_args(sub_matches)?)?;

    let id = sub_matches.get_one::<String>("id")
        .ok_or(MizeError::new().msg("No id Argument specified"))?;

    let item = instance.get(id)?;

    let (tx, rx) = bounded::<Update>(4);
    let sub = Subscription::from_sender(tx);
    instance.sub(id, sub)?;

    println!("ItemData: {}", item.as_data_full()?);

    for update in rx {
        println!("####### GOT UPDATE #######");
        println!("ItemData: {}", update.new_item()?.as_data_full()?);
    }

    Ok(())
}
