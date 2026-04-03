URL Source: https://facet.rs/showcases/derive/
Scraped: 2026-02-19T21:32:38Z

---

Title: Derive Macro Diagnostics - facet

URL Source: https://facet.rs/showcases/derive/

Markdown Content:
The `#[derive(Facet)]` macro provides helpful compile-time error messages when attributes are used incorrectly. This showcase demonstrates the various error scenarios and their diagnostics.

Representation Errors
---------------------

### Conflicting repr: C and Rust

Using both `#[repr(C)]` and `#[repr(Rust)]` is not allowed.

Facet defers to rustc's E0566 error for this - no duplicate diagnostic.

#### Rust Input

```
use facet::Facet;

#[derive(Facet)]
#[repr(C, Rust)]
enum Status {
    Active,
    Inactive,
}

fn main() {}
```

#### Compiler Error

```
error[E0566]: conflicting representation hints
 --> src/main.rs:4:8
  |
4 | #[repr(C, Rust)]
  |        ^  ^^^^
For more information about this error, try `rustc --explain E0566`.
error: could not compile `test` (bin "test") due to 1 previous error
```

### Conflicting repr: C and transparent

Combining `#[repr(C)]` with `#[repr(transparent)]` is not valid.

Facet defers to rustc's E0692 error for this - no duplicate diagnostic.

#### Rust Input

```
use facet::Facet;

#[derive(Facet)]
#[repr(C, transparent)]
struct Wrapper(u32);

fn main() {}
```

#### Compiler Error

```
error[E0692]: transparent struct cannot have other repr hints
 --> src/main.rs:4:8
  |
4 | #[repr(C, transparent)]
  |        ^  ^^^^^^^^^^^
For more information about this error, try `rustc --explain E0692`.
error: could not compile `test` (bin "test") due to 1 previous error
```

### Conflicting repr: transparent and primitive

Using `#[repr(transparent)]` with a primitive type like `u8` is not allowed.

Facet defers to rustc's E0692 error for this - no duplicate diagnostic.

#### Rust Input

```
use facet::Facet;

#[derive(Facet)]
#[repr(transparent, u8)]
enum Status {
    Active,
    Inactive,
}

fn main() {}
```

#### Compiler Error

```
error[E0692]: transparent enum cannot have other repr hints
 --> src/main.rs:4:8
  |
4 | #[repr(transparent, u8)]
  |        ^^^^^^^^^^^  ^^
error[E0731]: transparent enum needs exactly one variant, but has 2
 --> src/main.rs:5:1
  |
5 | enum Status {
  | ^^^^^^^^^^^ needs exactly one variant, but has 2
6 |     Active,
  |     ------ variant here
7 |     Inactive,
  |     -------- too many variants in `Status`
Some errors have detailed explanations: E0692, E0731.
For more information about an error, try `rustc --explain E0692`.
error: could not compile `test` (bin "test") due to 2 previous errors
```

### Multiple primitive types in repr

Specifying multiple primitive types in `#[repr(...)]` is not allowed.

Facet defers to rustc's E0566 error for this - no duplicate diagnostic.

#### Rust Input

```
use facet::Facet;

#[derive(Facet)]
#[repr(u8, u16)]
enum Priority {
    Low,
    Medium,
    High,
}

fn main() {}
```

#### Compiler Error

```
error[E0566]: conflicting representation hints
 --> src/main.rs:4:8
  |
4 | #[repr(u8, u16)]
  |        ^^  ^^^
  |
  = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
  = note: for more information, see issue #68585 <https://github.com/rust-lang/rust/issues/68585>
  = note: `#[deny(conflicting_repr_hints)]` (part of `#[deny(future_incompatible)]`) on by default
warning: enum `Priority` is never used
 --> src/main.rs:5:6
  |
5 | enum Priority {
  |      ^^^^^^^^
  |
  = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default
For more information about this error, try `rustc --explain E0566`.
warning: `test` (bin "test") generated 1 warning
error: could not compile `test` (bin "test") due to 2 previous errors; 1 warning emitted
```

### Unsupported repr (facet-specific)

Using `#[repr(packed)]` is valid Rust, but facet doesn't support it.

This is a facet-specific error with a helpful message.

#### Rust Input

```
use facet::Facet;

#[derive(Facet)]
#[repr(packed)]
struct Data {
    a: u8,
    b: u32,
}

fn main() {}
```

#### Compiler Error

```
error: unsupported repr `packed` - facet only supports C, Rust, transparent, and primitive integer types
 --> src/main.rs:4:8
  |
4 | #[repr(packed)]
  |        ^^^^^^
error: could not compile `test` (bin "test") due to 1 previous error
```

### Multiple #[repr] attributes

Having multiple separate `#[repr(...)]` attributes triggers rustc's E0566.

Facet defers to rustc for this - no duplicate diagnostic.

#### Rust Input

```
use facet::Facet;

#[derive(Facet)]
#[repr(C)]
#[repr(u8)]
enum Status {
    Active,
    Inactive,
}

fn main() {}
```

#### Compiler Error

```
error[E0566]: conflicting representation hints
 --> src/main.rs:4:8
  |
4 | #[repr(C)]
  |        ^
5 | #[repr(u8)]
  |        ^^
  |
  = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
  = note: for more information, see issue #68585 <https://github.com/rust-lang/rust/issues/68585>
  = note: `#[deny(conflicting_repr_hints)]` (part of `#[deny(future_incompatible)]`) on by default
warning: enum `Status` is never used
 --> src/main.rs:6:6
  |
6 | enum Status {
  |      ^^^^^^
  |
  = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default
For more information about this error, try `rustc --explain E0566`.
warning: `test` (bin "test") generated 1 warning
error: could not compile `test` (bin "test") due to 2 previous errors; 1 warning emitted
```

Rename Errors
-------------

### Unknown rename_all rule (facet-specific)

Using an unknown case convention in `rename_all` is a facet-specific error.

Valid options: `camelCase`, `snake_case`, `kebab-case`, `PascalCase`, `SCREAMING_SNAKE_CASE`.

#### Rust Input

```
use facet::Facet;

#[derive(Facet)]
#[facet(rename_all = "SCREAMING_SNAKE")]
struct Config {
    user_name: String,
    max_retries: u32,
}

fn main() {}
```

#### Compiler Error

```
error: unknown #[facet(rename_all = "...")] rule: `SCREAMING_SNAKE`. Valid options: camelCase, snake_case, kebab-case, PascalCase, SCREAMING_SNAKE_CASE, SCREAMING-KEBAB-CASE, lowercase, UPPERCASE
 --> src/main.rs:4:9
  |
4 | #[facet(rename_all = "SCREAMING_SNAKE")]
  |         ^^^^^^^^^^
error: could not compile `test` (bin "test") due to 1 previous error
```
