use std::path::PathBuf;

use clap::ArgMatches;
use home::home_dir;
use std::fs::File;
use log::{trace, debug, info, warn, error};
use nix::sys::signal::{self, Signal};

use mize::error::{MizeResultTrait, IntoMizeResult, MizeError, MizeResult};
use mize::instance::Instance;


pub fn run(sub_matches: &ArgMatches) -> MizeResult<()> {

    // TODO NEXT!!!
    // - --config arg
    // - --folder
    // - to set a location, where there will be a /store and a /sock
    // - listen for connections on that sock
    // - msg handeling
    let instance = Instance::with_config(todo!());

    /*
    let instance_path = match sub_matches.get_one::<String>("store") {
        Some(a) => PathBuf::from(a),
        None => {
            let mut home_dir = home_dir()
            .ok_or(MizeError::new().category("io").category("env").msg("could not get the home directory, where the mize store is by default")).critical();
            home_dir.push(".mize");
            home_dir
        },
    };

    let instance = Instance::new(instance_path.clone()).await.critical();

    info!("Opened Instance at \"{}\"", instance_path.clone().display());

    match sub_matches.subcommand() {
        // stop the daemon
        Some(("stop", _)) => {
            if instance.daemon_is_running().await.critical() {
                //instance.stop_daemon();
                println!("Todo")
            } else {
                warn!("Daemon is already not running.");
            };
        },
        Some(("status", _)) => {
            if instance.daemon_is_running().await.critical() {
                eprintln!("Daemon is running");
            } else {
                eprintln!("Daemon is not running");
            };
        },

        Some(("kill", _)) => {
            signal::kill(instance.get_daemon_pid().await.critical(), Signal::SIGTERM).mize_result_msg("Could not kill daemon").critical();
        },

        Some((cmd, _)) => {
            error!("Unknown Command: {}", cmd);
        }

        // just start the daemon
        None => {
            if !instance.daemon_is_running().await.critical() {
                debug!("Daemon is not running. starting...");
                instance.start_deamon().await.critical();
                info!("Daemon Started");
            };
        },
    }
    */
    return Ok(());
}
