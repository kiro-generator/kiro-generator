URL Source: https://facet.rs/showcases/assert/
Scraped: 2026-02-19T21:33:10Z

---

Title: Assertions - facet

URL Source: https://facet.rs/showcases/assert/

Markdown Content:
[`facet-assert`](https://docs.rs/facet-assert) provides structural assertions for any `Facet` type without requiring `PartialEq` or `Debug`. Compare values across different types with identical structure, and get precise structural diffs showing exactly which fields differ.

Same Values
-----------

Two values with identical content pass `assert_same!` — no `PartialEq` required.

Target Type
```
#[derive(Facet)]
struct Config {
    host: String,
    port: u16,
    debug: bool,
    tags: Vec<String>,
}
```

#### Success

```
Config {
  host: "localhost",
  port: 8080,
  debug: true,
  tags: Vec<String> ["prod", "api"],
}
```

Cross-Type Comparison
---------------------

Different type names (`Config` vs `ConfigV2`) with the same structure are considered "same". Useful for comparing DTOs across API versions or testing serialization roundtrips.

Target Type
```
#[derive(Facet)]
struct Config {
    host: String,
    port: u16,
    debug: bool,
    tags: Vec<String>,
}
```

#### Success

```
Config {
  host: "localhost",
  port: 8080,
  debug: true,
  tags: Vec<String> ["prod"],
}
```

Nested Structs
--------------

Nested structs are compared recursively, field by field.

Target Type
```
#[derive(Facet)]
struct Person {
    name: String,
    age: u32,
    address: Address,
}
#[derive(Facet)]
struct Address {
    street: String,
    city: String,
}
```

#### Success

```
Person {
  name: "Alice",
  age: 30,
  address: Address {
    street: "123 Main St",
    city: "Springfield",
  },
}
```

Structural Diff
---------------

When values differ, you get a precise structural diff showing exactly which fields changed and at what path — then render it as Rust, JSON, or XML for whichever toolchain you need.

#### Rust Diff Output

```
{
    debug: true → false
    host: "localhost" → "prod.example.com"
    port: 8080 → 443
    tags: [
        .. 1 unchanged item
        - "api"
    ]
}
```

#### JSON Diff Output

```
{ /* @Config */
      ← "debug": true , "host": "localhost"       , "port": 8080
      → "debug": false, "host": "prod.example.com", "port": 443
    
        "tags": [
            "prod"
            - "api"
        ],
    }
```

#### XML Diff Output

```
<@Config
      ← debug="true"  host="localhost"        port="8080"
      → debug="false" host="prod.example.com" port="443"
    >
        
            prod
            - api
        
    </@Config>
```

Vector Differences
------------------

Vector comparisons show exactly which indices differ, which elements were added, and which were removed.

#### Diff Output

```
[
    .. 2 unchanged items
    3 → 99
    .. 1 unchanged item
    - 5
]
```
