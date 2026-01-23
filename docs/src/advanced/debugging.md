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
