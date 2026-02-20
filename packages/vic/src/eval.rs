use crate::{error::{VicError, VicResult}, vic_err, victor::Victor};
use std::fs;
use nix_compat::flakeref::FlakeRefOutput;
use snix_eval::{Builtin, Evaluation};
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
use core::{convert::From, option::Option::None, todo, unimplemented};
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



pub type VicEvalResult = Result<Value, VicError>;


pub fn build_snix_evaluator<'a>(vic: &mut Victor) -> VicResult<Evaluation<'a, 'a, 'a, Box<dyn EvalIO>>> {
    let mut tokio_runtime = tokio::runtime::Runtime::new().expect("failed to setup tokio runtime");

    let nix_path = "nixpkgs=/nix/store/1pwi0971a1j83l325xswpc2y019zkxiz-source".to_owned();


    let tracing_handle = snix_tracing::TracingBuilder::default()
        .enable_progressbar()
        .build()
        .expect("unable to set up tracing subscriber");

    let mut stdout = tracing_handle.get_stdout_writer();
    let mut stderr = tracing_handle.get_stderr_writer();


    let (blob_service, directory_service, path_info_service, nar_calculation_service, build_service) = tokio_runtime.block_on(vic_construct_services(vic))?;



    let snix_store_io = Rc::new(SnixStoreIO::new(
        blob_service.clone(),
        directory_service.clone(),
        path_info_service,
        nar_calculation_service.into(),
        build_service.into(),
        tokio_runtime.handle().clone()
    ));

    let mut eval_builder = snix_eval::Evaluation::builder(Box::new(SnixIO::new(snix_store_io.clone() as Rc<dyn EvalIO>,)) as Box<dyn EvalIO>)
    .enable_import();

    eval_builder = eval_builder.add_builtins(impure_builtins());
    eval_builder = add_derivation_builtins(eval_builder, Rc::clone(&snix_store_io));
    eval_builder = add_fetcher_builtins(eval_builder, Rc::clone(&snix_store_io));
    eval_builder = add_import_builtins(eval_builder, Rc::clone(&snix_store_io));
    //eval_builder = add_get_flake_builtins(eval_builder, Rc::clone(&snix_store_io));
    eval_builder = configure_nix_path(eval_builder, &Some(nix_path));

    //if let Some(source_map) = source_map {
        //eval_builder = eval_builder.with_source_map(source_map);
    //}

    let source_map = eval_builder.source_map().clone();

    let mut compiler_observer = DisassemblingObserver::new(source_map.clone(), stderr.clone());
    //if args.dump_bytecode {
        //eval_builder.set_compiler_observer(Some(&mut compiler_observer));
    //}

    let mut runtime_observer = TracingObserver::new(stderr.clone());
    //if args.trace_runtime {
        //if args.trace_runtime_timing {
            //runtime_observer.enable_timing()
        //}
        //eval_builder.set_runtime_observer(Some(&mut runtime_observer));
    //}

    let eval = eval_builder.build();

    return Ok(eval);

}

pub async fn vic_construct_services(vic: &mut Victor) -> 
VicResult<(
        Arc<dyn BlobService>,
        Arc<dyn DirectoryService>,
        Arc<dyn PathInfoService>,
        Box<dyn NarCalculationService>,
        Box<dyn BuildService>,
)> {
    let vic_snix_path = PathBuf::from(vic.config_get("vic_dir")?).join("snix");

    // create the dirs
    {
        // the snix folder
        if !vic_snix_path.exists() {
            fs::create_dir(&vic_snix_path);
        }
        // the blobservice folder
        if !vic_snix_path.join("blobservice").exists() {
            fs::create_dir(&vic_snix_path.join("blobservice"));
        }
        // the blobservice folder
        if !vic_snix_path.join("buildservice").exists() {
            fs::create_dir(&vic_snix_path.join("buildservice"));
        }
    }

    let castore_service_addrs = snix_castore::utils::ServiceUrls {
        blob_service_addr: format!("objectstore+file://{}/blobservice", vic_snix_path.display()),
        directory_service_addr: format!("redb://{}/directoryservice.redb", vic_snix_path.display()),
        experimental_store_composition: None,
    };
    let path_info_service_addr = format!("redb://{}/pathinfo.redb", vic_snix_path.display());
    let service_urls = snix_store::utils::ServiceUrls { castore_service_addrs, path_info_service_addr, experimental_store_composition: None, };
    let build_service_addr = format!("oci://{}/buildservice", vic_snix_path.display());

    let (
        blob_service,
        directory_service,
        path_info_service,
        nar_calculation_service,
    ) = snix_store::utils::construct_services(service_urls)
        .await
        .expect("unable to setup {blob|directory|pathinfo}service before interpreter setup");


    let build_service = {
        let blob_service = blob_service.clone();

        let directory_service = directory_service.clone();

        async move {

            buildservice::from_addr(

                &build_service_addr,

                blob_service.clone(),

                directory_service.clone(),

            ).await

        }.await

    }.expect("unable to setup buildservice before interpreter setup");

    return Ok((blob_service, directory_service, path_info_service, nar_calculation_service, build_service));

}

/*

pub fn add_get_flake_builtins<'co, 'ro, 'env, IO>(
    eval_builder: snix_eval::EvaluationBuilder<'co, 'ro, 'env, IO>,
    io: Rc<SnixStoreIO>,
) -> snix_eval::EvaluationBuilder<'co, 'ro, 'env, IO> {

    let getflake_builtin = Builtin::new("getFlake", Some("see: https://nix.dev/manual/nix/2.18/language/builtins#builtins-getFlake"), 1, getflake_builtin_fn);

    todo!()
}

fn getflake_builtin_fn(url: Vec<Value>) -> Value {
    use nix_compat::flakeref::FlakeRef;
    /*
    let flake_ref: FlakeRef = url.into();

    match flake_ref {
        FlakeRef::GitHub { owner, repo, host, keytype, public_key, public_keys, r#ref, rev } => {
        },
        _ => unimplemented!()
    }
    */
    todo!()
}

*/
