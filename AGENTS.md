# AGENTS.md — AI Agent Guidelines

This file is the single source of truth for any AI agent working in this repository.
Read it in full before making any change.

---

## Project overview

A Minecraft modpack launcher built with **Tauri 2** (Rust backend) and **React + TypeScript** (frontend).

```
launcher/                   ← frontend (React, TypeScript, Vite, pnpm)
launcher/src-tauri/         ← backend (Rust, Tauri 2)
.github/workflows/ci.yml    ← CI pipeline
Taskfile.yml                ← developer task runner
AGENTS.md                   ← this file
```

---

## Architecture

The backend follows a clean-architecture layering. Dependency direction is strictly inward:

```
core/          ← pure domain logic, no I/O, no framework
  manifest/    — BuildManifest, FileEntry, VersionInfo
  state/       — LocalState, InstalledFile, UpdateStatus
  updater/     — UpdatePlan, UpdateService, ManifestComparator, ActionReport
  file_policy/ — FilePolicy, Decision, FileCategory
  settings.rs  — LauncherSettings

infrastructure/   ← implements I/O; depends on core, not on application
  fs/           — FsManager, PathResolver, FileScanner, SafeDelete, hasher
  network/      — HttpClient, FileDownloader, DownloadQueue, CancelToken
  cache/        — ManifestCache, CachePaths
  local_state/  — LocalStateStore, StatePaths
  logger.rs

application/      ← Tauri commands; bridges core+infra to the UI
  commands.rs     — settings, fs init, logging
  updater_commands.rs — start_update, start_repair, cancel_update, UpdaterState
  error.rs        — LauncherError

event_system/     ← serialisable event types emitted to the frontend
  events.rs
  updater_events.rs — UpdaterEvent, UpdateStage, UPDATER_EVENT
```

**Rules:**
- `core/` must never import from `infrastructure/` or `application/`.
- `infrastructure/` must never import from `application/`.
- `application/` is the only layer that may touch Tauri APIs (`AppHandle`, `State`, `Emitter`).

---

## Language

**All code, comments, commit messages, and documentation must be in English.**
Russian is only acceptable in user-facing strings shown inside the UI.

---

## Code style

### Rust

| Tool | Command | Enforced in CI |
|---|---|---|
| Formatter | `cargo fmt --all` | `cargo fmt --all -- --check` |
| Linter | `cargo clippy --all-targets --all-features -- -D warnings` | yes |
| Tests | `cargo test --all-features` | yes |

Active lint groups in `Cargo.toml`:
```toml
[lints.clippy]
pedantic  = { level = "warn", priority = -1 }
nursery   = { level = "warn", priority = -1 }
```

Allowed exceptions (already set in `Cargo.toml`):
- `module_name_repetitions` — allow
- `must_use_candidate` — allow
- `missing_errors_doc` / `missing_panics_doc` — allow (private code)
- `too_many_arguments` — allow

**Never add `#[allow(clippy::...)]` without a comment explaining why.**

### TypeScript / React

| Tool | Command |
|---|---|
| Type check | `pnpm type-check` |
| Linter | `pnpm lint` / `pnpm lint:fix` |
| Formatter | `pnpm format:check` / `pnpm format` |
| Build | `pnpm build` |

---

## Comments policy

Write no comments by default.

Add a comment **only** when the *why* is non-obvious:
- A hidden constraint or platform quirk (e.g. `EnteredSpan` not being `Send`)
- A deliberate trade-off or design decision ("temp dir is preserved for resume")
- A workaround for a known bug or framework limitation
- Ordering that would surprise a reader ("apply deletions only after all downloads succeed")

**Never write:**
- Comments that restate what the code already says
- Numbered step walkthrough comments (`// Step 1:`, `// 2.`)
- Section divider banners (`// ── Helpers ──────`)
- Multi-line docstrings for obvious functions

---

## Pre-commit checklist

Run **all** of these before committing. A commit that breaks any check must not be pushed.

```sh
# from repo root — requires Task (https://taskfile.dev)
task check        # rustfmt + clippy + tsc + eslint + prettier (read-only)
task test         # cargo test --all-features
task build        # pnpm build (smoke)
```

Or step by step:

```sh
# Rust
cd launcher/src-tauri
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features

# Frontend
cd launcher
pnpm type-check
pnpm lint
pnpm format:check
pnpm build
```

**All commands must exit 0 before any commit.**

Auto-fix helpers (do not substitute for the checks above):
```sh
task fix          # cargo fmt + eslint --fix + prettier --write
```

---

## Git conventions

