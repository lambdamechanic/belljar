# Work Plan: belljar (Rust clone of coplane/par) with per-session Docker Compose isolation

## Goals
- Provide a Rust CLI (belljar) with a similar interface to `coplane/par` for managing development sessions/worktrees and multi-repo workspaces.
- Each session/workspace runs in an isolated Docker Compose project (unique project name), enabling hermetic services (e.g., Postgres, Redis) per session.
- Clean lifecycle: provision -> run -> collect logs/artifacts -> teardown.
- Solid developer ergonomics: clear logs, good errors, cross-platform (Linux/macOS, Docker Desktop).

## High-Level Architecture
- Workspace layout:
  - `app/` — CLI crate (binary) mirroring `par` UX (start, checkout, ls, open, rm, send, control-center, workspace subcommands).
  - `core/` — library crate for repo/worktree management, tmux control, and docker-compose integration.
  - `tests/` — integration/e2e tests spanning the workspace (Rust integration tests; optional Docker-gated tests).
- Session isolation via `docker compose -p <unique_session>` to namespace networks, volumes, and containers.
- Config-driven services come from the target repository's compose files. belljar discovers `.belljar/compose/*.yml` or `docker-compose.yml|yaml`/`compose.yml|yaml` in the repo; no built-in templates. A lightweight `wizard` subcommand can scaffold starter Dockerfiles into the repo (users own/edit them).

## Phase 0 — Recon and Spec — [DONE]
1. Review `coplane/par` repository to enumerate:
   - CLI commands, flags, and config it supports (sessions/workspaces).
   - Core semantics: session labels, global registry, worktree/branch ergonomics, tmux integration.
   - Typical workflows and examples to mirror.
2. Draft a minimal feature spec and parity table (par -> par-rs), flag deltas.

Deliverables:
- `docs/spec.md` with command/flag mapping and MVP scope. [DONE]

## Phase 1 — Scaffolding — [DONE]
1. Initialize Rust workspace (`Cargo.toml` with members `app`, `core`).
2. Create crates:
   - `core/` (lib): repo/worktree ops, session registry, tmux + compose drivers.
   - `app/` (bin): clap-powered CLI, subcommands matching `par`, drives `core`.
3. Establish `tests/` for integration tests; add a `docker` feature gate to enable Docker tests.

Deliverables:
- Compiling workspace with `--help` showing CLI skeleton. [DONE]

## Phase 2 — CLI UX + Core APIs — [DONE]
1. Implement CLI surface with subcommands:
   - `start`, `checkout`, `ls`, `open`, `rm`, `send`, `control-center`, `workspace <...>`.
2. Define session model: label, repo path, branch, worktree path, compose project, selected services.
3. Establish a registry (on-disk DB under data dir) to track sessions/workspaces. [DONE]

Deliverables:
- CLI subcommands scaffolded; session model + registry implemented. [DONE]

## Phase 3 — Docker Compose Isolation — [DONE]
1. Compose driver:
   - Generate unique session id and compose project name. [DONE]
   - Discover repo-provided compose files (`.belljar/compose/*.yml` or `docker-compose*.yml`). [DONE]
   - Bring up/down with `docker compose -p <project>`; log errors. [DONE]
2. Lifecycle API: `provision()`, `exec(cmd)`, `teardown()` (up/down implemented; exec TBD).
3. Cross-platform nuances (Docker Desktop/macOS) and TTY handling.

Deliverables:
- Working isolated services (e.g., Postgres/Redis) per run with `-p <session>`.

## Phase 4 — Repo, Worktree, and Tmux — [DONE]
1. Implement `start` to create a worktree and branch, initialize session registry, and launch tmux session.
   - tmux `open` and `send` implemented; worktree/branch creation implemented.
2. Implement `checkout` to attach to existing branches (branch name only for MVP). [DONE]
3. Implement `control-center` with tmux panes/windows. [DONE — basic tmux session with per-session windows]

## Phase 5 — Testing and CI — [IN PROGRESS]
1. Tests added for registry, compose discovery/up/down, tmux helpers, git worktree, and CLI flows. [DONE]
2. Coverage: cargo-llvm-cov script and README docs; property tests preferred. [DONE]
3. GitHub Actions:
   - Lint + unit tests (no Docker) on PRs. [TODO]
   - Coverage report job (text + lcov). [TODO]
   - Optional self-hosted or workflow dispatch for Docker tests. [PENDING]

Deliverables:
- Green CI for non-Docker paths; documented local flow for Docker tests.

## Phase 6 — UX Polish & Docs — [IN PROGRESS]
1. Human-friendly logs (task prefixes, colors, timing) and `--json` for machine parsing. [PENDING]
2. Errors with suggestions; `belljar doctor` for environment diagnostics. [PENDING]
3. Documentation:
   - Quickstart, config reference, examples with popular stacks. [IN PROGRESS]
   - How isolation works; performance tips; cleanup commands. [PENDING]
4. Interactive scaffolding: `belljar wizard` prompts for language (Rust/Python) and AI coder (Codex/Claude/Goose/Aider), generating `Dockerfile.dev` and `Dockerfile.ai` (which now references the dev Dockerfile via BuildKit) with safe overwrite prompts. [DONE]

Deliverables:
- `README.md` and `docs/` with examples and troubleshooting.

## Implementation Notes
- Compose project isolation: always pass `-p parrs_<shortid>` and set `COMPOSE_PROJECT_NAME`.
- Resource limits: expose `--cpus/--memory` per service via compose overrides.
- Security: avoid mounting host docker.sock into containers; long-running services live only within the session project.
- Extensibility: service templates as modular YAML snippets in `assets/compose/`.

## Milestones
M1: Workspace + CLI skeleton + spec — DONE
M2: Registry + compose discovery + up/down — DONE
M3: tmux open/send for sessions — DONE
M4: Repo worktree/branch creation + checkout — DONE
M5: Tests + CI (coverage) — IN PROGRESS
M6: Docs + examples + 0.1 release — IN PROGRESS
    - Includes the `wizard` scaffolding command for quickstarts. [DONE]
M7: Dogfood belljar to develop itself — PENDING

## Open Questions / To Validate
- Exact parity surface from `coplane/par` (commands/flags that are must-have for MVP).
- Whether to support remote Docker contexts in MVP.
- Windows support scope.
