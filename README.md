# kiro-generator

[![Crates.io](https://img.shields.io/crates/v/kiro-generator.svg)](https://crates.io/crates/kiro-generator)
[![Docs.rs](https://docs.rs/kiro-generator/badge.svg)](https://docs.rs/kiro-generator)
[![ci](https://github.com/dougEfresh/kiro-generator/actions/workflows/test.yml/badge.svg)](https://github.com/dougEfresh/kiro-generator/actions/workflows/test.yml)
[![Cov](https://codecov.io/github/dougEfresh/kiro-generator/graph/badge.svg?token=dILa1k9tlW)](https://codecov.io/github/dougEfresh/kiro-generator)

## About

`kiro-generator` (aka `kg`) is a tool for managing and generating [Kiro](https://kiro.dev/docs/) custom agent [files](https://kiro.dev/docs/cli/custom-agents/).

Stop writing JSON. Define your Kiro agents in TOML with inheritance, templates, and reusable components.

## Why?

- **Composable**: Build agents from reusable templates
- **Type-safe**: TOML validation with JSON schema support
- **Shareable**: Package and distribute complete agent configurations
- **Hierarchical**: Global and project-specific agents
- **DRY**: Inherit and extend configurations

## Quick Start

```shell
# Initialize configuration
kg init

# Edit your agent manifest
$EDITOR ~/.kiro/generators/manifests/kg.toml

# Validate configuration
kg validate

# Generate agent JSON files
kg generate
```

See [documentation](https://kg.com) for detailed guides and examples.

---

## Installation

```shell
cargo install kiro-generator
```

---

## Development

See [DEVELOPMENT.md](DEVELOPMENT.md) for build requirements and development workflow.

---

## License

 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md).
