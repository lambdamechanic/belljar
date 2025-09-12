### Summary

- feat(cli): add `belljar new <label> [--from <base>] [--path <repo>]` to create a new branch from a base (default `main`), set up a worktree under `.belljar/worktrees/<label>`, provision compose, and focus the tmux session.
- core: add `git::ensure_worktree_from` and `tmux::switch_client` helpers.
- docs: README section with tmux popup bindings (Ctrl-b j) for quick session creation/focus.

### Rationale

Fast path to start a focused dev session: a single keystroke (tmux popup) to branch from `main`/`develop`, create a belljar session with isolated compose, and jump right into a tmux session.

### Usage

```bash
belljar new my-feature            # from main
belljar new my-fix --from develop # from a custom base
```

Tmux popup binding (Ctrl-b j): see README’s "Tmux Integration" for a command-prompt and display-popup variant.

### Notes

- If the session already exists, `belljar new` focuses it (switches client inside tmux; attaches outside tmux).
- No Docker dependency for non-compose repos. Compose up/down remains best-effort.

