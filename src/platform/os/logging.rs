use std::collections::HashMap;
use std::path::PathBuf;
use clap::{ArgAction, ArgMatches};
use clap::{Arg, crate_version, Command};

use mize::error::{MizeError, MizeResult};
use mize::id::MizeId;
use mize::item::ItemData;
use tokio::sync::mpsc;
use std::sync::Mutex;
use std::sync::Arc;
use colored::Colorize;
use std::env;
use tracing::{span_enabled, Instrument, Level};
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

use std::sync::atomic::AtomicUsize;

use tracing::{field::Visit, Id, Subscriber};
use tracing_core::Field;
use std::fmt::Write;

pub fn init_logger(cli_matches: &ArgMatches) {

    // to save log messages, before the logger is setup
    let mut log_messages: Vec<(Level, String)> = Vec::new();

    log_messages.push((Level::INFO, "Starting Mize".to_owned()));

    // builder with DEFAULT_LOG_LEVEL
    //let mut builder = tracing_subscriber::fmt::Subscriber::builder();

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

    // check env MIZE_LOG
    let mut val_mize_log = level_from_env("MIZE_LOG", log_level);
    if let Some(level_mize_log) = val_mize_log.0 {
        log_level = level_mize_log;
        log_messages.push((Level::TRACE, format!("set log level from the Variable MIZE_LOG to {}", level_mize_log)));
    }
    log_messages.append(&mut val_mize_log.1);

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
        let log_level_from_arg = match log_level_arg_string.to_owned().to_lowercase().as_str() {
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
    //let mut my_env_filter = tracing_subscriber::filter::EnvFilter::new("")
        //.add_directive("[get_raw_from_cbor]=debug".parse().expect("error parsing log directive"))
        //.add_directive("[MemStore::get_value_raw]=trace".parse().expect("error parsing log directive"))
        //.add_directive("trace".parse().unwrap())
        //.add_directive("[get_raw_from_cbor]=debug".parse().expect("error parsing log directive"))
    //;

    MinimalTracer::register();

    //builder
        //.with_env_filter(my_env_filter)
        //.without_time()
        //.event_format(MyFormatter)
        //.with_max_level(log_level)
        //.init();


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

// thanks: https://stackoverflow.com/a/75546824
pub struct StringVisitor<'a> {
    string: &'a mut String,
}
impl<'a> StringVisitor<'a> {
    pub(crate) fn new(string: &'a mut String) -> Self {
        StringVisitor { string }
    }
}

impl<'a> Visit for StringVisitor<'a> {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            write!(self.string, "{value:?} ").unwrap();
        } else {
            write!(self.string, "{} = {:?}; ", field.name(), value).unwrap();
        }
    }
}

struct LogFilter {
    target: Option<String>,
    level: Option<Level>,
}

pub struct MinimalTracer {
    enabled: bool,
    filters: Vec<LogFilter>,
    level: Arc<Mutex<Level>>,
    spans: Arc<Mutex<Vec<Id>>>,
    span_levels: Arc<Mutex<HashMap<Id, Level>>>,
    span_names: Arc<Mutex<HashMap<Id, String>>>,
    span_names_to_enable: Arc<Mutex<Vec<String>>>,
    level_no_span: Level,
}

fn string_to_level(string: &str) -> Option<Level> {
    match string.to_lowercase().as_str() {
        "info" => Some(Level::INFO),
        "debug" => Some(Level::DEBUG),
        "warn" | "warning" => Some(Level::WARN),
        "trace" => Some(Level::TRACE),
        "error" => Some(Level::ERROR),
        _ => None,
    }
}

impl MinimalTracer {
    pub fn register() -> Result<(), tracing::subscriber::SetGlobalDefaultError> {
        let mut enabled = true;
        let mut span_names_to_enable = Vec::new();
        let mut my_level = Level::ERROR;

        let mut filters: Vec<LogFilter> = Vec::with_capacity(10);
        if let Ok(env_value) = env::var("RUST_LOG") {
            for filter in env_value.split(',') {
                let mut target = Some(filter);
                let mut level = None;
                if let Some(equals_index) = target.unwrap().find('=') {
                    let (first, second) = filter.split_at(equals_index);
                    target = Some(first);
                    level = string_to_level(&second[1..])
                } else {
                    my_level = string_to_level(target.unwrap()).unwrap();
                }
                let target_level = string_to_level(target.unwrap());

                //if let Some(target_level) = target_level {
                    //level = Some(target_level);
                    //target = None;
                //}

                // in case target is a span name, add id to span_names_to_enable
                // for now every target is also a span_name.....
                // i would think everything, that contains a "." would be a span, and not a module
                // targeet
                if let Some(name) = target {
                    span_names_to_enable.push(name.to_owned());
                }

                filters.push(LogFilter {
                    target: target.map(|v| v.to_string()),
                    level,
                });
            }
        } else {
            enabled = false;
        }

        tracing::subscriber::set_global_default(MinimalTracer { 
            enabled, filters,
            spans: Arc::new(Mutex::new(Vec::new())),
            level: Arc::new(Mutex::new(my_level)),
            span_names: Arc::new(Mutex::new(HashMap::new())),
            span_names_to_enable: Arc::new(Mutex::new(span_names_to_enable)),
            span_levels: Arc::new(Mutex::new(HashMap::new())),
            level_no_span: my_level,
        })
    }
}

