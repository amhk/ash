mod config;
mod modinfo;

use clap::{App, Arg, ArgMatches, SubCommand};
use config::ModuleGroup;
use memmap::MmapOptions;
use modinfo::ModuleInfoSet;
use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;

fn cmd_init() -> Result<(), Box<std::error::Error>> {
    // TODO: println!("{}", include_str!("../scripts/init.sh"));
    Ok(())
}

fn cmd_envsetup(_script: &str) -> Result<(), Box<std::error::Error>> {
    Ok(())
}

fn cmd_show(
    modules: &ModuleInfoSet,
    groups: &[ModuleGroup],
    args: &ArgMatches,
) -> Result<(), Box<std::error::Error>> {
    let name = args.value_of("module-or-group").unwrap();
    if name.starts_with(':') {
        let group = groups
            .iter()
            .find(|item| item.name == name)
            .ok_or_else(|| format!("{}: group not found", name))?;
        println!("{:#?}", group);
    } else {
        let info = modules
            .find(&name)
            .ok_or_else(|| format!("{}: module not found", name))?;
        println!("{:#?}", info);
    }
    Ok(())
}

fn cmd_cd(
    modules: &ModuleInfoSet,
    _groups: &[ModuleGroup],
    args: &ArgMatches,
) -> Result<(), Box<std::error::Error>> {
    let module_name = args.value_of("module").unwrap();
    let info = modules
        .find(&module_name)
        .ok_or_else(|| format!("{}: module not found", module_name))?;
    assert!(!info.path.is_empty()); // required by parse step
    if info.path.len() > 1 {
        return Err("multiple paths".into());
    }
    let mut path = PathBuf::new();
    path.push(
        std::env::var("ANDROID_BUILD_TOP").or_else(|_| Err("failed to get ANDROID_BUILD_TOP"))?,
    );
    path = path
        .canonicalize()
        .map_err(|_| "failed to canonicalize ANDROID_BUILD_TOP")?;
    path.push(&info.path[0]);
    println!("cd \"{}\"", path.to_string_lossy());
    Ok(())
}

fn main() -> Result<(), Box<std::error::Error>> {
    let matches = App::new("ash")
        .arg(Arg::with_name("config").long("config").takes_value(true))
        .arg(Arg::with_name("modinfo").long("modinfo").takes_value(true))
        .subcommand(SubCommand::with_name("init"))
        .subcommand(SubCommand::with_name("envsetup"))
        .subcommand(
            SubCommand::with_name("show").arg(Arg::with_name("module-or-group").required(true)),
        )
        .subcommand(SubCommand::with_name("cd").arg(Arg::with_name("module").required(true)))
        .get_matches();
    let subcommand = matches.subcommand();

    // special commands, part 1
    if let ("init", _) = subcommand {
        return cmd_init();
    }

    // special commands, part 2
    let config_path = match matches.value_of("config") {
        Some(path) => PathBuf::from_str(path).unwrap(),
        None => config::find_default_config_file(std::env::current_dir()?)?,
    };
    if let ("envsetup", _) = subcommand {
        let envsetup = config::parse_envsetup(&config_path)?;
        return cmd_envsetup(&envsetup);
    }

    // all other commands (that depend on module-info.json)
    let groups = config::parse_groups(&config_path)?;
    let modinfo_path = match matches.value_of("modinfo") {
        Some(path) => PathBuf::from_str(path).unwrap(),
        None => config::find_default_modinfo_file()?,
    };
    let file = File::open(modinfo_path)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };
    let modules = ModuleInfoSet::new(&mmap[..]);
    match subcommand {
        ("show", Some(args)) => cmd_show(&modules, &groups, args),
        ("cd", Some(args)) => cmd_cd(&modules, &groups, args),
        _ => Err(matches.usage().into()),
    }
}
