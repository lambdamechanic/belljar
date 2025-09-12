# belljar

A Rust CLI inspired by coplane/par for managing development sessions and worktrees, with optional per-session Docker Compose provisioning. Services are defined by the target repository (no prebaked stacks).

## Quickstart
- Prepare your repo (one-time):
  - Add compose files in `.belljar/compose/*.yml` or a `docker-compose.yml` at repo root.
- Build CLI:
  - `cargo build` (workspace) or run via `cargo run -p belljar -- --help`.
- Start a session:
  - `cargo run -p belljar -- start my-feature --path .`
  - If compose files are present, belljar runs `docker compose -p <project> up -d`.
- Checkout a branch into a session:
  - `cargo run -p belljar -- checkout feature-x --path . --label fx`
  - Creates a git worktree at `.belljar/worktrees/fx` and records the session.
- Tests (unit + integration): `cargo test`.
- Property tests: proptest is preferred for pure logic. Default cases are modest. Increase cases for a smoke run: `PROPTEST_CASES=1000 cargo test`.
- Open and send commands (tmux):
  - `cargo run -p belljar -- open my-feature`
  - `cargo run -p belljar -- send my-feature "make test"`
- Workspaces (multi-repo context):
  - List: `cargo run -p belljar -- workspace ls`
  - Create: `cargo run -p belljar -- workspace start dev-ws --path . --repos frontend,backend --open`
  - Open: `cargo run -p belljar -- workspace open dev-ws`
  - Remove: `cargo run -p belljar -- workspace rm dev-ws`
- List and remove sessions:
  - `cargo run -p belljar -- ls`
  - `cargo run -p belljar -- rm my-feature` or `rm all`

Notes
- Compose discovery is repo-owned: belljar never ships service templates.
- If tmux is not installed, open prints a fallback path; send will fail gracefully.
- Worktrees are stored under `.belljar/worktrees/` and are ignored by git.
- Workspaces are recorded in the registry and open a dedicated tmux session (named `ws-<label>`).

## Tmux Integration (Popup + Quick Focus)

Bind a popup to create/focus a belljar session and jump to it with `Ctrl-b j`.

- Minimal prompt (uses tmux's command-prompt):

```tmux
# ~/.tmux.conf
bind-key j command-prompt -p "New belljar session:" \
  "run-shell 'belljar new %1 --path #{pane_current_path}'; switch-client -t %1"
```

- Popup window with inline prompt and log output:

```tmux
# ~/.tmux.conf
bind-key j display-popup -E "sh -lc 'read -p \"New belljar session (base: main): \" name; \\
  [ -z \"$name\" ] || belljar new \"$name\" --path #{pane_current_path} 2>&1 | sed -u \"s/^/[belljar] /\"; \\
  [ -z \"$name\" ] || tmux switch-client -t \"$name\"'"
```

Notes
- Inside tmux, `belljar new` switches the client to the session; outside tmux it attaches.
- Change the base branch with `--from <branch>` if you don't want `main`.
- Reload config: `tmux source-file ~/.tmux.conf`.
