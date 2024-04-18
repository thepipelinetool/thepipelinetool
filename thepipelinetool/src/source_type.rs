use std::path::Path;

#[derive(PartialEq, Eq)]
pub enum SourceType {
    Exe,
    Yaml,
    Raw,
}

impl SourceType {
    pub fn from_source(source: &str) -> Self {
        let p = Path::new(source);
        if p.exists() {
            match p.extension() {
                Some(ext) => match ext.to_str().unwrap() {
                    "yaml" => SourceType::Yaml,
                    _ => panic!("unknown extenstion type"),
                },
                None => SourceType::Exe,
            }
        } else {
            SourceType::Raw
        }
    }
}
