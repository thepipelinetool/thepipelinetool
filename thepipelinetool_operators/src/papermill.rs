use serde_json::Value;
use thepipelinetool_utils::run_bash_commmand;

// #[derive(Serialize, Deserialize, Clone, Debug)]
// pub struct PapermillConfig {
//     notebook_path: String,

//     output_path: Option<String>,

//     #[serde(default)]
//     pub options: Option<Value>,

//     #[serde(default)]
//     pub parameters: Option<Value>,
// }

// impl Default for PapermillConfig {
//     fn default() -> Self {
//         Self {
//             options
//         }
//     }
// }
// fn unescape(s: &str) -> String {
//     serde_json::from_str(&format!("{}", s)).unwrap()
// }
pub fn papermill_operator(config: Value) {
    let mut args = vec!["papermill"];
    // let params: Vec<String> = if let Some(options) = &config.options {
    //     options
    //         .as_array()
    //         .unwrap()
    //         .iter()
    //         .map(|s| serde_json::to_string(s).unwrap())
    //         .collect()
    // } else {
    //     vec![]
    // };

    let params: Vec<(String, String)> = config
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| {
            (
                k.to_string(),
                if v.is_array() || v.is_object() {
                    // println!("'{}'", v);
                    // let deserialized: String = serde_json::from_str(&v.to_string()).unwrap();

                    // deserialized
                    // println!("'{}'", serde_json::to_string(v).unwrap());

                    // unescape(&v.to_string())
                    v.to_string()
                } else {
                    v.as_str().unwrap().to_string()
                },
            )
        })
        .collect();
    for (k, v) in &params {
        args.push("-p");
        args.push(k);
        args.push(r#v);
    }
    args.push(&config["notebook_path"].as_str().unwrap());

    if let Some(output_path) = &config["output_path"].as_str() {
        args.push(output_path);
    }

    run_bash_commmand(&args, true);
}
