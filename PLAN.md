# Work Plan: par-rs (Rust clone of coplane/par) with per-session Docker Compose isolation

## Goals
- Provide a Rust CLI with a similar interface to `coplane/par` for managing development sessions/worktrees and multi-repo workspaces.
- Each session/workspace runs in an isolated Docker Compose project (unique project name), enabling hermetic services (e.g., Postgres, Redis) per session.
- Clean lifecycle: provision -> run -> collect logs/artifacts -> teardown.
- Solid developer ergonomics: clear logs, good errors, cross-platform (Linux/macOS, Docker Desktop).

## High-Level Architecture
- Workspace layout:
  - `app/` — CLI crate (binary) mirroring `par` UX (start, checkout, ls, open, rm, send, control-center, workspace subcommands).
  - `core/` — library crate for repo/worktree management, tmux control, and docker-compose integration.
  - `tests/` — integration/e2e tests spanning the workspace (Rust integration tests; optional Docker-gated tests).
- Session isolation via `docker compose -p <unique_session>` to namespace networks, volumes, and containers.
- Config-driven services using compose templates; CLI flags/env to select stacks (e.g., `--with postgres,redis`).

## Phase 0 — Recon and Spec
1. Review `coplane/par` repository to enumerate:
   - CLI commands, flags, and config it supports (sessions/workspaces).
   - Core semantics: session labels, global registry, worktree/branch ergonomics, tmux integration.
   - Typical workflows and examples to mirror.
2. Draft a minimal feature spec and parity table (par -> par-rs), flag deltas.

Deliverables:
- `docs/spec.md` with command/flag mapping and MVP scope.

## Phase 1 — Scaffolding
1. Initialize Rust workspace (`Cargo.toml` with members `app`, `core`).
2. Create crates:
   - `core/` (lib): repo/worktree ops, session registry, tmux + compose drivers.
   - `app/` (bin): clap-powered CLI, subcommands matching `par`, drives `core`.
3. Establish `tests/` for integration tests; add a `docker` feature gate to enable Docker tests.

Deliverables:
- Compiling workspace with `--help` showing CLI skeleton.

## Phase 2 — CLI UX + Core APIs
1. Implement CLI surface with subcommands:
   - `start`, `checkout`, `ls`, `open`, `rm`, `send`, `control-center`, `workspace <...>`.
2. Define session model: label, repo path, branch, worktree path, compose project, selected services.
3. Establish a registry (on-disk DB under `~/.local/share/par-rs`) to track sessions/workspaces.

Deliverables:
- Parsing, validation errors, and a sample config under `examples/`.

## Phase 3 — Docker Compose Isolation
1. Compose driver:
   - Generate unique session id and compose project name.
   - Render compose file(s) from templates (services selected via CLI/config).
   - Bring up with healthchecks; stream logs on demand.
   - Inject env/ports/volumes per session.
2. Lifecycle API: `provision()`, `exec(cmd)`, `teardown()`, with `--keep` to skip teardown and `--reuse <session>` for debugging.
3. Cross-platform nuances (Docker Desktop/macOS) and TTY handling.

Deliverables:
- Working isolated services (e.g., Postgres/Redis) per run with `-p <session>`.

## Phase 4 — Repo, Worktree, and Tmux
1. Implement `start` to create a worktree and branch, initialize session registry, and launch tmux session.
2. Implement `checkout` to attach to existing branches/PRs per par behavior.
3. Implement `open`, `send`, and `control-center` with tmux control and helpful UX.

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
M4: Repo/worktree + tmux integration
M5: Integration tests + CI
M6: Docs + examples + 0.1 release

## Open Questions / To Validate
- Exact parity surface from `coplane/par` (commands/flags that are must-have for MVP).
- Whether to support remote Docker contexts in MVP.
- Windows support scope.
