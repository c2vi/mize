
#![ allow( warnings ) ]

use std::env;
use clap::{ArgAction, ArgMatches};
use clap::{Arg, crate_version, Command};
use log::Level;
use log::LevelFilter;
use env_logger::Builder;
use std::io::Write;
use colored::Colorize;
use log::{trace, debug, info, warn, error};
use std::ffi::{c_char, CStr};

mod error;
mod utils;
mod victor;
mod eval;

mod commands {
    pub mod info;
    pub mod stat;
    pub mod run;
    pub mod build;
    pub mod list;
    pub mod gui;
    pub mod env;
    pub mod snix_testing;
}

static APPNAME: &str = "vic";
static DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::Warn;

fn main() {

    let cli_matches = cli_matches();

    //init_logger(&cli_matches);

    // log a message of every level, to test loging
    // test_logger();

    // match the subcommands
    let result = match cli_matches.subcommand() {
        Some(("info", sub_matches)) => commands::info::main(&sub_matches),
        Some(("stat", sub_matches)) => commands::stat::main(&sub_matches),
        Some(("run", sub_matches)) => commands::run::main(&sub_matches),
        Some(("build", sub_matches)) => commands::build::main(&sub_matches),
        Some(("list", sub_matches)) => commands::list::main(&sub_matches),
        Some(("gui", sub_matches)) => commands::gui::main(&sub_matches),

        Some(("env", sub_matches)) => commands::env::main(&sub_matches),
        Some(("env-get", sub_matches)) => commands::env::get(&sub_matches),

        Some(("snix-mount", sub_matches)) => commands::snix_testing::run_mount_blocking(&sub_matches),
        Some(("snix-test", sub_matches)) => commands::snix_testing::run_snix_test(&sub_matches),
        Some(("snix-build", sub_matches)) => commands::snix_testing::run_build(&sub_matches),

        None => { println!("Hi, this is victorinix (also known as victor or vic). \nrun 'vic help' for a list of subcommands."); Ok(())},
        Some((unknown_subcommand, _)) => { println!("The subcommand '{}' is not known to victorinix", unknown_subcommand); Ok(()) },
    };

    if let Err(vic_err) = result {
        vic_err.log();
    }
}



fn test() {
    let c_buf: *const c_char = unsafe { libelf::raw::elf_errmsg(-1) };
    let c_str: &CStr = unsafe { CStr::from_ptr(c_buf) };
    let str_slice: &str = c_str.to_str().unwrap();
    println!("ERROR: {}", str_slice);

}

fn init_logger(cli_matches: &ArgMatches) {

    // to save log messages, before the logger is setup
    let mut log_messages: Vec<(Level, String)> = Vec::new();

    log_messages.push((Level::Trace, "Starting victorinix".to_owned()));

    // builder with DEFAULT_LOG_LEVEL
    let mut builder = Builder::new();
    builder.format_indent(Some(8));



    // set loglevel
    let mut log_level = DEFAULT_LOG_LEVEL;
    log_messages.push((Level::Trace, format!("set log level to {} as specified by DEFAULT_LOG_LEVEL in the src/main.rs", DEFAULT_LOG_LEVEL)));

    // check env RUST_LOG
    let mut val_rust_log = level_from_env("RUST_LOG", log_level);
    log_level = if let Some(level_rust_log) = val_rust_log.0 {
        log_messages.push((Level::Trace, format!("set log level from the Variable RUST_LOG to {}", level_rust_log)));
        level_rust_log
    } else {log_level};
    log_messages.append(&mut val_rust_log.1);

    // check env VICTORINIX_LOG
    let mut val_victorinix_log = level_from_env("VICTORINIX_LOG", log_level);
    log_level = if let Some(level_victorinix_log) = val_victorinix_log.0 {
        log_messages.push((Level::Trace, format!("set log level from the Variable VICTORINIX_LOG to {}", level_victorinix_log)));
        level_victorinix_log
    } else {log_level};
    log_messages.append(&mut val_victorinix_log.1);


    // check -vvvvv flags
    let v_flag_count = cli_matches.get_count("verbose");
    let old_log_level_as_num = match log_level {
        LevelFilter::Off => 0,
        LevelFilter::Error => 1,
        LevelFilter::Warn => 2,
        LevelFilter::Info => 3,
        LevelFilter::Debug => 4,
        LevelFilter::Trace => 5,
    };
    let new_log_level_num = old_log_level_as_num + v_flag_count;

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
    if let Some(log_level_arg_string) = cli_matches.get_one::<String>("log-level") {
        let log_level_from_arg = match log_level_arg_string.to_lowercase().as_str() {
            "none" => LevelFilter::Off,
            "trace" => LevelFilter::Trace,
            "all" => LevelFilter::Trace,
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

    let main = Command::new(APPNAME)
        .version(crate_version!())
        .author("Sebastian Moser (c2vi)")
        .about("Victorinix: Friendly and usefull box of portable computer tools. Powered by Nix and MiZe")
        .arg_required_else_help(true)
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
        .subcommand(Command::new("info")
        )
        .subcommand(Command::new("stat")
        )
        .subcommand(Command::new("run")
            .arg(Arg::new("runnable")
                .help("The item to be run")
                .action(clap::ArgAction::Append)
                .trailing_var_arg(true)
            )
            //.arg(Arg::new("rest")
                //.help("args to pass down to the runnable command")
            //)
        )
        .subcommand(Command::new("build")
        )
        .subcommand(Command::new("list")
        )
        .subcommand(Command::new("gui")
        )

        .subcommand(Command::new("env")
        )
        .subcommand(Command::new("env-get")
            .alias("eg")
        )

        .subcommand(Command::new("snix-mount")
            .alias("sm")
        )
        .subcommand(Command::new("snix-test")
            .alias("st")
        )
        .subcommand(Command::new("snix-build")
            .alias("sb")
        )
        ;

    return main.get_matches();
}
