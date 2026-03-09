use std::collections::HashMap;

use clap::Command;
use clap::crate_version;
use mize::{Mize, MizeResult};

fn main() {
    let mut mize = Mize::new().expect("failed to create mize instance");

    #[cfg(feature = "target-os")]
    os_main(&mut mize);

    mize.run();
}

#[cfg(feature = "target-os")]
fn os_main(mize: &mut Mize) -> MizeResult<()> {
    marts::cli(mize)?;
    //marts::js(mize)?;
    marts::habitica(mize)?;
    marts::c2vi(mize)?;

    let mut cli = mize.get_part_native::<marts::CliPart>("cli")?;

    cli.with_cmd(|_, cmd| {
        Ok(cmd
            .version(crate_version!())
            .name("ppc")
            .author("ppc")
            .about("the ppc desktop program"))
    })?;

    cli.subcommand(Command::new("test"), |_, _| {
        println!("test ppc...");
        Ok(())
    });
    Ok(())
}
