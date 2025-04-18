use std::ffi::OsString;
use std::path::PathBuf;
use std::str::FromStr;
use clap::ArgMatches;
use home::home_dir;
use flume::bounded;
use mize::mize_err;
use std::io::Read;
use tracing::debug;

use mize::error::{IntoMizeResult, MizeError, MizeResult, MizeResultTrait};
use mize::instance::Instance;
use mize::platform::os::config_from_cli_args;
use mize::instance::subscription::Update;
use mize::platform::os::fsstore::FileStore;
use mize::instance::subscription::Subscription;
use mize::item::{IntoItemData, ItemData};



pub fn get(sub_matches: &ArgMatches) -> MizeResult<()> {

    let instance = Instance::with_config(config_from_cli_args(sub_matches)?)?;

    let id = sub_matches.get_one::<String>("id")
        .ok_or(MizeError::new().msg("No id Argument specified"))?;

    let item = instance.get(id)?;

    let recurse_count = sub_matches.get_count("recurse");

    println!("{}", item.as_data_full()?);
    return Ok(());

    if recurse_count > 0 {
        println!("{}", item.id());
        println!("{}", item.as_data_full()?);
    } else {
        println!("{}", item);
    };

    return Ok(());
}


pub fn call(sub_matches: &ArgMatches) -> MizeResult<()> {
    Ok(())
}


pub fn create(sub_matches: &ArgMatches) -> MizeResult<()> {

    let mut instance = Instance::with_config(config_from_cli_args(sub_matches)?)?;

    let item = instance.new_item()?;

    println!("id: {}", item.id());
    println!("with namespace: {}", item.id().namespace().as_real_string());

    return Ok(());
}


pub fn is_running(sub_matches: &ArgMatches) -> MizeResult<()> {

    let home_dir = env!("HOME");

    if FileStore::store_is_opened(home_dir.to_owned() + "/.mize")? {
        println!("true");
    } else {
        println!("false");
    }

    Ok(())
 
}


pub fn mount(sub_matches: &ArgMatches) -> MizeResult<()> {
    Ok(())
}


pub fn run(sub_matches: &ArgMatches) -> MizeResult<()> {

    let instance = Instance::with_config(config_from_cli_args(sub_matches)?)?;

    instance.wait();

    return Ok(());
}


pub fn set(sub_matches: &ArgMatches) -> MizeResult<()> {

    let instance = Instance::with_config(config_from_cli_args(sub_matches)?)?;

    let id = sub_matches.get_one::<String>("id")
        .ok_or(MizeError::new().msg("No id Argument specified"))?;

    let value = sub_matches.get_one::<String>("value")
        .ok_or(MizeError::new().msg("No value Argument specified"))?;

    instance.set_blocking(id, value.into_item_data())?;

    Ok(())
}


pub fn show(sub_matches: &ArgMatches) -> MizeResult<()> {

    let instance = Instance::with_config(config_from_cli_args(sub_matches)?)?;

    let id = sub_matches.get_one::<String>("id")
        .ok_or(MizeError::new().msg("No id Argument specified"))?;

    let item = instance.get(id)?;

    let (tx, rx) = bounded::<Update>(4);
    let sub = Subscription::from_sender(tx);
    instance.sub(id, sub)?;

    println!("item: {}", item.as_data_full()?);

    for update in rx {
        println!("####### GOT UPDATE #######");
        println!("item: {}", update.new_item()?.as_data_full()?);
    }

    Ok(())
}


pub fn stop(sub_matches: &ArgMatches) -> MizeResult<()> {
    Ok(())
}


pub fn gui(sub_matches: &ArgMatches) -> MizeResult<()> {
    let mut instance = Instance::with_config(config_from_cli_args(sub_matches)?)?;

    instance.load_module("mme")?;

    instance.wait();

    Ok(())
}

pub fn format_cbor(sub_matches: &ArgMatches) -> MizeResult<()> {

    let mut input: Vec<u8> =  Vec::new();
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();
    //handle.read_to_end(&mut input);

    let cbor_data = ciborium::from_reader(handle)?;
    let item_data = ItemData::from_cbor(cbor_data);

    println!("{}", item_data);

    Ok(())
}


pub fn run_from_module(cmd: &str, sub_matches: &ArgMatches) -> MizeResult<()> {
    let default = OsString::default();
    let mut sub_cmd_line: Vec<OsString> = sub_matches.get_many::<OsString>("").unwrap().map(|e| e.to_owned()).collect();
    sub_cmd_line.insert(0, OsString::from_str(cmd)?);

    let mut instance = Instance::with_config(config_from_cli_args(sub_matches)?)?;

    debug!("loading module '{}'", cmd);
    instance.load_module(cmd)?;

    instance.get_module(cmd)?.run_cli(&instance, sub_cmd_line)
        .ok_or(mize_err!("The Module '{cmd}' does not implement the run_cli() functionality"))??;

    Ok(())
}

