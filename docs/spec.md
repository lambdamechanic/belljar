# par-rs Spec and Parity Map

## Scope and Goals
- Rust clone of coplane/par focused on managing development sessions and workspaces.
- Addition: each session/workspace provisions an isolated Docker Compose project for hermetic services.

## CLI Surface (Parity with par)
- `par start <label> [--path <repo>] [--branch <name>] [--with <svc,svc>] [--keep]`
- `par checkout <target> [--path <repo>] [--label <label>]`
- `par ls`
- `par open <label>`
- `par rm <label|all>`
- `par send <label|all> <command...>`
- `par control-center`
- `par workspace <subcmd>` where subcmds include:
  - `start <label> [--path <root>] [--repos r1,r2] [--open]`
  - `code <label>` | `open <label>` | `rm <label>` | `ls`

Notes
- We will initially implement sessions: `start`, `ls`, `open`, `rm` and `send` with minimal functionality, then add `checkout`, `control-center`, and `workspace`.
- Compose selection via `--with postgres,redis` to include services; default stack is minimal.

## Session Model
- label: globally unique string.
- repo_path: absolute path to git repository.
- branch: branch/PR info.
- worktree_path: path to created worktree.
- compose_project: `parrs_<shortid>`; stored to allow cleanup.
- services: list of enabled services.
- tmux_session: tmux session name (derived from label).

## Storage
- Registry at `~/.local/share/par-rs/registry.json` (or platform-appropriate dir) tracks sessions and workspaces.

## Compose Isolation
- Project-scoped: `docker compose -p <project> -f <base.yml> [-f overrides...] up -d`.
- Templates live under `assets/compose/`. Selected via `--with`, rendered via environment variables.
- `par send` may run inside a service container via `docker compose exec <svc> <cmd>` (future).

## MVP
- Commands: `start`, `ls`, `open`, `rm`, `send` (host exec), `version`.
- Registry + compose up/down + tmux session creation.
- Basic Postgres service template.

## Out of Scope (initially)
- Windows support.
- Complex PR checkout flows.
- Multi-repo workspace orchestration.
