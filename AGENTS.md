# AGENTS.md

On startup, automatically read and follow these files (in this order):
1. `./.kiro/steering/product.md`
1. `./.kiro/steering/structure.md`
1. `./.kiro/steering/tech.md`
1. `./.kiro/steering/diff.md`


## Performance

Runtime performance is not a primary goal for this project.
Prioritize in this order: correctness, clarity, maintainability, deterministic output, then speed.

Rules:
- Prefer simple, readable code even if it is slower.
- Do not add micro-optimizations without measured evidence (benchmark/profile or user-reported bottleneck).
- Deterministic output is required; use stable ordering (e.g. BTreeMap/BTreeSet) when relevant.
- Avoid obvious pathological complexity regressions when a similarly simple approach exists.
