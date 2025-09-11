# Repository Guidelines

## Project Structure & Module Organization
- `app/` — CLI binary crate (belljar, par-like UX).
- `core/` — library crate (config, session, compose, runner).
- `tests/` — integration tests spanning the workspace.
- `docs/` — specs and usage docs (e.g., `docs/spec.md`).
- `assets/compose/` — docker-compose templates and snippets.
- `examples/` — sample configs and stacks.

## Build, Test, and Development Commands
- Build: `cargo build` (workspace) or `cargo build -p belljar`.
- Run CLI help: `cargo run -p belljar -- --help`.
- Start a session: `cargo run -p belljar -- start demo --path .` (belljar will provision compose if it finds `.belljar/compose/*.yml` or `docker-compose.yml|yaml`).
- Tests (unit + integration): `cargo test`.
- Docker-gated tests: `DOCKER_TESTS=1 cargo test` (requires Docker + Compose v2).
- Lint: `cargo clippy --all-targets --all-features -- -D warnings`.
- Format: `cargo fmt --all`.

## Coding Style & Naming Conventions
- Rust style via `rustfmt` (4-space indent, stable toolchain).
- Naming: `snake_case` for functions/modules, `CamelCase` for types/traits, `SCREAMING_SNAKE_CASE` for consts.
- Errors: prefer `anyhow` at CLI boundaries; `thiserror` for library error types.
- Keep functions focused; avoid one-letter identifiers; document public APIs with rustdoc.

## Testing Guidelines
- Unit tests inline with modules under `#[cfg(test)]`.
- Integration tests in `tests/` using black-box interfaces.
- Name tests descriptively (e.g., `test_runs_parallel_graph`), keep fast by default; gate slow/docker tests behind `DOCKER_TESTS=1`.
- Ensure core logic has meaningful coverage; test error paths and cancellation.
- Coverage: target and maintain 100%. Check with `cargo llvm-cov --workspace --text` (HTML: `--html`). If needed: `cargo install cargo-llvm-cov && rustup component add llvm-tools-preview`.
- Prefer property-based tests with `proptest` for pure logic (parsers, registry ops, path handling). Use example-based unit tests when properties are unclear or side-effect heavy.

## Commit & Pull Request Guidelines
- Use Conventional Commits: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `build:`, `ci:`, `chore:`.
- PRs: include summary, rationale, linked issues, and usage notes; attach logs or screenshots when UX changes.
- Keep changes scoped; update docs/examples when interfaces or flags change.
- Before opening a PR: ensure formatting and linting are clean.
  - Format: `cargo fmt --all` (no diffs).
  - Lint: `cargo clippy --all-targets --all-features -- -D warnings` (zero warnings).
  - Annotate PLAN stage in PR title or body (e.g., `Phase 6 — Docs polish`) and list PLAN items addressed.

## Plan-Driven Workflow
- Always work from `PLAN.md` as the single source of truth.
- Update statuses (DONE/IN PROGRESS/PENDING) as you progress; adjust phases/milestones when scope changes.
- Commit plan edits alongside feature changes affecting the plan, and reference them in PR descriptions.
- Prefer small, iterative plan updates over large rewrites to keep contributors aligned.

## Security & Configuration Tips
- Requires Docker and Docker Compose v2. Each session uses a unique compose project; avoid mounting host `docker.sock` into containers.
- Do not commit secrets; use env files or `direnv`. Clean up sessions unless `--keep` is intentional.
