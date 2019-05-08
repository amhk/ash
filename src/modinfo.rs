use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use serde_json;
use std::convert::TryFrom;

#[derive(Deserialize, Debug)]
pub struct ModuleInfo {
    pub module_name: String,
    pub path: Vec<String>,
    pub installed: Vec<String>,
}

impl TryFrom<&str> for ModuleInfo {
    type Error = serde_json::error::Error;
    fn try_from(src: &str) -> Result<Self, Self::Error> {
        serde_json::from_str(src)
    }
}

pub struct ModuleInfoSet<'a> {
    data: &'a [u8],
}

impl<'a> ModuleInfoSet<'a> {
    pub fn new(data: &'a [u8]) -> ModuleInfoSet<'a> {
        ModuleInfoSet { data }
    }

    pub fn find(&self, module_name: &str) -> Option<ModuleInfo> {
        let lines = unsafe { std::str::from_utf8_unchecked(&self.data[2..self.data.len() - 2]) }
            .lines()
            .collect::<Vec<_>>();
        let index = match lines.binary_search_by(|x| {
            let mut iter = x.split('\"');
            iter.next();
            let key = iter.next().unwrap_or("");
            key.cmp(module_name)
        }) {
            Err(_) => return None,
            Ok(index) => index,
        };

        // convert '  "foo": { ... },' to just '{ ... }'
        lazy_static! {
            static ref REGEX: Regex = Regex::new(r#"\s*".*?"\s*:\s*(\{.*\}),?"#).unwrap();
        }
        let json = REGEX.captures(lines[index])?.get(1)?.as_str();

        ModuleInfo::try_from(json).ok()
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    const MODULE_BAR_JSON: &'static str = r#"{
        "installed": [],
        "module_name": "bar",
        "path": ["path/to/bar"]
    }"#;

    const MODULE_FOO_JSON: &'static str = r#"{
        "installed": [
            "out/target/foo.apk",
            "out/target/foo.so"
        ],
        "module_name": "foo",
        "path": ["path/to/foo"]
    }"#;

    #[test]
    fn test_moduleinfo_from_json() {
        let info = super::ModuleInfo::try_from(MODULE_FOO_JSON).unwrap();
        assert_eq!(info.module_name, "foo");
        assert_eq!(info.path.len(), 1);
        assert_eq!(info.installed.len(), 2);
    }

    #[test]
    fn test_moduleinfoset_from_json() {
        let json = format!(
            "{{\n  \"bar\": {},\n  \"foo\": {}\n }}",
            MODULE_BAR_JSON.replace("\n", ""),
            MODULE_FOO_JSON.replace("\n", "")
        );
        let modules = super::ModuleInfoSet::new(json.as_ref());

        let foo = modules.find("foo").unwrap();
        assert_eq!(foo.module_name, "foo");

        let bar = modules.find("bar").unwrap();
        assert_eq!(bar.module_name, "bar");

        assert!(modules.find("does-not-exist").is_none());
    }
}
