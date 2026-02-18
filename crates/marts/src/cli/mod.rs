use clap::{ArgMatches, Command};
use std::collections::HashMap;

use mize::{mize_part, Mize, MizePart, MizeResult};

#[mize_part("cli")]
#[derive(Default)]
pub struct CliPart {
    mize: Mize,
    cmd: Option<Command>,
    actions: HashMap<String, fn(&ArgMatches) -> MizeResult<()>>,
}

pub fn cli(mize: &mut Mize) -> MizeResult<()> {
    let cli_part = CliPart {
        mize: mize.clone(),
        cmd: Some(Command::new("marts-cli")),
        actions: HashMap::new(),
    };

    mize.register_part(Box::new(cli_part))
}

impl MizePart for CliPart {
    fn init(&mut self, _mize: &mut Mize) -> MizeResult<()> {
        Ok(())
    }
    fn run(&mut self, _mize: &mut Mize) -> MizeResult<()> {
        let matches = self.cmd.take().unwrap().get_matches();
        let sub_cmd = matches.subcommand_name().unwrap();
        let action = self.actions.get(sub_cmd).unwrap();
        action(matches.subcommand_matches(sub_cmd).unwrap())?;
        Ok(())
    }
    fn opts(&self, mize: &mut Mize) {
        mize.new_opt("cli.name");
    }
}

impl CliPart {
    pub fn subcommand(
        &mut self,
        subcmd: Command,
        action: fn(sub_matches: &ArgMatches) -> MizeResult<()>,
    ) -> &mut Command {
        let name = subcmd.get_name().to_string();
        let cmd = self.cmd.take().unwrap().subcommand(subcmd);
        self.actions.insert(name, action);
        self.cmd = Some(cmd);
        self.cmd.as_mut().unwrap()
    }
    pub fn with_cmd(
        &mut self,
        func: fn(mize: Mize, cmd: Command) -> MizeResult<Command>,
    ) -> MizeResult<()> {
        let cmd = func(self.mize.clone(), self.cmd.take().unwrap())?;
        self.cmd = Some(cmd);
        Ok(())
    }
}
