use std::path::PathBuf;
use clap::{ArgAction, ArgMatches};
use clap::{Arg, crate_version, Command};

use std::sync::Arc;
use std::io::Write;
use colored::Colorize;
use std::env;
use tracing_subscriber::fmt::Subscriber;
use tracing::Level;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::fmt::FormatEvent;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::fmt::FormatFields;
use tracing_subscriber::fmt::FmtContext;
use tracing_subscriber::fmt::format;
use tracing::Event;
use core::fmt;
use tracing_subscriber::fmt::FormattedFields;

use tracing::{trace, debug, info, warn, error};

use crate::DEFAULT_LOG_LEVEL;

pub fn init_logger(cli_matches: &ArgMatches) {

    // to save log messages, before the logger is setup
    let mut log_messages: Vec<(Level, String)> = Vec::new();

    log_messages.push((Level::INFO, "Starting MME".to_owned()));

    // builder with DEFAULT_LOG_LEVEL
    let mut builder = tracing_subscriber::fmt::Subscriber::builder();

    // set loglevel
    let mut log_level = DEFAULT_LOG_LEVEL;
    log_messages.push((Level::TRACE, format!("set log level from DEFAULT_LOG_LEVEL to {}", DEFAULT_LOG_LEVEL)));

    // check env RUST_LOG
    let mut val_rust_log = level_from_env("RUST_LOG", log_level);
    if let Some(level_rust_log) = val_rust_log.0 {
        log_level = level_rust_log;
        log_messages.push((Level::TRACE, format!("set log level from the Variable RUST_LOG to {}", level_rust_log)));
    }
    log_messages.append(&mut val_rust_log.1);

    // check env MME_LOG
    let mut val_mme_log = level_from_env("MME_LOG", log_level);
    if let Some(level_mme_log) = val_mme_log.0 {
        log_level = level_mme_log;
        log_messages.push((Level::TRACE, format!("set log level from the Variable MME_LOG to {}", level_mme_log)));
    }
    log_messages.append(&mut val_mme_log.1);

    // check -vvvvv flags
    let v_flag_count = cli_matches.get_count("verbose");
    let default_log_level_as_num = match DEFAULT_LOG_LEVEL {
        LevelFilter::OFF => 0,
        LevelFilter::ERROR => 1,
        LevelFilter::WARN => 2,
        LevelFilter::INFO => 3,
        LevelFilter::DEBUG => 4,
        LevelFilter::TRACE => 5,
    };
    let new_log_level_num = default_log_level_as_num + v_flag_count;
    log_level = match new_log_level_num {
        0 => LevelFilter::OFF,
        1 => LevelFilter::ERROR,
        2 => LevelFilter::WARN,
        3 => LevelFilter::INFO,
        4 => LevelFilter::DEBUG,
        5 => LevelFilter::TRACE,
        _ => {
            log_messages.push((Level::TRACE, format!("the number of verbose (-v) flags passed ({}) exeeds the maximumg log_level", v_flag_count)));
            LevelFilter::TRACE
        },
    };
    log_messages.push((Level::TRACE, format!("set log level from the verbose flags to {}", log_level)));
 
    // check --log-level arg
    if let Some(log_level_arg_string) = cli_matches.get_one::<String>("log-level") {
        let log_level_from_arg = match log_level_arg_string.to_lowercase().as_str() {
            "none" => LevelFilter::OFF,
            "trace" => LevelFilter::TRACE,
            "debug" => LevelFilter::DEBUG,
            "info" => LevelFilter::INFO,
            "warn" => LevelFilter::WARN,
            "error" => LevelFilter::ERROR,
            _ => {
                log_messages.push((Level::WARN, format!(
                        "The argument to --log-level \"{}\" is not a valid log-level. loglevel stays at: {}"
                        , log_level_arg_string
                        , log_level
                    )));
                log_level
            }
        };
        log_level = log_level_from_arg;
        log_messages.push((Level::TRACE, format!("set log level from the --log-level arg to {}", log_level_from_arg)));
    }

    // check --silent flag
    if cli_matches.get_flag("silent") {
        if log_level != DEFAULT_LOG_LEVEL {
            log_messages.push((Level::WARN, format!(
                    "The loglevel was set by some method (it's not the DEFAULT_LOG_LEVEL) and is now overwritten by the --siltne to be OFF"
                )))
        }
        log_level = LevelFilter::OFF;
    }

    // actually set the log_level on the builder
    log_messages.push((Level::INFO, format!("loglevel is now {}", log_level)));

    // filter loging
    let mut my_env_filter = tracing_subscriber::filter::EnvFilter::new("")
        .add_directive("[get_raw_from_cbor]=debug".parse().expect("error parsing log directive"))
        //.add_directive("[MemStore::get_value_raw]=trace".parse().expect("error parsing log directive"))
        .add_directive("trace".parse().unwrap())
        .add_directive("[get_raw_from_cbor]=debug".parse().expect("error parsing log directive"))
    ;

    builder
        .with_env_filter(my_env_filter)
        .without_time()
        .event_format(MyFormatter)
        //.with_max_level(log_level)
        .init();


    // write out all log messages from log_messages
    for (level, msg) in log_messages {
        match level {
            Level::ERROR => error!("{}", msg),
            Level::WARN => warn!("{}", msg),
            Level::INFO => info!("{}", msg),
            Level::DEBUG => debug!("{}", msg),
            Level::TRACE => trace!("{}", msg),
        }
    }
}

