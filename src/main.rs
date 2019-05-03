mod modinfo;

use memmap::MmapOptions;
use modinfo::ModuleInfoSet;
use std::fs::File;

fn main() -> Result<(), Box<std::error::Error>> {
    let mut args = std::env::args();
    if args.len() < 3 {
        return Err("usage: ash <path-to-json> <module> [<module> [...]]".into());
    }
    args.next(); // skip binary name
    let file = File::open(args.next().unwrap())?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };
    let modules = ModuleInfoSet::new(&mmap[..]);
    for module_name in args {
        let info = modules
            .find(&module_name)
            .ok_or_else(|| format!("{}: module not found", module_name))?;
        println!("{:#?}", info);
    }
    Ok(())
}
