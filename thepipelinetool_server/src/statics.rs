// use std::{
//     collections::{HashMap, HashSet},
//     path::PathBuf,
//     process::Command,
//     sync::{Arc, OnceLock},
// };

// use parking_lot::Mutex;
// use saffron::Cron;
// use thepipelinetool_core::dev::Task;
// use thepipelinetool_runner::{
//     get_pipeline_path_buf_by_name, get_pipelines_dir, pipeline_options::PipelineOptions,
// };

// use crate::env::get_tpt_command;
// use anyhow::anyhow;
// use anyhow::Result;

// type StaticServerTasks = Arc<Mutex<HashMap<String, Vec<Task>>>>;
// type StaticServerHashes = Arc<Mutex<HashMap<String, String>>>;
// type StaticServerEdges = Arc<Mutex<HashMap<String, HashSet<(usize, usize)>>>>;
// type StaticServerPipelineOptions = Arc<Mutex<HashMap<String, PipelineOptions>>>;

// static TASKS: OnceLock<StaticServerTasks> = OnceLock::new();
// static HASHES: OnceLock<StaticServerHashes> = OnceLock::new();
// static EDGES: OnceLock<StaticServerEdges> = OnceLock::new();
// static PIPELINE_OPTIONS: OnceLock<StaticServerPipelineOptions> = OnceLock::new();

// pub fn _get_default_tasks(pipeline_name: &str) -> Result<Vec<Task>> {
//     let mut tasks = TASKS
//         .get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
//         .lock();

//     if !tasks.contains_key(pipeline_name) {
//         let pipeline_source = get_pipeline_path_buf_by_name(pipeline_name)?;

//         let output = Command::new(get_tpt_command())
//             .arg(pipeline_source)
//             .arg("describe")
//             .arg("tasks")
//             .output()?;

//         tasks.insert(
//             pipeline_name.to_owned(),
//             serde_json::from_str(&String::from_utf8_lossy(&output.stdout))?,
//         );
//     }

//     Ok(tasks
//         .get(pipeline_name)
//         .ok_or(anyhow!(format!("{pipeline_name} does not exist")))?
//         .clone())
// }

// pub fn _get_hash(pipeline_name: &str) -> Result<String> {
//     let mut hashes = HASHES
//         .get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
//         .lock();

//     if !hashes.contains_key(pipeline_name) {
//         let pipelines_dir = &get_pipelines_dir();
//         let path: PathBuf = [pipelines_dir, pipeline_name].iter().collect();
//         let output = Command::new(get_tpt_command())
//             .arg(path)
//             .arg("describe")
//             .arg("hash")
//             .output()?;

//         hashes.insert(
//             pipeline_name.to_owned(),
//             String::from_utf8_lossy(&output.stdout).to_string(),
//         );
//     } else {
//         return Err(anyhow!(format!("{pipeline_name} does not exist")));
//     }

//     Ok(hashes
//         .get(pipeline_name)
//         .ok_or(anyhow!(format!("{pipeline_name} does not exist")))?
//         .to_string())
// }

// pub fn _get_default_edges(pipeline_name: &str) -> Result<HashSet<(usize, usize)>> {
//     let mut edges = EDGES
//         .get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
//         .lock();

//     if !edges.contains_key(pipeline_name) {
//         let pipeline_source = get_pipeline_path_buf_by_name(pipeline_name)?;

//         let output = Command::new(get_tpt_command())
//             .arg(pipeline_source)
//             .arg("describe")
//             .arg("edges")
//             .output()?;

//         edges.insert(
//             pipeline_name.to_owned(),
//             serde_json::from_str(&String::from_utf8_lossy(&output.stdout))?,
//         );
//     }

//     Ok(edges
//         .get(pipeline_name)
//         .ok_or(anyhow!(format!("{pipeline_name} does not exist")))?
//         .clone())
// }

// pub fn _get_options(pipeline_name: &str) -> Result<PipelineOptions> {
//     let mut pipeline_options = PIPELINE_OPTIONS
//         .get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
//         .lock();

//     if !pipeline_options.contains_key(pipeline_name) {
//         let pipeline_source = get_pipeline_path_buf_by_name(pipeline_name)?;

//         let output = Command::new(get_tpt_command())
//             .arg(pipeline_source)
//             .arg("describe")
//             .arg("options")
//             .output()?;

//         let options: PipelineOptions =
//             serde_json::from_str(&String::from_utf8_lossy(&output.stdout))?;

//         if let Some(schedule) = &options.schedule {
//             schedule.parse::<Cron>().map_err(|e| anyhow!(e))?;
//         }

//         pipeline_options.insert(pipeline_name.to_owned(), options);
//     }

//     Ok(pipeline_options
//         .get(pipeline_name)
//         .ok_or(anyhow!(format!("{pipeline_name} does not exist")))?
//         .clone())
// }
