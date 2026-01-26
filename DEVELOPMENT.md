# Development

## Prerequisites

- **Rust Nightly**: Required for code formatting with advanced features
  ```bash
  rustup install nightly
  ```

## Getting Started

1. **Clone the repository**
   ```bash
   git clone https://github.com/dougEfresh/kiro-generator.git
   cd kiro-generator
   ```

2. **Build and test**
   ```bash
   # Build the project
   cargo build

   # Run tests
   cargo test

   # Format code (requires nightly)
   cargo +nightly fmt --all
   ```

## Code Formatting

This project uses advanced Rust formatting features that require nightly:

```bash
# Format all code
cargo +nightly fmt --all

# Check formatting
cargo +nightly fmt --all -- --check
```
