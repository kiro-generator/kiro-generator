# kiro-generator

[![Crates.io](https://img.shields.io/crates/v/kiro-generator.svg)](https://crates.io/crates/kiro-generator)
[![Docs.rs](https://docs.rs/kiro-generator/badge.svg)](https://docs.rs/kiro-generator)
[![ci](https://github.com/CarteraMesh/kiro-generator/actions/workflows/test.yml/badge.svg)](https://github.com/CarteraMesh/kiro-generator/actions/workflows/test.yml)
[![Cov](https://codecov.io/github/CarteraMesh/kiro-generator/graph/badge.svg?token=dILa1k9tlW)](https://codecov.io/github/CarteraMesh/kiro-generator)

## About

`kiro-generator` (aka `kg`) is a tool for managing and generating [Kiro](https://kiro.dev/docs/) custom agent [files](https://kiro.dev/docs/cli/custom-agents/).

## Quick Start 

1. Initialize your config

```shell
$ kg init

Created /home/user/.kiro/generators/kg.kdl
Created /home/user/.kiro/generators/default.kdl
Created /home/user/.kiro/generators/example.kdl

✓ Initialized kg configuration in /home/user/.kiro/generators

```

2. Review/Modify/Add to your config

See [documentation](https://kg.cartera-mesh.com) for further info and examples

3. Validate your config

```shell
$ kg validate 
╭────────────────────┬─────┬─────────────────┬────────────────────────────────────────────────┬────────────────────┬────────┬────────┬────────╮
│ Agent 🤖 (PREVIEW) ┆ Loc ┆ MCP 💻          ┆ Allowed Tools ⚙️                               ┆ Resources 📋       ┆    Overrides             │
╞════════════════════╪═════╪═════════════════╪════════════════════════════════════════════════╪════════════════════╪══════════════════════════╡
│ default            ┆ 📁  ┆                 ┆ knowledge, read, web_search                    ┆ - file://README.md ┆                          │
│                    ┆     ┆                 ┆                                                ┆ - file://AGENTS.md ┆                          │
│                    ┆     ┆                 ┆                                                ┆                    ┆                          │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ rust               ┆ 📁  ┆ cargo, rustdocs ┆ @cargo, @rustdocs, knowledge, read, web_search ┆ - file://README.md ┆                          │
│                    ┆     ┆                 ┆                                                ┆ - file://AGENTS.md ┆                          │
│                    ┆     ┆                 ┆                                                ┆ - file://RUST.md   ┆                          │
│                    ┆     ┆                 ┆                                                ┆                    ┆                          │
╰────────────────────┴─────┴─────────────────┴────────────────────────────────────────────────┴────────────────────┴──────────────────────────╯

🎉 Config is valid
→ Run kg generate to generate agent files
```

4. Generate 

```shell
$ kg generate
```

profit

---

## Installation

```shell
cargo install kiro-generator
```


---

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
