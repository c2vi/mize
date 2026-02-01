#![ allow( warnings ) ]

use std::path::PathBuf;
use clap::{ArgAction, ArgMatches};
use clap::{Arg, crate_version, Command};
//use cpp::cpp;
//use slint::ComponentHandle;

use std::sync::Arc;
use std::io::Write;
use colored::Colorize;
use std::env;
use tracing_subscriber::fmt::Subscriber;
use tracing::Level;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::filter::EnvFilter;
use crate::logging::init_logger;
use mize_module_mme::mme::Mme;

use mize::{mize_err, Instance};
use mize::MizeResult;
use mize::MizeError;

use tracing::{trace, debug, info, warn, error};
//use slint::platform::Platform;
//use qt_core::{qs, QString, QTimer, SlotNoArgs};
//use qt_widgets::{QApplication, QGridLayout, QWidget};

mod cli {
}

mod logging;

static APPNAME: &str = "mme";
static DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::WARN;

fn main() {
    let cli_matches = cli_matches();

    init_logger(&cli_matches);

    // match command
    let result = match cli_matches.subcommand() {
        // mi daemon
        //Some(("run", sub_matches)) => cli::run(sub_matches),
        // some unknown command passed

        Some((cmd, sub_matches)) => Err(mize_err!("The subcommand: {} is not known. use --help to list availavle commands", cmd)),

        None => {
            unsafe { default_cmd() }
        },
    };

    if let Err(err) = result {
        err.log();
    }
}


fn default_cmd() -> MizeResult<()> {
    let mize = Instance::new()?;
    let mut mme = Mme::new(mize)?;

    mme.create_x_window()?;


    Ok(())
}

/*
#[no_mangle]
unsafe extern "C" fn place_slint_widget(slint_widget: *mut QWidget) {
    println!("i am here");
    let mme_widget = get_widget_by_title("place_slint");
    println!("mme_widget: {:?}", mme_widget);
    println!("mme_widget title: {}", qstring_to_rust((*mme_widget).window_title()));

    let layout = (*mme_widget).layout();
    println!("layout: {:?}", layout);
    let grid_layout = layout.dynamic_cast::<QGridLayout>();
    println!("grid_layout: {:?}", grid_layout);
    println!("slint_widget: {:?}", slint_widget);

    (*grid_layout).add_widget_3a(slint_widget, 0, 0);
}

unsafe extern "C" fn get_widget_by_title(title: &str) -> * mut QWidget {
    let widgets = QApplication::top_level_widgets();
    let mut mme_widget: *mut QWidget = *widgets.index_mut(0);

    let widgets_size = widgets.size();
    for i in 0..widgets_size {
        let widget = *widgets.index(i);
        let widget_title = qstring_to_rust((*widget).window_title());
        println!("get_mme_main_widget: window_title: {}", widget_title);
        if widget_title == title.to_owned() {
            println!("found: {:?}", widget);
            mme_widget = widget
        }
    }
    return mme_widget;
}
*/




fn cli_matches() -> clap::ArgMatches {


    let main = Command::new(APPNAME)
        .version(crate_version!())
        .author("Sebastian Moser")
        .about("The Main Mize Explorer or Mize Ui Framework")
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
        );

    return main.get_matches();
}



