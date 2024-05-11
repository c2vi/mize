
#![ allow( warnings ) ]

use std::path::PathBuf;
use clap::{ArgAction, ArgMatches};
use clap::{Arg, crate_version, Command};

use tokio::sync::{Mutex, mpsc};
use std::sync::Arc;
use log::Level;
use log::LevelFilter;
use env_logger::{Builder, Env};
use std::io::Write;
use colored::Colorize;
use std::env;

use log::{trace, debug, info, warn, error};

mod cli {
    pub mod daemon;
    pub mod serve;
    pub mod get;
    pub mod set;
    pub mod mount;
    pub mod print;
    pub mod call;

    pub use daemon::daemon;
    pub use serve::serve;
    pub use get::get;
    pub use set::set;
    pub use mount::mount;
    pub use print::print;
    pub use call::call;
}

mod error;
// mod types;
mod instance;
mod item;
mod itemstore;
mod proto;

#[cfg(test)]
mod test;

static APPNAME: &str = "mize";
static DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::Warn;

#[tokio::main]
async fn main() {
    // welcome to the mize source code
    // this is the main entry point for platforms with an os (eg: Linux, MacOS, Windows, BSD, ...)
    // Also let's try to make the most best documented code ever. (eventually) so it's fun to work on

    let cli_matches = cli_matches();

    init_logger(&cli_matches);

    // log a message of every level, to test loging
    // test_logger();


    // match command
    match cli_matches.subcommand() {
        // mi daemon
        Some(("daemon", sub_matches)) => cli::daemon(sub_matches).await,

        // mi serve
        Some(("serve", sub_matches)) => cli::serve(sub_matches).await,

        // mi mount
        Some(("mount", sub_matches)) => cli::mount(sub_matches).await,

        // mi get
        Some(("get", sub_matches)) => cli::get(sub_matches).await,

        // mi set
        Some(("set", sub_matches)) => cli::set(sub_matches).await,

        // mi sub
        Some(("sub", sub_matches)) => cli::sub(sub_matches).await,

        // mi call
        Some(("call", sub_matches)) => cli::call(sub_matches).await,

        // some unknown command passed
        Some((cmd, sub_matches)) => error!("The subcommand: {} is not known. use --help to list availavle commands", cmd),

        None => error!("No subcommand was passed. use --help to list availavle comamnds."),
    }


}

fn init_logger(cli_matches: &ArgMatches) {

    // to save log messages, before the logger is setup
    let mut log_messages: Vec<(Level, String)> = Vec::new();

    log_messages.push((Level::Info, "Starting Mize".to_owned()));

    // builder with DEFAULT_LOG_LEVEL
    let mut builder = Builder::new();
    builder.format_indent(Some(8));



    // set loglevel
    let mut log_level = DEFAULT_LOG_LEVEL;
    log_messages.push((Level::Trace, format!("set log level from DEFAULT_LOG_LEVEL to {}", DEFAULT_LOG_LEVEL)));

    // check env RUST_LOG
    let mut val_rust_log = level_from_env("RUST_LOG", log_level);
    if let Some(level_rust_log) = val_rust_log.0 {
        log_level = level_rust_log;
        log_messages.push((Level::Trace, format!("set log level from the Variable RUST_LOG to {}", level_rust_log)));
    }
    log_messages.append(&mut val_rust_log.1);

    // check env MIZE_LOG
    let mut val_mize_log = level_from_env("MIZE_LOG", log_level);
    if let Some(level_mize_log) = val_mize_log.0 {
        log_level = level_mize_log;
        log_messages.push((Level::Trace, format!("set log level from the Variable MIZE_LOG to {}", level_mize_log)));
    }
    log_messages.append(&mut val_mize_log.1);

    // check -vvvvv flags
    let v_flag_count = cli_matches.get_count("verbose");
    let default_log_level_as_num = match DEFAULT_LOG_LEVEL {
        LevelFilter::Off => 0,
        LevelFilter::Error => 1,
        LevelFilter::Warn => 2,
        LevelFilter::Info => 3,
        LevelFilter::Debug => 4,
        LevelFilter::Trace => 5,
    };
    let new_log_level_num = default_log_level_as_num + v_flag_count;
    log_level = match new_log_level_num {
        0 => LevelFilter::Off,
        1 => LevelFilter::Error,
        2 => LevelFilter::Warn,
        3 => LevelFilter::Info,
        4 => LevelFilter::Debug,
        5 => LevelFilter::Trace,
        _ => {
            log_messages.push((Level::Trace, format!("the number of verbose (-v) flags passed ({}) exeeds the maximumg log_level", v_flag_count)));
            LevelFilter::Trace
        },
    };
    log_messages.push((Level::Trace, format!("set log level from the verbose flags to {}", log_level)));
 
    // check --log-level arg
    if let Some(log_level_arg_string) = cli_matches.get_one::<&str>("log-level") {
        let log_level_from_arg = match log_level_arg_string.to_lowercase().as_str() {
            "none" => LevelFilter::Off,
            "trace" => LevelFilter::Trace,
            "debug" => LevelFilter::Debug,
            "info" => LevelFilter::Info,
            "warn" => LevelFilter::Warn,
            "error" => LevelFilter::Error,
            _ => {
                log_messages.push((Level::Warn, format!(
                        "The argument to --log-level \"{}\" is not a valid log-level. loglevel stays at: {}"
                        , log_level_arg_string
                        , log_level
                    )));
                log_level
            }
        };
        log_level = log_level_from_arg;
        log_messages.push((Level::Trace, format!("set log level from the --log-level arg to {}", log_level_from_arg)));
    }

    // check --silent flag
    if cli_matches.get_flag("silent") {
        if log_level != DEFAULT_LOG_LEVEL {
            log_messages.push((Level::Warn, format!(
                    "The loglevel was set by some method (it's not the DEFAULT_LOG_LEVEL) and is now overwritten by the --siltne to be OFF"
                )))
        }
        log_level = LevelFilter::Off;
    }

    // actually set the log_level on the builder
    builder.filter_level(log_level);
    log_messages.push((Level::Info, format!("loglevel is now {}", log_level)));


    // set format
    builder.format(|buf, record| {
            match record.level() {
                Level::Trace => writeln!(buf, "[ {} ] {}", format!("{}", record.level()).yellow(), record.args()),
                Level::Debug => writeln!(buf, "[ {} ] {}", format!("{}", record.level()).green(), record.args()),
                Level::Info => writeln!(buf, "[ {}  ] {}", format!("{}", record.level()).blue(), record.args()),
                Level::Warn => writeln!(buf, "[ {}  ] {}", format!("{}", record.level()).truecolor(245, 164, 66), record.args()),
                Level::Error => writeln!(buf, "[ {} ] {}", format!("{}", record.level()).red(), record.args()),
            }
        });

    builder.init();


    // write out all log messages from log_messages
    for (level, msg) in log_messages {
        match level {
            Level::Error => error!("{}", msg),
            Level::Warn => warn!("{}", msg),
            Level::Info => info!("{}", msg),
            Level::Debug => debug!("{}", msg),
            Level::Trace => trace!("{}", msg),
        }
    }
}