static AUTO_ID: AtomicUsize = AtomicUsize::new(1);

impl Subscriber for MinimalTracer {
    fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
        return true;

        if metadata.is_span() {
            return true;
            let mut span_names_to_enable = self.span_names_to_enable.lock().unwrap();
            if span_names_to_enable.contains(&metadata.name().to_owned()) {
                return true;
            }
        }

        if self.enabled {
            if self.filters.is_empty() {
                return true;
            }

            let mut matches: bool;
            for filter in &self.filters {
                matches = true;
                if let Some(level) = filter.level {
                    if metadata.level() != &level {
                        matches = false;
                    }
                }
                if let Some(target) = &filter.target {
                    if !metadata.target().starts_with(target) {
                        matches = false;
                    }
                }
                if matches {
                    return true;
                }
            }
            return false;
        }
        false
    }

    fn new_span(&self, _span: &tracing_core::span::Attributes<'_>) -> tracing_core::span::Id {
        let mut span_names = self.span_names.lock().unwrap();
        let mut span_levels = self.span_levels.lock().unwrap();
        let mut span_names_to_enable = self.span_names_to_enable.lock().unwrap();

        let id = Id::from_u64(AUTO_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed) as u64);
        let span_name = _span.metadata().name().to_string();

        span_names.insert(id.clone(), span_name.clone());
        if span_names_to_enable.contains(&span_name) {
            span_levels.insert(id.clone(), Level::TRACE);
        } else {
            span_levels.insert(id.clone(), Level::ERROR);
        }

        id
    }

    fn record(&self, _span: &tracing_core::span::Id, _values: &tracing_core::span::Record<'_>) {
    }

    fn record_follows_from(
        &self,
        _span: &tracing_core::span::Id,
        _follows: &tracing_core::span::Id,
    ) {
    }

    fn event(&self, event: &tracing::Event<'_>) {
        let metadata = event.metadata();

        let level = metadata.level();
        let my_level = self.level.lock().unwrap();

        //println!("uuuuuuuuuuuu event name: {}", metadata.name());
        //println!("uuuuuuuuuuuu my_level: {}", my_level);
        //println!("uuuuuuuuuuuu event level: {}", level);

        if *level > *my_level {
            //println!("uuuuuuuuuuuu comparison");
            //println!("BLOCKED EVENT")
            return;
        }

        let target = metadata.target();

        let mut text = String::new();

        let mut visitor = StringVisitor::new(&mut text);
        event.record(&mut visitor);

        let level_str = match metadata.level() {
            &Level::TRACE => format!("[ {} ]", "TRACE".yellow()),
            &Level::DEBUG => format!("[ {} ]", "DEBUG".green()),
            &Level::INFO => format!("[ {}  ]", "INFO".blue()),
            &Level::WARN => format!("[ {}  ]", "WARN".truecolor(245, 164, 66)),
            &Level::ERROR => format!("[ {} ]", "ERROR".red()),
        };

        //println!("{} {:?}", "META:".red(), metadata.in_current_span());
        let mut spans = self.spans.lock().unwrap();
        let mut span_names = self.span_names.lock().unwrap();
        let span_text = spans.iter()
            .map(|id| span_names.get(id).unwrap().to_owned())
            .collect::<Vec<String>>()
            .join("::");
        println!("{level_str} {span_text} {text}");
    }

    fn enter(&self, _span: &tracing_core::span::Id) {
        //println!("eeeeeeeeeeeeeeeee enter");
        let mut spans = self.spans.lock().unwrap();
        let mut level = self.level.lock().unwrap();
        let mut span_levels = self.span_levels.lock().unwrap();
        let mut span_names = self.span_names.lock().unwrap();

        let name = span_names.get(_span).unwrap().to_owned();
        *level = span_levels.get(_span).unwrap().to_owned();
        spans.push(_span.clone());
        //println!("my_level is now: {}", level);
    }

    fn exit(&self, _span: &tracing_core::span::Id) {
        //println!("eeeeeeeeeeeeeeeee exit");

        let mut level = self.level.lock().unwrap();
        *level = self.level_no_span;

        let mut spans = self.spans.lock().unwrap();
        spans.pop();
    }
}
