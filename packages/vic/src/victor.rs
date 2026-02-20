use core::fmt::Display;
use std::collections::HashMap;
use std::io;
use std::os::unix::fs::FileExt;
use log::info;
use serde_json::Value as JsonValue;
use serde_json::Map as JsonMap;
use snix_eval::Evaluation;
use snix_eval::Value;
use std::{fs::File, io::copy};
use anyhow::Result;
use log::{debug, trace};
use flate2::read::GzDecoder;
use tar::Archive;
use std::path::Path;
use std::fs;
use std::io::{SeekFrom, Seek};
use std::process::{Command, Stdio};

use crate::error::{VicResult, VicError, IntoVicResult};
use crate::vic_err;
use crate::eval::VicEvalResult;
use crate::eval::build_snix_evaluator;

pub static BUILD_CONFIG: &str = include_str!(std::env!("VIC_BUILD_CONFIG"));

// a struct for state
pub struct Victor {
    pub config: VictorConfig,
}

impl Victor {
    pub fn eval(&mut self, expr: impl AsRef<str> + Display) -> VicResult<Value> {
        let evaluator = build_snix_evaluator(self)?;
        let vicpkgs_url = self.config_get("vicpkgs_url")?;
        let expr = format!(r#" let
            vicpkgs_url = "{vicpkgs_url}";
            src = if (builtins.substring 0 7 vicpkgs_url) == "github:" 
                then builtins.fetchGit {{
                    url = "git@github.com:c2vi/nixpkgs";
                }}
                else {vicpkgs_url};
            vic = import src {{}};
        in ({})
        "#, expr);
        let result = evaluator.evaluate(expr, None);

        if let Some(value) = result.value {
            return Ok(value);
        } else {
           let mut err = vic_err!("error during evaluation....");
           for error in result.errors {
               err = err.msg(format!("SnixEvalError: {:?}", error));
           }
           for warning in result.warnings {
               err = err.msg(format!("SnixEvalWarning: {:?}", warning));
           }

           return Err(err);
        }
    }

    pub fn new() -> VicResult<Victor> {
        let mut victor = Victor { 
            config: VictorConfig::empty(),
        };

        //victor.pre_build_config_init()?;

        victor.config.read_build_config()?;

        victor.pre_folder_config_init()?;

        victor.config.read_folder_config()?;

        //victor.post_config_init()?;

        Ok(victor)
    }

    fn pre_folder_config_init(&mut self) -> VicResult<()> {

        //////////// on android $HOME points to / which is not readable
        // so set vic_dir to /sdcard/.victorinix
        if are_we_on_android()? {
            self.config_set("vic_dir", "/data/local/.victorinix")?;
        }

        Ok(())
    }

    pub fn config_get<P: IntoVecString>(&self, path: P) -> VicResult<String> {
        self.config.get(path.to_vec_string())
    }

    pub fn config_set<P: IntoVecString, V: Into<String>>(&mut self, path: P, value: V) -> VicResult<()> {
        self.config.set(path.to_vec_string(), value.into());
        Ok(())
    }

    pub fn config_exists<P: IntoVecString>(&self, path: P) -> bool {
        self.config.exists(path.to_vec_string())
    }

    fn fetch_tarball(&mut self) -> VicResult<()> {
        if self.config_exists("internal.tarball_is_fetched") && self.config_get("internal.tarball_is_fetched")? == "true" {
            // tarball is already fetched
            return Ok(());
        }

        if Path::new(&self.config_get("vic_dir")?).join("nix").exists() {
            //vic $VIC_DIR/nix exists the tarball should also be fetched
            return Ok(());
        }

        // create_folder if it doesn't already
        self.create_folder()?;

        // determine arch
        let mut arch_cmd = Command::new("uname");
        arch_cmd.arg("-m");
        let arch_output = arch_cmd.output()
            .vic_result_msg("can't get output of 'uname -m'")?;
        let mut arch_iter = arch_output.stdout.clone().into_iter();
        arch_iter.next_back();
        let arch = String::from_utf8(arch_iter.collect())
            .vic_result_msg("not able to turn the output of 'uname -m' into a string")?;
        debug!("found arch: '{}'", arch);

        let mut url = format!("{}/tars/{}-linux.tar.gz", self.config_get("url")?, arch);
        let vic_dir = self.config_get("vic_dir")?;
        let tmp_file_path_gz = vic_dir.clone() + "/tmp.tar.gz";

        info!("fetching tarball from: {}", url);

        let mut response = reqwest::blocking::get(url)?;

        let mut tmp_file_gz = File::create(&tmp_file_path_gz)?;

        copy(&mut response, &mut tmp_file_gz)?;

        let tar_gz = File::open(&tmp_file_path_gz)?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);
        //archive.set_mask(umask::Mode::parse("rwxrwxrwx")?.into());
        archive.unpack(&vic_dir)?;

        fs::remove_file(&tmp_file_path_gz)?;

        self.config_set("internal.tarball_is_fetched", "true");

        Ok(())
    }

