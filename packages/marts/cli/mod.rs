use clap::{ArgMatches, Command};
use std::collections::HashMap;

use mize::{mize_part, Mize, MizePart, MizeResult};

#[mize_part("cli")]
#[derive(Default)]
pub struct CliPart {
    mize: Mize,
    cmd: Option<Command>,
    actions: HashMap<
        String,
        Box<dyn FnOnce(&ArgMatches, Mize) -> MizeResult<()> + Send + Sync + 'static>,
    >,
    parsers:
        Option<Vec<Box<dyn FnOnce(Mize, Vec<String>) -> MizeResult<()> + Send + Sync + 'static>>>,
}

pub fn cli(mize: &mut Mize) -> MizeResult<()> {
    let command = Command::new("marts-cli").allow_external_subcommands(true);
    let cli_part = CliPart {
        mize: mize.clone(),
        cmd: Some(command),
        actions: HashMap::new(),
        parsers: Some(Vec::new()),
    };

    mize.add_part(Box::new(cli_part))
}

impl MizePart for CliPart {
    fn init(&mut self, _mize: &mut Mize) -> MizeResult<()> {
        Ok(())
    }
    fn run(&mut self, mize: &mut Mize) -> MizeResult<()> {
        let matches = self.cmd.take().unwrap().get_matches();
        let sub_cmd = match matches.subcommand_name() {
            Some(sub_cmd) => sub_cmd,
            None => {
                println!("No subcommand provided");
                return Ok(());
            }
        };
        let action = match self.actions.remove(sub_cmd) {
            Some(action) => action,
            None => {
                // external parsers
                for parser in self.parsers.take().unwrap() {
                    parser(
                        mize.clone(),
                        matches
                            .subcommand_matches(sub_cmd)
                            .unwrap()
                            .get_many::<String>("")
                            .unwrap()
                            .map(|s| s.to_string())
                            .collect::<Vec<String>>(),
                    )?;
                }
                return Ok(());
            }
        };
        action(matches.subcommand_matches(sub_cmd).unwrap(), mize.clone())?;
        Ok(())
    }
    fn opts(&self, mize: &mut Mize) {
        mize.new_opt("cli.name");
    }
}

impl CliPart {
    pub fn subcommand<T: FnOnce(&ArgMatches, Mize) -> MizeResult<()> + Send + Sync + 'static>(
        &mut self,
        subcmd: Command,
        action: T,
    ) -> &mut Command {
        let name = subcmd.get_name().to_string();
        let cmd = self.cmd.take().unwrap().subcommand(subcmd);
        self.actions.insert(name, Box::new(action));
        self.cmd = Some(cmd);
        self.cmd.as_mut().unwrap()
    }
    pub fn add_sub_parser<
        T: FnOnce(Mize, Vec<String>) -> MizeResult<()> + Send + Sync + 'static,
    >(
        &mut self,
        parser: T,
    ) -> MizeResult<()> {
        self.parsers.as_mut().unwrap().push(Box::new(parser));
        Ok(())
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
