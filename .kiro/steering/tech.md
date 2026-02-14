# Technology Stack

## Language

Rust (edition 2024). Binary crate, not a library.

## Core Dependencies

| Crate | Purpose |
|---|---|
| `clap` (derive) | CLI argument parsing with subcommands |
| `color-eyre` | Error reporting with context chains and suggestions |
| `facet` / `facet-toml` / `facet-json` | Serialization/deserialization (replaces serde) |
| `facet-json-schema` | JSON schema generation from Rust types |
| `facet-value` | Dynamic value representation |
| `tokio` | Async runtime (fs, macros, rt-multi-thread) |
| `tracing` / `tracing-subscriber` | Structured logging with env-filter |
| `serde_json` | JSON output (agent files are JSON) |
| `rediff` | Diff rendering for `kg diff` |
| `super-table` | Formatted table output |
| `colored` | Terminal color support |
| `jsonschema` | Schema validation |

### Linux-only

| Crate | Purpose |
|---|---|
| `notify-rust` | Desktop notifications |
| `zbus` | D-Bus for systemd integration |

## Serialization: facet, NOT serde

This project uses [facet](https://facet.rs) for all (de)serialization. Do not use serde derives or serde attributes. Key differences:

- `#[derive(Facet)]` instead of `#[derive(Serialize, Deserialize)]`
- `#[facet(default)]`, `#[facet(rename = "...")]`, `#[facet(deny_unknown_fields)]`
- Deserialization via `facet_toml::from_str()` and serialization via `facet_json`

## Error Handling

`color_eyre::Result<T>` everywhere. The project type alias is `pub type Result<T> = color_eyre::Result<T>`.

- Use `.wrap_err()` / `.wrap_err_with()` to add context at each meaningful layer
- Context should describe what the code was trying to do, with file paths and identifiers
- Use `.suggestion()` for user-actionable hints
- Never use `.unwrap()` or `.expect()` outside of tests

## Testing

- `cargo test` (no `--lib` flag -- this is a binary crate)
- `#[test_log::test]` over `#[test]` for tracing output during tests
- `#[test_log::test(tokio::test)]` for async tests
- Always return `Result<()>` from tests, never `.unwrap()`
- The `Fs` abstraction provides chroot-based filesystem isolation in tests

## Formatting and Linting

- Code must pass `cargo +nightly fmt` and `cargo clippy` (enforced via hooks)
- Rust naming conventions: `snake_case` functions, `PascalCase` types, `SCREAMING_SNAKE_CASE` constants

## Constraints

- Simplicity over performance -- clean, maintainable code wins over optimization
- Prefer `?` operator for error propagation
- Prefer iterators over explicit loops where it improves readability
- Avoid `unsafe` code
- Avoid unnecessary `.clone()` but don't contort the code to eliminate them
- No global mutable state
- Cargo target directory is non-standard: `~/.cache/cargo/target/`