    fn create_folder(&self) -> VicResult<()> {
        let path = self.config_get("vic_dir")?;

        info!("creating vic_dir at: {}", path);

        if !Path::new(&path).exists() {
            fs::create_dir(path);
        }
        Ok(())
    }

    pub fn run_from_vic_pkgs(&mut self, runnable: &str, sub_args: Vec<&str>) -> VicResult<()> {

        debug!("running '{}' from vic pkgs", runnable);

        self.fetch_tarball()?;

        self.nix_run(runnable, sub_args)?;

        Ok(())
    }

    pub fn nix_run(&mut self, runnable: &str, sub_args: Vec<&str>) -> VicResult<()> {

        let tmp_binding = self.config_get("vic_dir")?;
        let vic_dir_path = Path::new(&tmp_binding);

        let proot_path = vic_dir_path.join("nix").join("proot");
        let nix_path = fs::read_to_string(vic_dir_path.join("nix").join("nix-path"))?;
        let busybox_path = fs::read_to_string(vic_dir_path.join("nix").join("busybox-path"))?;
        let cacert_path = fs::read_to_string(vic_dir_path.join("nix").join("cacert-path"))?;

        println!("nix_path: {}", nix_path);
        println!("runnable: {}", runnable);

        if runnable == "default" {
            let mut proot = Command::new(&proot_path);
            proot.env_clear();
            proot.env("PATH", format!("{}:{}", busybox_path, nix_path));
            proot.env("HISTFILE", "/history");
            proot.env("SSL_CERT_FILE", cacert_path);
            proot.arg("-r").arg(vic_dir_path);
            proot.arg("-b").arg("/:/out");
            proot.arg("-w").arg("/");
            proot.arg("-i").arg("0:0");
            proot.arg(busybox_path.clone() + "/sh");

            let mut child_handle = proot.spawn()?;

            child_handle.wait();;

            return Ok(());
        }

        let mut proot = Command::new(&proot_path);
        proot.env_clear();
        proot.env("PATH", format!("{}:{}", busybox_path, nix_path));
        proot.env("HISTFILE", "/history");
        proot.env("TERM", "xterm-color");
        proot.env("SSL_CERT_FILE", cacert_path);
        proot.arg("-r").arg(vic_dir_path);
        proot.arg("-b").arg("/:/out");
        proot.arg("-w").arg("/");
        //proot.arg("-0");
        proot.arg("-i").arg("0:0");

        proot.arg(busybox_path.clone() + "/sh");
        proot.arg("/run");

        proot.env("VIC_TO_RUN", format!("nix --extra-experimental-features nix-command --extra-experimental-features flakes run nixpkgs#{}", runnable));

        //proot.arg(nix_path.clone() + "nix");
        //proot.arg("--extra-experimental-features nix-command");
        //proot.arg("--extra-experimental-features flakes");
        //proot.arg("run");
        //proot.arg(format!("nixpkgs#{}", runnable));

        //proot.arg(nix_path);
        //for sub_arg in sub_args {
            //proot.arg(sub_arg);
        //};

        let mut child_handle = proot.spawn()?;

        child_handle.wait();;

        Ok(())
    }

    pub fn run_from_resource(&self, runnable: &str) -> VicResult<()> {
        println!("running from resource: {}", runnable);
        Ok(())
    }

    pub fn run_flake_url(&self, runnable: &str) -> VicResult<()> {
        println!("running flake url: {}", runnable);
        Ok(())
    }

}

struct VictorConfig {
    inner: HashMap<String, String>,
}

impl VictorConfig {
    pub fn new() -> VicResult<VictorConfig> {
        let mut config = VictorConfig::empty();
        config.read_build_config()?;
        config.read_folder_config()?;
        Ok(config)
    }

