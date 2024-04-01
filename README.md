<h1 align=center>thepipelinetool</h1>
<!-- <h4 align="center"></h4> -->

<div align="center">
  <a href="https://github.com/thepipelinetool/thepipelinetool/releases" target="_blank">
    <img alt="GitHub Release" src="https://img.shields.io/github/v/release/thepipelinetool/thepipelinetool" />
  </a>
  <a href="https://crates.io/crates/thepipelinetool" target="_blank">
    <img src="https://img.shields.io/crates/v/thepipelinetool" />
  </a>
  <a href="https://github.com/thepipelinetool/thepipelinetool/actions/workflows/build.yml" target="_blank">
    <img src="https://github.com/thepipelinetool/thepipelinetool/actions/workflows/build.yml/badge.svg" />
  </a>
  <a href="https://github.com/thepipelinetool/thepipelinetool/actions/workflows/release.yml" target="_blank">
    <img src="https://github.com/thepipelinetool/thepipelinetool/actions/workflows/release.yml/badge.svg" />
  </a>  
</div>

</br>

`tpt` runs ensures orderly pipeline orchestration according to task [dependencies](#usage). Pipelines can be [deployed](https://github.com/thepipelinetool/thepipelinetool/tree/main/thepipelinetool_server) to enjoy scheduling, catchup, retries, and live task monitoring with a modern web UI.

## Features
- *Simple Usage* - Write YAML or Rust code and let `thepipelinetool` handle execution order, concurrent execution, timeouts, and retries
- *Special Tasks* - Create multiple [Dynamic](#dynamic-tasks) tasks from upstream results or a control flow using [Branching](#branching-tasks) tasks
- *Easy Testing* - Test both YAML and Rust code locally
- *Safety and Reliability* - Rust's compile-time checks ensure code safety and prevent common bugs

## Contents
- [Installation](#installation)
- [Usage](#usage)
- [Documentation](#documentation)
- [Examples](#examples)
- [Deployment](#deployment)
- [Advanced](#advanced)
- [License](#license)

## Installation
```bash
cargo install thepipelinetool_cli
```

## Usage
```bash
tpt your_dag run in_memory
```
## Examples
Find more examples [here](https://github.com/thepipelinetool/thepipelinetool/tree/main/thepipelinetool/examples)

## Deployment
To deploy DAGs, the compiled binaries must be placed inside `DAGS_DIR` for both the server and workers to access.
Visit the [template](https://github.com/thepipelinetool/thepipelinetool_template) project for the docker-compose.yml example

## Advanced
Get started by cloning the [template](https://github.com/thepipelinetool/thepipelinetool_template) project
```bash
git clone https://github.com/thepipelinetool/thepipelinetool_template
```

Or create a new project and add `thepipelinetool` dependency
```bash
mkdir your_dag_name
cd your_dag_name
cargo init --bin
cargo add thepipelinetool
```
Add the following to `src/main.rs`
```rust 
use thepipelinetool::prelude::*;

#[dag]
fn main() {
    // define your tasks here
}
```

## License
AGPLv3
