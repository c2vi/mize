use clap::ArgMatches;
use tokio::runtime::Runtime;

use crate::{error::{VicError, VicResult}, vic_err, victor::Victor};

use snix_cli::AllowIncomplete;
use snix_cli::args::Args;
use clap::Parser;
use snix_eval::{
    ErrorKind, EvalIO, EvalMode, GlobalsMap, SourceCode, Value,
    builtins::impure_builtins,
    observer::{DisassemblingObserver, TracingObserver},
};
use snix_glue::{
    builtins::{add_derivation_builtins, add_fetcher_builtins, add_import_builtins},
    configure_nix_path,
    snix_io::SnixIO,
    snix_store_io::{self, SnixStoreIO},
};
use tracing::{Span, info_span};
use snix_build::buildservice;
use core::{convert::From, option::Option::None};
use std::fmt::Write;
use tracing_indicatif::span_ext::IndicatifSpanExt;
use snix_cli::EvalResult;
use std::rc::Rc;
use std::path::Path;
use std::path::PathBuf;
use snix_cli::IncompleteInput;
use std::sync::Arc;
use snix_castore::blobservice::BlobService;
use snix_castore::directoryservice::DirectoryService;
use snix_store::pathinfoservice::PathInfoService;
use snix_store::nar::NarCalculationService;
use snix_build::buildservice::BuildService;
use tracing::info;
use snix_store::pathinfoservice::make_fs;
use snix_castore::fs::fuse::FuseDaemon;
use crate::eval::VicEvalResult;



pub fn main(matches: &ArgMatches) -> VicResult<()> {
    let mut tokio_runtime = tokio::runtime::Runtime::new().expect("failed to setup tokio runtime");

    match matches.subcommand() {
        Some(("build", matches)) => run_build(matches),
        Some(("mount", matches)) => tokio_runtime.block_on(run_mount(matches)),
        Some(("dev", matches)) => run_dev(matches),
        Some((_, _)) => Err(vic_err!("unknown subcommand...")),
        None => Err(vic_err!("need subcommand...")),
    }
}

pub fn run_snix_test(matches: &ArgMatches) -> VicResult<()> {
    let mut vic = Victor::new()?;

    //println!("test: {}", vic.eval("5 + vic.testing")?);
    println!("test: {}", vic.eval(r#"let vic = import /home/me/work/pkgsvic {}; in vic.testing"#)?);

    Ok(())
}

pub fn run_mount_blocking(matches: &ArgMatches) -> VicResult<()> {
    let mut tokio_runtime = tokio::runtime::Runtime::new().expect("failed to setup tokio runtime");
    tokio_runtime.block_on(run_mount(matches));

    Ok(())
}

pub fn run_dev(matches: &ArgMatches) -> VicResult<()> {
    /*

    let mut tokio_runtime = tokio::runtime::Runtime::new().expect("failed to setup tokio runtime");

    let nix_path = "nixpkgs=/nix/store/1pwi0971a1j83l325xswpc2y019zkxiz-source".to_owned();


    let tracing_handle = snix_tracing::TracingBuilder::default()
        .enable_progressbar()
        .build()
        .expect("unable to set up tracing subscriber");

    let mut stdout = tracing_handle.get_stdout_writer();
    let mut stderr = tracing_handle.get_stderr_writer();


    let (blob_service, directory_service, path_info_service, nar_calculation_service, build_service) = tokio_runtime.block_on(my_construct_services());

    let snix_store_io = Rc::new(SnixStoreIO::new(
        blob_service.clone(),
        directory_service.clone(),
        path_info_service,
        nar_calculation_service.into(),
        build_service.into(),
        tokio_runtime.handle().clone()
    ));

    let bundle_name = Uuid::new_v4();
    let bundle_path = PathBuf::from("/home/me/work/tori-victorinix/gitignore/building/snix/buildservice").join(bundle_name.to_string());

    let oci_process = oci_spec::runtime::ProcessBuilder::default()
        .terminal(true)
        .user(
            oci_spec::runtime::UserBuilder::default()
                .uid(1000u32)
                .gid(100u32)
                .build()?,
        )
        .cwd(Path::new("/"))
        .capabilities(
            HashSet::from([
                Capability::AuditWrite,
                Capability::Chown,
                Capability::DacOverride,
                Capability::Fowner,
                Capability::Fsetid,
                Capability::Kill,
                Capability::Mknod,
                Capability::NetBindService,
                Capability::NetRaw,
                Capability::Setfcap,
                Capability::Setgid,
                Capability::Setpcap,
                Capability::Setuid,
                Capability::SysChroot,
            ])
        )
        .build().expect("failed to build oci_process");
        ;
    
    let spec_builder = oci_spec::runtime::SpecBuilder::default()
        .process(
            oci_process
        )
        .root(
            oci_spec::runtime::RootBuilder::default()
                .path("root")
                .readonly(true)
                .build()
                .map_err(SpecError::OciError)?,
        )
        .hostname("localhost")
        .mounts(
            configure_mounts(
                rootless,
                allow_network,
                request.scratch_paths.iter().map(|e| e.as_path()),
                request.inputs.iter(),
                &request.inputs_dir,
                ro_host_mounts,
            )
            .map_err(SpecError::OciError)?,
        );

    println!("before builder.build()");
    let spec = builder.build().expect("failed to build spec_builder");

    ////////////// create all the paths for the bundle
    fs::create_dir_all(bundle_path).expect("failed to create bundle path");
    fs::create_dir_all(bundle_path.join("root").join("etc")).expect("failed to create bundle path");

    ////// write config.json
    let spec_json = serde_json::to_string(spec).expect("failed to render spec to json")?;
    fs::write(bundle_path.join("config.json"), spec_json).expect("failed to write config.json")?;
    */

    Ok(())

}

pub async fn run_mount(matches: &ArgMatches) -> VicResult<()> {

    let mut vic = Victor::new()?;

    let mnt_string_ref = matches.get_one::<String>("mnt-path").unwrap();
    let mnt_path: String = format!("{}", mnt_string_ref);

    let (blob_service, directory_service, path_info_service, nar_calculation_service, build_service) = crate::eval::vic_construct_services(&mut vic).await?;


            let fuse_daemon = tokio::task::spawn_blocking(move || {
                let fs = make_fs(
                    blob_service,
                    directory_service,
                    path_info_service,
                    true,
                    true,
                );

                FuseDaemon::new(fs, PathBuf::from(mnt_path), 4, true)
            })
            .await??;

            // Wait for a ctrl_c and then call fuse_daemon.unmount().
            tokio::spawn({
                let fuse_daemon = fuse_daemon.clone();
                async move {
                    tokio::signal::ctrl_c().await.unwrap();
                    info!("interrupt received, unmounting…");
                    tokio::task::spawn_blocking(move || fuse_daemon.unmount()).await??;
                    info!("unmount occured, terminating…");
                    Ok::<_, std::io::Error>(())
                }
            });

            // Wait for the server to finish, which can either happen through it
            // being unmounted externally, or receiving a signal invoking the
            // handler above.
            tokio::task::spawn_blocking(move || fuse_daemon.wait()).await?;

    Ok(())
}

pub fn run_build(matches: &ArgMatches) -> VicResult<()> {

    let mut vic = Victor::new()?;

    let expr = r#"
        let
            a = builtins.trace "hooooooooooo" "hello world";
            pkgs = import <nixpkgs> {};
            b = builtins.readFile "${pkgs.hello}/share/info/hello.info";
        in ''
            a: ${a}
            b: ${b}
        ''
    "#;

    let result = vic.eval(expr)?;

    println!("result: {}", result);

    Ok(())
}

