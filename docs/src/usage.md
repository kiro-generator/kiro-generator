# Usage

## Setup 

1. Create global config file `~/.kiro/generators/kg.toml` 

2. Add your 1st agent: 

```toml
[agents]
default = { inherits = [] }
```

3. Verify

```
kg validate
```

4. Generate

```shell
kg generate
```