    pub fn empty() -> VictorConfig {
        VictorConfig { inner: HashMap::new() }
    }

    fn read_folder_config(&mut self) -> VicResult<()> {
        debug!("reading config from vic_dir");
        trace!("read folder config ... VictorConfig is now: {:?}", self.inner);
        self.fix_vic_dir()?;
        Ok(())
    }

    // this fn replaces a leading '~' in the vic_dir conf var with an absolute path to a users
    // home dir
    fn fix_vic_dir(&mut self) -> VicResult<()> {
        let old_vic_dir = self.get(vec!["vic_dir".to_owned()])?;

        let first_char = old_vic_dir.chars().nth(0)
            .ok_or(vic_err!("vic_dir config path is an empty string, should not be possible"))?;

        if first_char == '~' {
            debug!("substituting ~ with val from $HOME in vic_dir");
            let home_path = std::env::var("HOME").vic_result_msg("Could not get $HOME")?;
            let new_vic_dir = old_vic_dir.replace("~", &home_path);
            self.set(vec!["vic_dir".to_owned()], new_vic_dir)?;
        }

        Ok(())
    }

    fn read_build_config(&mut self) -> VicResult<()> {
        debug!("reading config that was set at build time");
        let mut json_val: JsonValue = serde_json::from_str(BUILD_CONFIG)
            .vic_result_msg("Error parsing string from BUILD_CONFIG")
            .map_err(|e| e.msg("which is read at build time from a file specified in the VIC_BUILD_CONFIG env var"))?;

        if let JsonValue::Object(ref mut map) = json_val {
            let vec = json_map_to_vec(map, Vec::new(), Vec::new())?;
            for (key, val) in vec {
                self.inner.insert(key, val);
            }
        } else {
            return Err(vic_err!("root of json string from BUILD_CONFIG is not a map"));
        }
        self.fix_vic_dir()?;
        trace!("read build config ... VictorConfig is now: {:?}", self.inner);
        Ok(())
    }

    pub fn get(&self, path: Vec<String>) -> VicResult<String> {
        let val = self.inner.get(&path.join("."))
            .ok_or(vic_err!("Could not get config path '{}'", path.join(".")))?;
        return Ok(val.to_owned());
    }

    pub fn set(&mut self, path: Vec<String>, val: String) -> VicResult<()> {
        self.inner.insert(path.join("."), val);
        Ok(())
    }

    pub fn exists(&self, path: Vec<String>) -> bool {
        self.inner.contains_key(&path.join("."))
    }
}


fn json_map_to_vec(map: &mut JsonMap<String, JsonValue>, cur_path: Vec<String>, mut out_vec: Vec<(String, String)>) -> VicResult<Vec<(String, String)>> {
    for (key, value) in map.iter_mut() {
        trace!("json_map_to_vec loop - key: {}", key);
        trace!("json_map_to_vec loop - value: {}", value);
        let mut inner_path = cur_path.clone();
        inner_path.push(key.to_owned());
        match value {
            JsonValue::Object(ref mut inner_map) => {
                return json_map_to_vec(inner_map, inner_path, out_vec);
            },
            JsonValue::String(string) => {
                out_vec.push((inner_path.join("."), string.to_owned()));
            }
            val => {
                out_vec.push((inner_path.join("."), format!("{}", val)));
            }
        }
    };
    Ok(out_vec)
}

trait IntoVecString {
    fn to_vec_string(self) -> Vec<String>;
}

impl IntoVecString for &str {
    fn to_vec_string(self) -> Vec<String> {
        self.to_owned().split(".").map(|v| v.to_owned()).collect()
    }
}


fn are_we_on_android() -> VicResult<bool> {
    // fn to check if we are on android

    let mut check_for_getprop = Command::new("sh");
    check_for_getprop.arg("-c").arg("command -v getprop");
    check_for_getprop.stdout(Stdio::null());
    check_for_getprop.stderr(Stdio::null());
    let status = check_for_getprop.status();

    if status.is_ok() {
        trace!("getprop command found");
        // we could be on android, check further by getting the ro.build.version.release prop

        let mut get_prop = Command::new("getprop");
        get_prop.arg("ro.build.version.release");
        get_prop.stdout(Stdio::null());
        get_prop.stderr(Stdio::null());

        if get_prop.status().is_ok() {
            // we are on android
            debug!("ANDROID platform found");
            return Ok(true);

        }
    }

    // no android found
    return Ok(false);
}

