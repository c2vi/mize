use deno_core::extension;
use deno_core::op2;
use deno_core::OpState;
use mize::{Mize, MizeError};
use std::sync::Arc;

use crate::CliPart;
use crate::JsPart;

#[op2(fast)]
fn op_mize_get_part(#[string] name: &str) {
    println!("js wants the part {}", name);
}

#[op2]
#[string]
fn op_mize_get_config(state: &mut OpState, #[string] key: &str) -> String {
    let mize: Mize = state.borrow::<Mize>().clone();
    format!("config_value_for_{}", key)
}

#[op2(fast)]
fn op_mize_add_part(state: &mut OpState, #[string] key: &str) {
    let mize: Mize = state.borrow::<Mize>().clone();
    println!("adding part {}", key);
}

#[op2(fast)]
fn op_cli_subcommand(state: &mut OpState, #[string] name: String) {
    let mut mize: Mize = state.borrow::<Mize>().clone();
    let mut cli = mize.get_part_native::<CliPart>("cli").unwrap();
    cli.subcommand(
        clap::Command::new(name.clone()),
        move |_sub_matches, mut mize| {
            let mut js = mize.get_part_native::<JsPart>("js").unwrap();
            js.eval("log(\"cli command being called\")".to_string())?;
            js.eval(format!("mize.vars.cli.subcommand({name})"))?;
            Ok(())
        },
    );
}

// Create an extension with your custom ops
extension!(
    my_extension,
    ops = [op_mize_get_part, op_mize_get_config],
    options = {
      mize: Mize,
    },
    state = |state: &mut OpState, config: Config| {
        state.put::<Mize>(config.mize);
    },
);
