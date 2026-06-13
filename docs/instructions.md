# Project Blueprint: **Farfetch**

An open-source, blistering fast, keyboard-driven **Terminal User Interface (TUI)** API client designed specifically to sit natively inside **Zed's GPU-accelerated terminal panels**. It combines the weightless performance of a Rust-native binary with modern developer features—shifting focus from *heavy browser views* to a *zero-configuration, git-aware workspace*.

---

## 🚀 Core Value Proposition

> **"Zero Config, Git Smart, Clipboard Native."**
> Unlike Postman (heavy Electron app, forced cloud sync) or Slumber (strict, manual YAML configuration before you can test anything), Farfetch lets you query ad-hoc endpoints instantly, maps your target environment directly to your local Git branch, and treats system copy-pasting as a first-class citizen.

---

## 🛠️ The Feature Set

### 1. The Dynamic Interface Modes

- **Ad-Hoc Scratchpad:** Launch into a completely blank split pane. Type or paste a raw URL/method, modify headers inline, and hit `Ctrl + Enter` to fire a request instantly without adding it to a collection.
- **Persistent Collections:** Hit `Ctrl + S` from the scratchpad to cleanly append any temporary request into an open-source, human-readable `.json` or `.yaml` file tracked right inside your project repository.

### 2. Context-Aware Environment Management

- **Git-Branch Environment Mapping:** Define an option inside your project configuration that automatically links environments to your current local Git branches. Switching from your `feature/login` branch to your `main` or `release/uat` branch automatically toggles your targeted backend hosts and API keys without a single click.
- **Explicit Override Toggle:** A top bar dashboard allowing rapid `Tab` or mouse-click selection between variables (`Local`, `Dev`, `UAT`, `Prod`) when manual execution is necessary.

### 3. Clipboard & Editor Ingestion Engine

- **Smart cURL Parser:** Pasting a standard `curl -X POST ...` string directly into the query bar immediately disintegrates the payload, dynamically parsing headers, methods, query parameters, and JSON blocks directly into their respective TUI layout fields.
- **External Editor Hand-off:** Focus on a large JSON payload structure inside the TUI and press `E`. Farfetch spins up a temporary target file directly within a native Zed editor buffer tab. Utilize all of Zed's powerful multi-cursors, formatting tools, and snippet expansions; saving and closing the tab instantly updates the input payload state inside the TUI pane.
- **System Clipboard Sync:** Native binding to the system clipboard context. Tapping `Y` (Yank) on the response view instantly pushes formatted, indented JSON to the operating system's global clipboard ring.

---

## 🏗️ Architectural Component Layout

```
| [Local] -> (Git: feature/auth)                             [Help: ?]    |  <- Header State Area
|-------------------------------------------------------------------------|
| METHODS & ENDPOINT                                                       |
| [POST] http://localhost:8080/api/v1/auth/login                           |  <- Query Inputs Panel
|-------------------------------------------------------------------------|
| HEADERS & PARAMETERS      | REQUEST BODY (JSON)                          |
| Content-Type : app/json   | {                                            |  <- Interactive Forms
| Authorization: Bearer ... |   "username": "developer",                   |     (Tab to cycle)
|                           |   "password": "•••••••••"                    |
|                           | }                                            |
|-------------------------------------------------------------------------|
| RESPONSE METRICS: [200 OK] | [Time: 42ms] | [Size: 1.2 KB]               |
|-------------------------------------------------------------------------|
| RESPONSE BODY                                                            |
| {                                                                        |  <- Pretty JSON Output
|   "status": "authenticated",                                             |     (Scrollable with Vim /
|   "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."                    |      Arrow Navigation)
| }                                                                        |
|__________________________________________________________________________|
```

---

## 🧰 Technology & Tooling Stack

Farfetch is designed to run close to the metal with no bloated runtime engines, maintaining a memory envelope of less than **15 MB RAM**.

| Dependency Layer | Open-Source Choice | Role / Responsibility |
|---|---|---|
| **Core UI Engine** | `ratatui` (Rust) | High-performance, immediate-mode terminal UI frame buffer rendering. |
| **Terminal Backend** | `crossterm` | Cross-platform terminal raw mode manipulation, event parsing (keyboards/mouse input). |
| **Async Core & Networking** | `tokio` + `reqwest` | Asynchronous I/O polling with a lightning-fast HTTP networking client wrapper. |
| **System Clipboard Link** | `copypasta` | Safe, cross-platform programmatic gateway straight to OS window managers (Wayland, X11, macOS AppKit, Windows API). |
| **Fuzzy History Lookup** | `nucleo` or `fuzzy-matcher` | High-efficiency matching algorithms driving instant historical endpoint lookup pipelines (`Ctrl + R`). |
| **Syntax Decoration** | `syntect` | Local engine driving lightning-fast colorization profiles for incoming text (JSON/XML/HTML response view). |

---

## 📋 File Layout Strategy

All configuration remains entirely local, declarative, and production-safe.

### Local Workspace Structure

```
.farfetch/
├── config.json         <- Project global defaults and Git branch environmental maps
├── environments.json   <- Key-Value tokens safely segregated (Add to your local .gitignore)
└── collections.json    <- The shared endpoints and saved requests folder (Commit to Git)
```

#### Example Environment Map Configuration (`config.json`)

```json
{
  "git_branch_mapping": {
    "main": "production",
    "release/*": "uat",
    "feature/*": "local",
    "bugfix/*": "dev"
  },
  "default_editor": "zed"
}
```

---

## 🛣️ MVP Execution Milestones

### Phase 1: Core Networking & TUI Layout (The Scaffold)

- [ ] Configure a basic split ratatui viewport grid displaying Methods/URL, Parameters, Request Body, and Response Text.
- [ ] Wire keyboard navigation hooks (Tab cycles panes, Arrow/Vim keys navigate internal contents).
- [ ] Hook asynchronous HTTP execution handler loops leveraging reqwest via background tokio processes to keep the core main thread completely responsive during flight.

### Phase 2: System Interoperability (The Polish)

- [ ] Implement the copypasta bridge providing native text copying and automated payload parsing routines.
- [ ] Build the file creation logic managing the temporary external document swap to support the external editor command flow (E).
- [ ] Add regex parsing mechanisms processing string injections to identify incoming raw cURL block arguments.

### Phase 3: The Automation Engine (The Edge over Competitors)

- [ ] Embed native Git CLI sub-shell listeners parsing `.git/HEAD` changes to drive automatic background state mutation routines.
- [ ] Package the repository binary and create the simple configuration template manifest to register farfetch into the main Zed Extensions Marketplace Registry.
