use clap::{ArgMatches, Command};
use std::any::Any;
use std::collections::HashMap;

use mize::{Mize, MizePart, MizeResult, instance::MizePartGuard};

struct CliPart {
    mize: Mize,
    cmd: Option<Command>,
    actions: HashMap<String, fn(&ArgMatches) -> MizeResult<()>>,
}

pub fn new(mize: Mize) -> CliPart {
    CliPart {
        mize,
        cmd: None,
        actions: HashMap::new(),
    }
}

impl MizePart for CliPart {
    fn name(&self) -> &'static str {
        "cli"
    }
    fn get_mize(&mut self) -> &mut Mize {
        &mut self.mize
    }
    fn init(&mut self, _mize: &mut Mize) -> MizeResult<()> {
        Ok(())
    }
    fn opts(&self, mize: &mut Mize) {
        mize.new_opt("cli.name");
    }
    fn run(&mut self, _mize: &mut Mize) -> MizeResult<()> {
        let matches = self.cmd.take().unwrap().get_matches();
        let sub_cmd = matches.subcommand_name().unwrap();
        let action = self.actions.get(sub_cmd).unwrap();
        action(matches.subcommand_matches(sub_cmd).unwrap());
        Ok(())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&self) -> &dyn std::any::Any {
        self
    }
    fn into_any(self: Box<CliPart>) -> Box<dyn Any> {
        self
    }
}

impl CliPart {
    pub fn subcommand(
        &mut self,
        cmd: Command,
        action: fn(sub_matches: &ArgMatches) -> MizeResult<()>,
    ) -> &mut Command {
        let cmd = self.cmd.take().unwrap().subcommand(cmd);
        self.actions.insert(cmd.get_name().to_string(), action);
        self.cmd = Some(cmd);
        self.cmd.as_mut().unwrap()
    }
    pub fn get(mize: &mut Mize) -> MizePartGuard<CliPart> {
        let mut dyn_guard = mize.get_part("cli").unwrap();
        let part = dyn_guard.part.take().unwrap();
        let cli_part = part.into_any().downcast::<CliPart>().unwrap();
        MizePartGuard {
            mize: dyn_guard.mize.clone(),
            part: Some(*cli_part),
        }
    }
}
