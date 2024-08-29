use std::process::Command;
use std::env;

fn main() {

    // compile the string plugin
    //let _out_dir = env::var_os("OUT_DIR").unwrap();
    //let mut string_module = Command::new("bash");
    //string_module.arg("-c");
    //string_module.arg("echo hiiiiiiiiiiiiiiiii");
    //string_module.arg("--help");
    //string_module.arg("--out-dir");
    //string_module.arg(out_dir);
    //string_module.env("PWD", "/hoem/me/work/mize/module/modules/Blob");

    //let _res = string_module.spawn();
    //let _child = Command::new("cargo")
        //.arg("build")
        //.arg("--lib")
        //.arg("--manifest-path")
        //.arg("/home/me/work/mize/modules/modules/Blob/Cargo.toml")
        //.spawn()
        //.expect("failed to start wasm build");

        //to outdir (), using the nightly feature --out-dir to have the result there. By default I get a libhearts.rlib
}
