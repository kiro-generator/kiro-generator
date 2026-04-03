# Docs TODO

This file is a restart point for the mdBook/docs cleanup work.

## Good Stopping Point

The docs now have:

- a rewritten landing page
- a more realistic install page
- a short `quickstart.md` built around `kg init --skeleton`
- a new `guided-migration.md` page for the `kg-helper` flow
- `config/manifests.md` and `config/agents.md` collapsed into `config/defining-agents.md`
- `kg-resources` renamed to `project-resources` in the local project config and related docs

## Highest-Value Next Step

Add a short **"Why This Matters"** section somewhere prominent.

Best example:

```toml
[agents.rust.skills.kg-helper]
disabled = true
```

Why this matters:

- `kg-helper` exists globally
- this project does not want that skill injected into the local `rust` agent context
- one small local override changes behavior without forking the whole agent
- this is a clean example of local composition beating copy-paste JSON drift

This should probably become a short highlighted callout in one of:

- `docs/src/index.md`
- `docs/src/config/defining-agents.md`
- `docs/src/advanced/knowledge-bases.md`

## Other Good Examples To Emphasize

### `project-resources`

The local template pattern is strong:

- global `rust` agent stays intact
- local `project-resources` adds project-specific docs/knowledge
- local `rust` manifest inherits from `project-resources`
- `kg diff` showed no behavioral drift after renaming the template

This is a great proof point for both individuals and teams.

### `kg tree details rust`

This command is worth showing more often because it proves how `kg` actually resolves config.

In this repo, `rust` is assembled from:

- a global agent file
- a global manifest
- a local manifest

That is a better real-world example than toy "default/rust" docs alone.

### `kg diff`

The `project-resources` rename was a good story:

- source naming improved
- final generated agent behavior did not change
- `kg diff` reported `No changes (1 agents checked)`

That is an excellent confidence/safety message for docs and promotion.

## Likely Next Docs To Review

- `docs/src/inheritance.md`
- `docs/src/templates.md`
- `docs/src/advanced/knowledge-bases.md`
- `docs/src/reference/cli.md`

## Deliberately Deferred

- insecure `install.sh` / curl-pipe story
- `~/.local/share/kg` standalone install flow in user-facing docs
- anything that depends on choosing a final publish/distribution system

## Notes

- Keep examples realistic and based on the actual project where possible
- Prefer short docs with proof over broad docs with filler
- Use `kg tree details`, `kg validate`, and `kg diff` as evidence, not just explanation
