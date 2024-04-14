use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use thepipelinetool_task::Task;

use crate::pipeline_options::PipelineOptions;

#[derive(Deserialize, Serialize, Debug)]
pub struct Pipeline {
    // pub name: String,
    pub path: String,
    pub options: PipelineOptions,
    pub tasks: Vec<Task>,
    pub edges: HashSet<(usize, usize)>,
}
