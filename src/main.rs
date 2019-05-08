mod config;
mod modinfo;

use clap::{App, Arg, ArgMatches, SubCommand};
use memmap::MmapOptions;
use modinfo::ModuleInfoSet;
use std::fs::File;

fn cmd_init(_args: &ArgMatches) -> Result<(), Box<std::error::Error>> {
    // TODO: println!("{}", include_str!("../scripts/init.sh"));
    Ok(())
}

fn cmd_show(args: &ArgMatches) -> Result<(), Box<std::error::Error>> {
    let file = File::open(args.value_of("json").unwrap())?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };
    let modules = ModuleInfoSet::new(&mmap[..]);
    let module_name = args.value_of("module").unwrap();
    let info = modules
        .find(&module_name)
        .ok_or_else(|| format!("{}: module not found", module_name))?;
    println!("{:#?}", info);
    Ok(())
}

fn main() -> Result<(), Box<std::error::Error>> {
    let matches = App::new("ash")
        .subcommand(SubCommand::with_name("init"))
        .subcommand(
            SubCommand::with_name("show")
                .arg(Arg::with_name("json").required(true))
                .arg(Arg::with_name("module").required(true)),
        )
        .get_matches();

    match matches.subcommand() {
        ("init", Some(args)) => cmd_init(args),
        ("show", Some(args)) => cmd_show(args),
        _ => Err(matches.usage().into()),
    }
}
