<h1 align=center>tpt</h1>
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
  <a href="https://github.com/thepipelinetool/thepipelinetool/actions/workflows/ci.yml" target="_blank">
    <img src="https://github.com/thepipelinetool/thepipelinetool/actions/workflows/ci.yml/badge.svg" />
  </a>
  <a href="https://github.com/thepipelinetool/thepipelinetool/actions/workflows/release.yml" target="_blank">
    <img src="https://github.com/thepipelinetool/thepipelinetool/actions/workflows/release.yml/badge.svg" />
  </a>
</div>

</br>

Orchestrate your pipelines using `tpt`. [Deploy](https://github.com/thepipelinetool/thepipelinetool/tree/main/thepipelinetool_server) them for scheduling, catchup, retries, and live monitoring.


## Features
- write your pipeline YAML or Rust code and let `tpt` handle execution order, parallelism, timeouts, and retries
- create multiple dynamic tasks from upstream results or control flow using branching tasks
- easy testing
  - test both YAML and Rust pipelines locally
  - rust's compile-time checks ensure code safety and prevent common bugs

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
cargo install thepipelinetool
```

## Usage
```
Usage: tpt [pipeline] <COMMAND>

Commands:
  describe  Describe pipeline tasks or edges
  check     Check for circular depencencies
  graph     Displays graph
  tree      Displays tree
  run       Run complete pipeline or function by name
  help      Print this message or the help of the given subcommand(s)

Arguments:
  [pipeline]

Options:
  -h, --help     Print help
  -V, --version  Print version
```
## Examples
Find more examples [here](https://github.com/thepipelinetool/thepipelinetool/tree/main/thepipelinetool/examples)

## Deployment
The pipeline files must be placed inside `PIPELINES_DIR` for both the server and workers to access.
Visit the [template](https://github.com/thepipelinetool/thepipelinetool_template) project for the docker-compose.yml example

## Advanced
Get started by cloning the [template](https://github.com/thepipelinetool/thepipelinetool_template) project
```bash
git clone https://github.com/thepipelinetool/thepipelinetool_template
```

Or create a new project and add `thepipelinetool_core` dependency
```bash
mkdir your_pipeline_name
cd your_pipeline_name
cargo init --bin
cargo add thepipelinetool_core
```
Add the following to `src/main.rs`
```rust
use thepipelinetool_core::{prelude::*, tpt};

#[tpt::main]
fn main() {
    // define your tasks here
}
```
Find advanced usage [here](https://github.com/thepipelinetool/thepipelinetool/tree/main/thepipelinetool_core)

## License
AGPLv3
