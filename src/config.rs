use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::{fs, io};

#[derive(Deserialize, Debug)]
pub struct ModuleGroup {
    pub name: String,
    pub modules: Vec<String>,
    pub tests: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
struct Config {
    envsetup: String,
    #[serde(rename(deserialize = "module-group"))]
    groups: Option<Vec<ModuleGroup>>,
}

fn parse(path: &Path) -> Result<Config, String> {
    let toml = fs::read_to_string(path)
        .or_else(|_| Err(format!("failed to open config {}", path.to_string_lossy())))?;
    toml::from_str(&toml).or_else(|e| {
        Err(format!(
            "failed to parse config {}: {}",
            path.to_string_lossy(),
            e
        ))
    })
}

pub fn parse_envsetup<P: AsRef<Path>>(config_path: P) -> Result<String, String> {
    Ok(parse(config_path.as_ref())?.envsetup)
}

pub fn parse_groups<P: AsRef<Path>>(config_path: P) -> Result<Vec<ModuleGroup>, String> {
    let config = parse(config_path.as_ref())?;
    let groups = config.groups.unwrap_or_default();
    for g in &groups {
        if !g.name.starts_with(':') {
            return Err(format!("{}: module-group.name must begin with ':'", g.name));
        }
    }
    Ok(groups)
}

pub fn find_default_config_file(mut root: PathBuf) -> Result<PathBuf, io::Error> {
    loop {
        let path = root.join("ash.toml");
        if path.exists() {
            return Ok(path);
        }
        if !root.pop() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "ash.toml not found",
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_envsetup() {
        let envsetup = super::parse_envsetup("tests/ash.toml").unwrap();
        let lines = envsetup.lines().collect::<Vec<_>>();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "source build/envsetup.sh");
        assert_eq!(lines[1], "lunch aosp_x86_64-eng");
    }

    #[test]
    fn test_parse_groups() {
        let groups = super::parse_groups("tests/ash.toml").unwrap();
        assert_eq!(groups.len(), 2);
        let mod1 = groups.iter().find(|item| item.name == ":idmap").unwrap();
        assert_eq!(mod1.modules.len(), 3);
        assert_eq!(mod1.tests, None);
    }

    #[test]
    fn test_find_default_config_file() {
        assert!(super::find_default_config_file("tests".into()).is_ok());
        assert!(super::find_default_config_file("/".into()).is_err());
    }
}
