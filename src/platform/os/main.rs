
#![ allow( warnings ) ]

use std::path::PathBuf;
use clap::{ArgAction, ArgMatches};
use clap::{Arg, crate_version, Command};

use mize::error::{MizeError, MizeResult};
use mize::item::ItemData;
use tokio::sync::{Mutex, mpsc};
use std::sync::Arc;
use std::io::Write;
use colored::Colorize;
use std::env;
use tracing_subscriber::fmt::Subscriber;
use tracing::Level;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::filter::EnvFilter;
use crate::logging::init_logger;

use tracing::{trace, debug, info, warn, error};

mod cli;
mod logging;

static APPNAME: &str = "mize";
static DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::WARN;

fn main() {
    // welcome to the mize source code
    // this is the main entry point for platforms with an os (eg: Linux, MacOS, Windows, BSD, ...)
    // Also let's try to make the most best documented code ever. (eventually) so it's fun to work on

    let cli_matches = cli_matches();

    init_logger(&cli_matches);

    // match command
    let result = match cli_matches.subcommand() {
        // mi daemon
        Some(("run", sub_matches)) => cli::run(sub_matches),

        Some(("is-running", sub_matches)) => cli::is_running(sub_matches),

        // mi serve
        Some(("serve", sub_matches)) => cli::stop(sub_matches),

        // mi mount
        Some(("mount", sub_matches)) => cli::mount(sub_matches),

        // mi get
        Some(("get", sub_matches)) => cli::get(sub_matches),

        // mi set
        Some(("set", sub_matches)) => cli::set(sub_matches),

        // mi show
        Some(("show", sub_matches)) => cli::show(sub_matches),

        // mi call
        Some(("call", sub_matches)) => cli::call(sub_matches),

        // mi create
        Some(("create", sub_matches)) => cli::create(sub_matches),

        // mi gui
        Some(("gui", sub_matches)) => cli::gui(sub_matches),

        // some unknown command passed
        Some((cmd, sub_matches)) => Err(MizeError::new().msg(format!("The subcommand: {} is not known. use --help to list availavle commands", cmd))),

        None => Err(MizeError::new().msg("No subcommand was passed. use --help to list availavle comamnds.")),
    };

    if let Err(err) = result {
        err.log();
    }
}


fn cli_matches() -> clap::ArgMatches {


    let main = Command::new(APPNAME)
        .version(crate_version!())
        .author("Sebastian Moser")
        .about("The MiZe Command line tool")
        .arg(Arg::new("verbose")
            .long("verbose")
            .short('v')
            .action(ArgAction::Count)
            .global(true)
        )
        .arg(Arg::new("log-level")
            .long("log-level")
            .value_name("LOGLEVEL")
            .help("set the log-level to one of OFF, ERROR, WARN, INFO, DEBUG, TRACE")
            .global(true)
        )
        .arg(Arg::new("silent")
            .long("silent")
            .action(ArgAction::SetTrue)
            .help("set the log-level to OFF")
            .global(true)
        )
        .arg(Arg::new("folder")
            .short('f')
            .long("folder")
            .help("The folder the Instance stores all it's data and the socket for connections")
            .global(true)
        )
        .arg(Arg::new("config")
            .short('c')
            .long("config")
            .help("overwrite config options")
            .global(true)
        )
        .arg(Arg::new("config-file")
            .long("config-file")
            .help("specify a config file")
            .global(true)
        )
        .subcommand(
                Command::new("run")
                .aliases(["r"])
                .about("Run a MiZe Instance")
            )
        .subcommand(Command::new("stop")
                .about("Stop a MiZe Instance")
            )
        .subcommand(
                Command::new("mount")
                .aliases(["m"])
            )
        .subcommand(
                Command::new("get")
                .aliases(["g"])
                .arg(Arg::new("id").help("The id or path to get"))
            .arg(Arg::new("recurse")
                .short('r')
                .action(ArgAction::Count)
                )
            )
        .subcommand(
                Command::new("set")
                .aliases(["s"])
                .arg(Arg::new("id").help("The id or path to set"))
                .arg(Arg::new("value").help("The value to set the path to"))
            )
        .subcommand(
                Command::new("show")
                .aliases(["so"])
                .arg(Arg::new("id").help("The id or path to sub to and show"))
            )
        .subcommand(
                Command::new("call")
                .aliases(["c"])
            )
        .subcommand(
                Command::new("create")
                .aliases(["cr"])
            )
        .subcommand(
                Command::new("is-running")
                .aliases(["isr"])
            )
        .subcommand(
                Command::new("gui")
            )
        .arg_required_else_help(true);

    return main.get_matches();
}


