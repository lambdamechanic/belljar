### Summary

- Finalize `par` -> `belljar` rename across core, CLI, and docs.
- Core: change data dir vendor/app to `belljar` and compose project prefix to `belljar_<shortid>`.
- CLI: version output now prints `belljar <cli> (core <ver>)`.
- Docs: update spec and plan to reflect new compose project prefix and registry path.

### Rationale

Maintains consistent naming and user-facing identifiers after the refactor. Using `belljar_<shortid>` for compose project names avoids collisions and aligns with the tool’s name. Data directory under `~/.local/share/belljar` is clearer and decoupled from historical `par-rs`.

### Notes

- All tests pass locally: `cargo test` (no Docker required for these tests).
- Lint and format are clean: `cargo fmt --all` and `cargo clippy --all-targets --all-features -- -D warnings`.
- No behavioral changes aside from identifiers and version string.

### PLAN

- Phase 5 — Tests & CI: keep CI wiring PENDING.
- Phase 6 — Docs Polish: adjusted spec/plan strings accordingly.

