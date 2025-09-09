# belljar

A Rust CLI inspired by coplane/par for managing development sessions and worktrees, with optional per-session Docker Compose provisioning. Services are defined by the target repository (no prebaked stacks).

## Quickstart
- Prepare your repo (one-time):
  - Add compose files in `.belljar/compose/*.yml` or a `docker-compose.yml` at repo root.
- Build CLI:
  - `cargo build` (workspace) or run via `cargo run -p par-cli -- --help`.
- Start a session:
  - `cargo run -p par-cli -- start my-feature --path .`
  - If compose files are present, belljar runs `docker compose -p <project> up -d`.
- Checkout a branch into a session:
  - `cargo run -p par-cli -- checkout feature-x --path . --label fx`
  - Creates a git worktree at `.belljar/worktrees/fx` and records the session.
- Open and send commands (tmux):
  - `cargo run -p par-cli -- open my-feature`
  - `cargo run -p par-cli -- send my-feature "make test"`
- List and remove sessions:
  - `cargo run -p par-cli -- ls`
  - `cargo run -p par-cli -- rm my-feature` or `rm all`

Notes
- Compose discovery is repo-owned: belljar never ships service templates.
- If tmux is not installed, open prints a fallback path; send will fail gracefully.
- Worktrees are stored under `.belljar/worktrees/` and are ignored by git.