fn level_from_env(var_name: &str, log_level: LevelFilter) -> (Option<LevelFilter>, Vec<(Level, String)>) {
    let mut log_messages: Vec<(Level, String)> = Vec::new();
    let var_value = env::var(var_name);
    match var_value {
        Ok(val) => {
            // set loglevel acording to RUST_LOG
            let level = match val.to_lowercase().as_str() {
                "none" => LevelFilter::OFF,
                "trace" => LevelFilter::TRACE,
                "debug" => LevelFilter::DEBUG,
                "info" => LevelFilter::INFO,
                "warn" => LevelFilter::WARN,
                "error" => LevelFilter::ERROR,
                _ => {
                    log_messages.push((Level::WARN, format!(
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
            log_messages.push((Level::DEBUG, format!(
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


struct MyFormatter;

impl<S, N> FormatEvent<S, N> for MyFormatter
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: format::Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        // Format values from the event's's metadata:

        let metadata = event.metadata();

    // set format
    /*
    builder.format(|buf, record| {
            match record.level() {
            }
        });
    */
        let level_str = match metadata.level() {
            &Level::TRACE => format!("[ {} ]", "TRACE".yellow()),
            &Level::DEBUG => format!("[ {} ]", "DEBUG".green()),
            &Level::INFO => format!("[ {}  ]", "INFO".blue()),
            &Level::WARN => format!("[ {}  ]", "WARN".truecolor(245, 164, 66)),
            &Level::ERROR => format!("[ {} ]", "ERROR".red()),
        };

        // with target
        // write!(&mut writer, "{} {}: ", level_str, metadata.target())?;

        write!(&mut writer, "{} ", level_str)?;

        // Format all the spans in the event's span context.
        if let Some(scope) = ctx.event_scope() {
            for span in scope.from_root() {
                write!(writer, "{}", span.name())?;

                // `FormattedFields` is a formatted representation of the span's
                // fields, which is stored in its extensions by the `fmt` layer's
                // `new_span` method. The fields will have been formatted
                // by the same field formatter that's provided to the event
                // formatter in the `FmtContext`.
                let ext = span.extensions();
                let fields = &ext
                    .get::<FormattedFields<N>>()
                    .expect("will never be `None`");

                // Skip formatting the fields if the span had no fields.
                if !fields.is_empty() {
                    //write!(writer, "{{{}}}", fields)?;
                }
                write!(writer, ": ")?;
            }
        }

        // Write fields on the event
        ctx.field_format().format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}
