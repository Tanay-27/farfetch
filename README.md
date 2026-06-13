# farfetch

An open-source, git-aware TUI API client built in Rust. No bloat, no cloud sync — just instant ad-hoc requests, smart cURL parsing, and environment switching tied directly to your active Git branch.

> **A lightning-fast, zero-config REST client in under 15 MB of RAM? Sounds *farfetch'd*.**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

`farfetch` is a keyboard-driven **Terminal User Interface (TUI)** API client built in Rust. It runs natively inside **Zed's terminal panels** and any standalone terminal emulator.

Say goodbye to heavy Electron wrappers, forced cloud sync, and massive memory footprints.

---

## Features

- **Zero-config scratchpad** — launch, paste a URL, and fire instantly. `Ctrl+S` saves the request to a persistent collection when you're ready.
- **Git-branch environment syncing** — link API environments to local Git branches. Switching from `feature/*` to `main` automatically swaps host URLs and tokens. No accidental prod requests.
- **Smart cURL ingestion** — paste a raw `curl` string from DevTools or Slack and the parser populates every field automatically (method, headers, body).
- **External editor hand-off** — press `E` on the body pane to open a temp buffer in your editor. Save and close; the TUI updates instantly.
- **Microscopic footprint** — native binary, no runtime overhead, under 15 MB RAM at 60 FPS idle.

---

## Interface

```
┌──────────────────────────────────────────────────────────┐
│ [local] → (Git: feature/auth)              [?] Help      │
├──────────────────────────────────────────────────────────┤
│ [POST] http://localhost:8080/api/v1/auth/login            │
├─────────────────────────────┬────────────────────────────┤
│ Headers                     │ Body (JSON)                 │
│   Content-Type: app/json    │ {                           │
│   Authorization: Bearer ... │   "username": "developer", │
│                             │   "password": "•••••••••"  │
│                             │ }                           │
├──────────────────────────────────────────────────────────┤
│ 200 OK  ·  42ms  ·  1.2 KB                               │
├──────────────────────────────────────────────────────────┤
│ {                                                         │
│   "status": "authenticated",                              │
│   "token": "eyJhbGci..."                                  │
│ }                                                         │
└──────────────────────────────────────────────────────────┘
```

---

## Keybindings

| Key | Action |
|---|---|
| `Tab` / `Shift+Tab` | Cycle focused pane |
| `Enter` | Enter editing mode |
| `Esc` | Exit editing mode |
| `Ctrl+Enter` | Send request |
| `j` / `k` | Scroll response / navigate headers |
| `←` / `→` | Cycle HTTP method (when URL pane focused) |
| `Y` | Yank response body to clipboard |
| `E` | Open body in external editor |
| `Ctrl+S` | Save request to collection |
| `Ctrl+R` | Fuzzy-search request history |
| `?` | Toggle help overlay |
| `q` | Quit |

---

## Workspace layout

Configuration is entirely local and human-readable:

```
.farfetch/
├── config.json         # Workspace defaults and Git branch → environment maps
├── environments.json   # API keys and secrets (add to .gitignore)
└── collections.json    # Saved request collections (safe to commit)
```

Example `config.json`:

```json
{
  "git_branch_mapping": {
    "main": "production",
    "release/*": "uat",
    "feature/*": "local",
    "bugfix/*": "dev"
  },
  "default_editor": "zed",
  "danger_accept_invalid_certs": false
}
```

---

## Getting started

**Prerequisites:** Rust toolchain via [rustup.rs](https://rustup.rs).

```bash
git clone https://github.com/yourusername/farfetch.git
cd farfetch
cargo run
```

For live reloading while working on the UI:

```bash
cargo install cargo-watch
cargo watch -x run
```

---

## Contributing

Browse open issues for `good first issue` or `help wanted` tags. Open a discussion before large feature work so design can be aligned first.

```bash
cargo fmt --check
cargo clippy
cargo test
```

Fork → branch (`feature/my-thing`) → PR against `main`.

---

## License

MIT — see [LICENSE](LICENSE).
