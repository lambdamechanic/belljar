# Work Plan: par-rs (Rust clone of coplane/par) with per-session Docker Compose isolation

## Goals
- Provide a Rust CLI with a similar interface to `coplane/par` for running tasks/steps in parallel.
- Each run/session executes inside an isolated Docker Compose project (unique project name), enabling hermetic services (e.g., Postgres, Redis) per session.
- Clean lifecycle: provision -> run -> collect logs/artifacts -> teardown.
- Solid developer ergonomics: clear logs, good errors, cross-platform (Linux/macOS, Docker Desktop).

## High-Level Architecture
- Workspace layout:
  - `app/` — CLI crate (binary) mirroring `par` UX.
  - `core/` — library crate for configs, orchestration, runners, docker-compose integration.
  - `tests/` — integration/e2e tests spanning the workspace (Rust integration tests; optional Docker-gated tests).
- Session isolation via `docker compose -p <unique_session>` to namespace networks, volumes, and containers.
- Config-driven services using compose templates; CLI flags/env to select stacks (e.g., `--with postgres,redis`).

## Phase 0 — Recon and Spec
1. Review `coplane/par` repository to enumerate:
   - CLI commands, flags, and config files it supports.
   - Core semantics: task graph, parallelism, retries, env injection, artifacts.
   - Typical workflows and examples to mirror.
2. Draft a minimal feature spec and parity table (par -> par-rs), flag deltas.

Deliverables:
- `docs/spec.md` with command/flag mapping and MVP scope.

## Phase 1 — Scaffolding
1. Initialize Rust workspace (`Cargo.toml` with members `app`, `core`).
2. Create crates:
   - `core/` (lib): config types, session manager, compose driver, task runner traits.
   - `app/` (bin): clap-powered CLI, subcommands, config loading, drives `core`.
3. Establish `tests/` for integration tests; add a `docker` feature gate to enable Docker tests.

Deliverables:
- Compiling workspace with `--help` showing CLI skeleton.

## Phase 2 — Config + CLI UX
1. Define config schema (YAML/TOML) close to `par` (tasks, env, matrix, includes).
2. Implement CLI:
   - `par-rs run` (default): executes tasks defined in config.
   - Global flags: `--config`, `--var KEY=VAL`, `--parallel N`, `--retry`, `--with <services>`, `--keep`.
3. Environment & variable resolution order (env, file, CLI) with clear precedence.

Deliverables:
- Parsing, validation errors, and a sample config under `examples/`.

## Phase 3 — Docker Compose Isolation
1. Compose driver:
   - Generate unique session id and compose project name.
   - Render compose file(s) from templates (services selected via CLI/config).
   - Bring up with healthchecks; stream logs on demand.
   - Inject env/ports/volumes per session.
2. Lifecycle API: `provision()`, `exec(task)`, `teardown()`, with `--keep` to skip teardown and `--reuse <session>` for debugging.
3. Cross-platform nuances (Docker Desktop/macOS) and TTY handling.

Deliverables:
- Working isolated services (e.g., Postgres/Redis) per run with `-p <session>`.

## Phase 4 — Task Runner
1. Implement task graph (DAG) with dependencies, concurrency limits, and retries.
2. Execution modes:
   - Host execution (default) and container execution (targeted service) via `docker compose exec`.
   - Log capture and per-task status (success/fail/timeout).
3. Cancellations and graceful shutdown (Ctrl-C) with cleanup.

Deliverables:
- Deterministic execution order with parallelism, exit codes, and summaries.

## Phase 5 — Testing and CI
1. Unit tests for parsing, graph topology, and CLI arg merging.
2. Integration tests (gated with `DOCKER_TESTS=1`) to spin services and run sample tasks.
3. GitHub Actions:
   - Lint + unit tests (no Docker) on PRs.
   - Optional self-hosted or workflow dispatch for Docker tests.

Deliverables:
- Green CI for non-Docker paths; documented local flow for Docker tests.

## Phase 6 — UX Polish & Docs
1. Human-friendly logs (task prefixes, colors, timing) and `--json` for machine parsing.
2. Errors with suggestions; `par-rs doctor` for environment diagnostics.
3. Documentation:
   - Quickstart, config reference, examples with popular stacks.
   - How isolation works; performance tips; cleanup commands.

Deliverables:
- `README.md` and `docs/` with examples and troubleshooting.

## Implementation Notes
- Compose project isolation: always pass `-p parrs_<shortid>` and set `COMPOSE_PROJECT_NAME`.
- Resource limits: expose `--cpus/--memory` per service via compose overrides.
- Security: avoid mounting host docker.sock into containers; long-running services live only within the session project.
- Extensibility: service templates as modular YAML snippets in `assets/compose/`.

## Milestones
M1: Workspace + CLI skeleton + spec
M2: Config parsing + basic run
M3: Per-session compose up/down with Postgres
M4: DAG runner with parallelism + retries
M5: Integration tests + CI
M6: Docs + examples + 0.1 release

## Open Questions / To Validate
- Exact parity surface from `coplane/par` (commands/flags that are must-have for MVP).
- Whether to support remote Docker contexts in MVP.
- Windows support scope.

