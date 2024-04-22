use std::path::Path;

#[derive(PartialEq, Eq)]
pub enum SourceType {
    Exe,
    Yaml,
    Raw,
    None,
}

impl SourceType {
    pub fn from_source(source: Option<&String>) -> Self {
        if let Some(source) = source {
            if source == "" {
                SourceType::None
            } else {
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
        } else {
            SourceType::None
        }
    }
}