- Branch from `master`, PR back to `master`.
- Commit message format: `type(scope): short description`
  - Types: `feat`, `fix`, `refactor`, `chore`, `docs`, `test`
  - Scope examples: `updater`, `fs`, `ui`, `ci`, `settings`
  - Body: explain *why*, not *what*. Keep it under 72 chars per line.
- One logical change per commit. Do not bundle unrelated fixes.
- Never force-push `master`.
- Never skip hooks (`--no-verify`).

---

## Key runtime behaviour

### Update / Repair flow

```
start_update / start_repair (Tauri command)
  └─ begin_run()           sets is_running, stores CancelToken
       └─ spawn run_updater()
              ├─ emit Stage::checking_files
              ├─ UpdateService::prepare_update_plan  (fetches manifest, diffs local files)
              ├─ emit Stage::downloading
              ├─ DownloadQueue::run (bounded concurrency via Semaphore)
              │    └─ per-file: download_to_temp → verify → install_temp_file
              ├─ emit Stage::finalizing
              └─ emit Done(ActionReport) | Error | Cancelled
```

Frontend listens on `updater_event` channel. Event shape:

```json
{ "type": "stage",    "payload": { "stage": "checking_files" } }
{ "type": "progress", "payload": { "total_files": 42, "completed_files": 3, ... } }
{ "type": "done",     "payload": { "files_checked": 42, "files_restored": 5, ... } }
{ "type": "error",    "payload": { "message": "Network error: ..." } }
{ "type": "cancelled" }
```

### File safety invariants

- A file is **never** written to `game_dir` before passing both size and SHA-256 checks.
- Downloads go to `{app_data}/temp/{sha256}.tmp`.
- The temp dir is **not** wiped on startup — valid temp files are reused for resume.
- Stale `.tmp` files (not in the current plan) are removed at the start of each `DownloadQueue::run`.
- Cancellation cleans up the in-progress temp file; already-installed files are untouched.
- Deletions (`to_delete`) are applied **only after** all downloads succeed.

### Default concurrency

`DEFAULT_CONCURRENCY = 3` simultaneous downloads (defined in `update_service.rs`).

---

## Adding a new Tauri command

1. Implement the logic in `core/` or `infrastructure/` as a pure function or async fn.
2. Write a thin wrapper in `application/commands.rs` or a new `application/X_commands.rs`.
3. Register it in `lib.rs` inside `invoke_handler!(...)`.
4. If the command needs shared state, define a `XState` struct, add `app.manage(XState::default())` in `setup`, and receive it as `State<'_, XState>` in the command.
5. Run the pre-commit checklist.

---

## Adding a new event type

1. Add a variant to `UpdaterEvent` in `event_system/updater_events.rs` (or create a new event enum).
2. Derive `Clone + Serialize`. Use `#[serde(tag = "type", content = "payload", rename_all = "snake_case")]` on the enum.
3. Emit with `app.emit(EVENT_NAME, payload)`.
4. Mirror the type in the frontend TypeScript (`src/types/events.ts` or equivalent).

---

## Error handling

- `LauncherError` is the top-level error type returned by all Tauri commands.
- `DownloadError` is the infrastructure-layer error; it converts to `LauncherError::DownloadError` via `#[from]`.
- `LauncherError::UpdateCancelled` is a soft error (user-initiated), not a crash.
- Return descriptive user-facing messages; log technical details with `tracing::error!`.
- Never `.unwrap()` on anything that can legitimately fail in production paths.
  Panics in Tauri commands propagate as opaque errors to the frontend.

---

## Testing

- Unit tests live in `#[cfg(test)] mod tests` at the bottom of the file they test.
- Use `tempfile::TempDir` for any test that needs the filesystem.
- Do not mock the filesystem or HTTP client — use real temp dirs; test against actual bytes.
- Integration / HTTP tests require a live server; mark them `#[ignore]` and document the setup.
- Test names describe the expected behaviour, not the implementation:
  - ✓ `reuses_valid_temp_file`
  - ✗ `test_verify_existing_temp_ok`

---

## What NOT to do

- Do not add features not requested in the task.
- Do not add error handling for scenarios that cannot happen inside the current call graph.
- Do not add backwards-compatibility shims unless a migration is explicitly needed.
- Do not commit with failing tests, clippy warnings, or fmt diffs.
- Do not write to `game_dir` without going through `FileDownloader::install_temp_file`.
- Do not hold a `MutexGuard` or `EnteredSpan` across an `.await` point.
- Do not make `core/` depend on `infrastructure/` or any async runtime.
