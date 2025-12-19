# Usage

## Setup 

1. Create global config file `~/.kiro/generators/kg.toml` 

2. Add your first agent: 

```toml
[agents]
default = { inherits = [] } # `default` is the name of the agent
```

3. Verify

```
kg validate
```

4. Generate

```shell
kg generate
```


## Inheritance

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

## Migrate 

TBD