fn test_logger() {
    error!("hi from error");
    warn!("hi from warn");
    info!("hi from info");
    debug!("hi from debug");
    trace!("hi from trace");
}

fn level_from_env(var_name: &str, log_level: LevelFilter) -> (Option<log::LevelFilter>, Vec<(Level, String)>) {
    let mut log_messages: Vec<(Level, String)> = Vec::new();
    let var_value = env::var(var_name);
    match var_value {
        Ok(val) => {
            // set loglevel acording to RUST_LOG
            let level = match val.to_lowercase().as_str() {
                "none" => LevelFilter::Off,
                "trace" => LevelFilter::Trace,
                "debug" => LevelFilter::Debug,
                "info" => LevelFilter::Info,
                "warn" => LevelFilter::Warn,
                "error" => LevelFilter::Error,
                _ => {
                    log_messages.push((Level::Warn, format!(
                            "The value of Environment Variable {}: \"{}\" is not a valid log-level. loglevel stays at: {}"
                            , var_name
                            , val
                            , log_level
                        )));
                    log_level
                }
            };
            return (Some(level), log_messages)
        },
        Err(env::VarError::NotPresent) => {},
        Err(env::VarError::NotUnicode(os_string)) => {

            let os_string_as_hex : String = os_string.clone().into_encoded_bytes().iter()
              .map(|b| format!("{:02x}", b).to_string())
              .collect::<Vec<String>>()
              .join("");
            log_messages.push((Level::Debug, format!(
                "value of Environment Variable {} was not a unicode valid string. Therefore ignoring it.
                \n value with replacement char U+FFFD: {}
                \n the WTF-8 raw bytes as hex: {}"
                , var_name
                , os_string.to_string_lossy(), os_string_as_hex
            )));
        },
    };
    return (None, log_messages);
}

fn cli_matches() -> clap::ArgMatches {

    let store_arg = Arg::new("store")
        .short('s')
        .long("store")
        .help("The folder used as the Itemstore");

    let main = Command::new(APPNAME)
        .version(crate_version!())
        .author("Sebastian Moser")
        .about("The MiZe Command line tool")
        .arg(Arg::new("verbose")
            .long("verbose")
            .short('v')
            .action(ArgAction::Count)
        )
        .arg(Arg::new("log-level")
            .long("log-level")
            .value_name("LOGLEVEL")
            .help("set the log-level to one of OFF, ERROR, WARN, INFO, DEBUG, TRACE")
        )
        .arg(Arg::new("silent")
            .long("silent")
            .action(ArgAction::SetTrue)
            .help("set the log-level to OFF")
        )
        .subcommand(
                Command::new("daemon")
                .aliases(["da"])
                .arg(&store_arg)
                .about("Starts the mize daemon. Has subcommands to stop, etc")
                .subcommand(Command::new("stop")
                        .about("Stops the mize daemon if one is running")
                    )
                .subcommand(Command::new("kill"))
                        .about("Stops the mize daemon with a signal")
                        .alias("k")
                .subcommand(Command::new("status")
                        .alias("st")
                    )
            )
        .subcommand(
                Command::new("serve")
                .aliases(["se"])
                .arg(&store_arg)
            )
        .subcommand(
                Command::new("mount")
                .aliases(["m"])
                .arg(&store_arg)
            )
        .subcommand(
                Command::new("get")
                .aliases(["g"])
                .arg(&store_arg)
                .arg(Arg::new("id").help("The id or path to get"))
            )
        .subcommand(
                Command::new("set")
                .aliases(["s"])
                .arg(&store_arg)
                .arg(Arg::new("id").help("The id or path to set"))
            )
        .subcommand(
                Command::new("sub")
                .aliases(["su"])
                .arg(&store_arg)
                .arg(Arg::new("id").help("The id or path to sub to and show"))
            )
        .subcommand(
                Command::new("call")
                .aliases(["c"])
                .arg(&store_arg)
            )
        .arg_required_else_help(true);

    return main.get_matches();
}


