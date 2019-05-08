mod config;
mod modinfo;

use clap::{App, Arg, ArgMatches, SubCommand};
use config::ModuleGroup;
use memmap::MmapOptions;
use modinfo::ModuleInfoSet;
use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;

enum ShellReturnCode {
    Print = 0,
    Execute = 27,
    Error = 1,
}

fn cmd_init() -> Result<ShellReturnCode, String> {
    // TODO: println!("{}", include_str!("../scripts/init.sh"));
    Ok(ShellReturnCode::Error)
}

fn cmd_envsetup(_script: &str) -> Result<ShellReturnCode, String> {
    Ok(ShellReturnCode::Error)
}

fn cmd_show(
    modules: &ModuleInfoSet,
    groups: &[ModuleGroup],
    args: &ArgMatches,
) -> Result<ShellReturnCode, String> {
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
    Ok(ShellReturnCode::Print)
}

fn cmd_cd(
    modules: &ModuleInfoSet,
    _groups: &[ModuleGroup],
    args: &ArgMatches,
) -> Result<ShellReturnCode, String> {
    let module_name = args.value_of("module").unwrap();
    let info = modules
        .find(&module_name)
        .ok_or_else(|| format!("{}: module not found", module_name))?;
    assert!(!info.path.is_empty()); // required by parse step
    if info.path.len() > 1 {
        return Err("multiple paths".to_string());
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
    Ok(ShellReturnCode::Execute)
}

fn exec_cmd(args: &ArgMatches) -> Result<ShellReturnCode, String> {
    let subcommand = args.subcommand();

    // special commands, part 1
    if let ("init", _) = subcommand {
        return cmd_init();
    }

    // special commands, part 2
    let config_path = match args.value_of("config") {
        Some(path) => PathBuf::from_str(path).unwrap(),
        None => {
            config::find_default_config_file(std::env::current_dir().map_err(|x| x.to_string())?)
                .map_err(|x| x.to_string())?
        }
    };
    if let ("envsetup", _) = subcommand {
        let envsetup = config::parse_envsetup(&config_path)?;
        return cmd_envsetup(&envsetup);
    }

    // all other commands (that depend on module-info.json)
    let groups = config::parse_groups(&config_path)?;
    let modinfo_path = match args.value_of("modinfo") {
        Some(path) => PathBuf::from_str(path).unwrap(),
        None => config::find_default_modinfo_file().map_err(|x| x.to_string())?,
    };
    let file = File::open(modinfo_path).map_err(|x| x.to_string())?;
    let mmap = unsafe { MmapOptions::new().map(&file).map_err(|x| x.to_string())? };
    let modules = ModuleInfoSet::new(&mmap[..]);
    match subcommand {
        ("show", Some(args)) => cmd_show(&modules, &groups, args),
        ("cd", Some(args)) => cmd_cd(&modules, &groups, args),
        _ => Err(format!("{}: unknown subcommand", subcommand.0)),
    }
}

fn main() {
    let args = App::new("ash")
        .arg(Arg::with_name("config").long("config").takes_value(true))
        .arg(Arg::with_name("dry-run").long("dry-run"))
        .arg(Arg::with_name("modinfo").long("modinfo").takes_value(true))
        .subcommand(SubCommand::with_name("init"))
        .subcommand(SubCommand::with_name("envsetup"))
        .subcommand(
            SubCommand::with_name("show").arg(Arg::with_name("module-or-group").required(true)),
        )
        .subcommand(SubCommand::with_name("cd").arg(Arg::with_name("module").required(true)))
        .get_matches();

    let dry_run = args.is_present("dry-run");

    match (dry_run, exec_cmd(&args)) {
        (_, Ok(ShellReturnCode::Print)) | (true, Ok(ShellReturnCode::Execute)) => {
            std::process::exit(ShellReturnCode::Print as i32);
        }
        (false, Ok(ShellReturnCode::Execute)) => {
            std::process::exit(ShellReturnCode::Execute as i32);
        }
        (_, Ok(ShellReturnCode::Error)) => {
            std::process::exit(ShellReturnCode::Error as i32);
        }
        (_, Err(s)) => {
            eprintln!("{}", s);
            std::process::exit(ShellReturnCode::Error as i32);
        }
    }
}
