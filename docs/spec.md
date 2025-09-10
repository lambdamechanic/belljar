# belljar Spec and Parity Map

## Scope and Goals
- Rust clone of coplane/par focused on managing development sessions and workspaces.
- Addition: each session/workspace provisions an isolated Docker Compose project for hermetic services.

## CLI Surface (Parity with par; use `belljar` in place of `par`)
- `belljar start <label> [--path <repo>] [--branch <name>] [--with <svc,svc>] [--keep]`
- `belljar checkout <target> [--path <repo>] [--label <label>]`
- `belljar ls`
- `belljar open <label>`
- `belljar rm <label|all>`
- `belljar send <label|all> <command...>`
- `belljar control-center`
- `belljar workspace <subcmd>`: workspace (multi-repo) management
  - `ls` — list workspaces
  - `start <label> [--path <root>] [--repos r1,r2] [--open]`
  - `open <label>` — ensure/attach tmux session
  - `rm <label>` — remove workspace by label or id

Notes
- We will initially implement sessions: `start`, `ls`, `open`, `rm` and `send` with minimal functionality, then add `checkout`, `control-center`, and `workspace`.
- Compose files are defined in the target repository, not built into belljar. Discovery order:
  1) `.belljar/compose/*.yml|yaml` in the repo
  2) `docker-compose.yml|yaml` or `compose.yml|yaml` in the repo root
  belljar will `docker compose -p <project> -f <...> up -d` when present.

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
- Project-scoped: `docker compose -p <project> [-f files...] up -d`. Files come from the repo as described above.
- No built-in service templates. Users check in Dockerfiles/compose snippets in their repo.
- `belljar send` may run inside a service container via `docker compose exec <svc> <cmd>` (future).

## MVP
- Commands: `start`, `ls`, `open`, `rm`, `send` (host exec), `version`.
- Registry + compose up/down + tmux session creation.
- Compose discovery from repo; no built-in service templates.

## Out of Scope (initially)
- Windows support.
- Complex PR checkout flows.
- Multi-repo workspace orchestration.
## Workspace Model
- id: UUID4
- label: unique string
- root_path: absolute path to workspace root
- repos: absolute paths to member repos (optional list)
- tmux_session: name used for tmux (e.g., `ws-<label>`)
- created_at: RFC3339 timestamp
