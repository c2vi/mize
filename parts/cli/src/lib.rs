use clap::{ArgMatches, Command};
use std::any::Any;
use std::collections::HashMap;

use mize::{Mize, MizePart, MizeResult, instance::MizePartGuard};

#[mize_part("cli")]
struct CliPart {
    mize: Mize,
    cmd: Option<Command>,
    actions: HashMap<String, fn(&ArgMatches) -> MizeResult<()>>,
}

impl MizePart for CliPart {
    fn init(&mut self, _mize: &mut Mize) -> MizeResult<()> {
        Ok(())
    }
    fn run(&mut self, _mize: &mut Mize) -> MizeResult<()> {
        let matches = self.cmd.take().unwrap().get_matches();
        let sub_cmd = matches.subcommand_name().unwrap();
        let action = self.actions.get(sub_cmd).unwrap();
        action(matches.subcommand_matches(sub_cmd).unwrap());
        Ok(())
    }
    fn opts(&self, mize: &mut Mize) {
        mize.new_opt("cli.name");
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
}
