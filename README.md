# kiro-generator

[![Crates.io](https://img.shields.io/crates/v/kiro-generator.svg)](https://crates.io/crates/kiro-generator)
[![Docs.rs](https://docs.rs/kiro-generator/badge.svg)](https://docs.rs/kiro-generator)
[![CI](https://github.com/CarteraMesh/kiro-generator/workflows/test/badge.svg)](https://github.com/CarteraMesh/kiro-generator/actions)
[![Cov](https://codecov.io/github/CarteraMesh/kiro-generator/graph/badge.svg?token=dILa1k9tlW)](https://codecov.io/github/CarteraMesh/kiro-generator)

## About

`kiro-generator` is a tool for managing and generating [Kiro](https://kiro.dev/docs/) custom agent [files](https://kiro.dev/docs/cli/custom-agents/).

## Quick Start 

1. Define your agents in `~/.kiro/generators/kg.toml`

`cat ~/.kiro/generators/kg.toml`

```toml
[agents]
default = {  } # default is the agent name
rust = { inherits = ["default"] }  # rust agent config is merged with default
```

`cat ~/.kiro/generators/default.toml`

```toml
description = "Default agent"
tools = ["*"]
allowedTools = ["read", "knowledge", "web_search"]
resources = ["file://README.md", "file://AGENTS.md"]
[toolsSettings.execute_bash]
allowedCommands = ["git status", "git fetch", "git diff .*" ]
deniedCommands = ["git commit .*", "git push .*" ]
autoAllowReadonly = true
```

`cat ~/.kiro/generators/rust.toml`

```toml
description = "General Rust agent"
resources = ["file://~/.kiro/resources/rust.md"]
allowedTools = [ "@rustdocs", "@cargo" ] # also ["read", "knowledge", "web_search"] from default.toml
[mcpServers]
rustdocs = { type = "stdio" , command = "rust-docs-mcp", timeout = 10000 }
cargo = {  command = "cargo-mcp" , timeout = 120000  }

[toolsSettings]
[toolsSettings.execute_bash]
allowedCommands = ["cargo .+" ] ## inherits allowedCommands from default.toml
deniedCommands = ["cargo publish .*"] ## inherits allowedCommands from default.toml
```


```shell
$ kg generate 
## TODO Show command output
```

---

## Features

## Config Hierarchy 

KG provides a hierarchical configuration system that allows you to define and override settings at different levels. The schemais the same as `kiro-cli` JSON's format but defined as TOML, with a few added fields explained below

### Agent Declaration

By default Agents can be declared globally `~/.kiro/generators/kg.toml` or locally `.kiro/generators/kg.toml` . If both are present, the local configuration takes precedence, however both configurations are merged together. You can use `--local` argument to ignore the global configuration.

### Agent Configuration

From least precedence to highest precedence (local overrides global): 

`~/.kiro/generators/<agent-name>.toml` ->
`~/.kiro/generators/kg.toml` ->  
`.kiro/generators/<agent-name>.toml` ->
`.kiro/generators/kg.toml`


The `<agent-name>.toml` files are optional. You can defined your agent entirely in  `~/.kiro/generators/kg.toml` or `.kiro/generators/kg.toml` files (or both)

### Defining Inheritance

`kg.toml` defined the relationship or inheritance of your agents. You can define the relationship between agents using the `inherit` field. For example, if you have an agent named `default` and another agent named `rust`, you can define the relationship between them as follows:

```toml
[agents]
default = { inherits = [] } # parent 
rust = { inherits = ["default"] } # child
```

`kg` will then look for agent configuration files in the following order:

- `~/.kiro/generators/<agent-name>.toml`  # e.g. rust.toml
- `.kiro/generators/<agent-name>.toml`  # e.g. rust.toml

Both can be present and will be merged together.

### "Inline" Agent Configuration

You can define agent properties "inline" using only `kg.toml` files: 

```toml
[agents.default]
inherits = []
allowedTools = ["read", "knowledge"]


[agents.rust]
inherits= ["default"]
allowedTools = [ "@rustdocs", "@cargo" ]
```

In this example, the allowedTools field is inherited from the default agent, and the rust agent adds two additional tools: `@rustdocs` and `@cargo`.

### Skeletons 

`Skeletons` are like agent templates. `kg` will skip agent `JSON` files. You can use them building blocks or components to derive other real agents.

`kg.toml`: 

```toml

[agents.default]
inherits = []
allowedTools = ["read", "knowledge"]

[agents.git-readonly]
skeleton = true # do not generate JSON amazon-q agent config
[toolsSettings.execute_bash]
allowedCommands = ["git status .*", "git fetch .*", "git diff .*" , "git log .*"]

[agents.git-write]
skeleton = true
[toolsSettings.execute_bash]
allowedCommands = ["git .*"]

[agents.rust]
inherits = ["default", "git-readonly"]
allowedTools = [ "@rustdocs", "@cargo" ]

[agents.dependabot]
inherits = ["default", "git-write", "rust"]
```

The `dependabot` agent will be able to use any git command. 

---

## Installation

### Cargo

* Install the rust toolchain in order to have cargo installed by following
  [this](https://www.rust-lang.org/tools/install) guide.
* run `cargo install kiro-generator`


## Development

### Prerequisites

- **Rust Nightly**: Required for code formatting with advanced features
  ```bash
  rustup install nightly
  ```

### Getting Started

1. **Clone the repository**
   ```bash
   git clone https://github.com/CarteraMesh/kiro-generator.git
   cd kiro-generator
   ```

2. **Build and test**
   ```bash
   # Build the project
   cargo build

   # Run tests (requires valid Fireblocks credentials in .env)
   cargo test

   # Format code (requires nightly)
   cargo +nightly fmt --all
   ```

### Code Formatting

This project uses advanced Rust formatting features that require nightly:

```bash
# Format all code
cargo +nightly fmt --all

# Check formatting
cargo +nightly fmt --all -- --check
```

## License

 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md).
