# Debugging

Use trace and debug logging to troubleshoot configuration issues.

## Debug Mode

Enable debug output with `-d` or `--debug`:

```bash
kg validate --debug
kg generate --debug
kg diff --debug
```

Debug mode shows:
- Configuration file loading
- Agent resolution
- Basic merge operations

## Trace Mode

Enable detailed trace logging for specific agents with `-t` or `--trace`:

```bash
# Trace specific agent
kg validate --trace rust

```
TRACE main{dry_run=true}:discover{fs= location=[global,local]}:agent{name="rust"}: matching location
TRACE main{dry_run=true}:discover{fs= location=[global,local]}:agent{name="rust"}: found inline global and global file source
TRACE main{dry_run=true}:discover{fs= location=[global,local]}:agent{name="rust"}: merge start self_name=rust self_template=false other_name=rust other_template=false
TRACE main{dry_run=true}:discover{fs= location=[global,local]}:agent{name="rust"}: inherits: extended count=1
TRACE main{dry_run=true}:discover{fs= location=[global,local]}:agent{name="rust"}: subagents: merging
TRACE main{dry_run=true}:write_all{self=global_path=/home/user/.kiro/generators exists=true local_agents=false}:agent{name="rust" local=false}:write{self=global_path=/home/user/.kiro/generators exists=true local_agents=false agent=rust}: {
```

# Trace all agents (very verbose)
kg validate --trace all
```

Trace output shows:
- Configuration file loading
- Inheritance resolution
- Detailed merge operations
- Force property application
- Step-by-step agent construction

## Use Cases

**Debug inheritance issues:**
```bash
kg validate --trace rust
```

**Debug all agents:**
```bash
kg validate --trace all
```

**Debug generation:**
```bash
kg generate --trace rust
```

**Debug diff behavior:**
```bash
kg diff --trace rust
```

## Output

Trace and debug output goes to stderr, so you can still pipe stdout:

```bash
kg validate --debug --format json | jq '.agents'
```
