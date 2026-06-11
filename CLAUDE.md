# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

JX3-Tools is a Tauri v2 desktop application for the game JX3 (剑网三), primarily targeting **Windows** (macOS is dev-only; the core features are stubbed there). Three vertical features:

- **Keyboard remapping (改键)** - Copy keyboard config directories between game accounts/characters inside the JX3 `userdata` folder
- **MAC address management (MAC地址)** - View/randomize/restore the NIC MAC address via PowerShell + registry, with optional auto-restore on reboot
- **Hotkey automation (按键)** - Global start/stop hotkeys (tauri-plugin-global-shortcut) that toggle an auto key-press loop, either globally (Interception kernel-driver injection — required because JX3's anti-cheat filters user-mode synthesized input) or sent to a specific window (PostMessage)

## Tech Stack

- **Frontend**: Vue 3 (`<script setup>` + TS) + Vite 8 (Rolldown) + Pinia + Vue Router 5 + UnoCSS
- **UI**: shuimo-ui (水墨 UI, registered globally via `createMUI`) **and** naive-ui (auto-imported via resolver; `useMessage` is the standard toast/error channel)
- **Backend**: Rust, Tauri 2 (plugins: log, dialog, global-shortcut)
- **Testing**: Vitest (jsdom), Cypress e2e; **Linting**: ESLint @antfu/eslint-config

## Common Commands

```bash
pnpm install          # pnpm@10, Node >= 20.19
pnpm tauri:dev        # Run Tauri app in development mode
pnpm tauri:build      # Build production Tauri app
pnpm dev              # Vite dev server only (no Tauri), port 5400
pnpm build            # Type-check (vue-tsc) and build frontend
pnpm type-check       # Type-check only
pnpm test:unit        # Vitest; single test: pnpm test:unit <name-pattern>
pnpm test:e2e         # Cypress against production build
pnpm test:e2e:dev     # Cypress against Vite dev server
pnpm lint             # ESLint with auto-fix
pnpm release          # Version bump via bumpp (see Release section)
```

Unit specs live in `src/**/__tests__/*.spec.ts`. Rust has no test suite; CI (`windows-test.yml`) builds on windows-2022 for every push/PR to main.

## Architecture

A call flows: Vue view → `src/services/*.service.ts` (typed `invoke` wrappers — the **only** place that calls `invoke`) → Rust `commands/*.rs` (thin `#[tauri::command]` layer that pulls services from `AppState`) → Rust `services/*` (business logic). Keep this layering when adding features.

**Rust↔TS contract**: Rust types use `#[serde(rename_all = "camelCase")]`; the matching TS interfaces live in `src/types/` (`hotkey.ts`, `keyboard.ts`, `mac.ts`). Changing one side requires changing the other. Errors cross IPC as plain strings (`AppError` serializes to its Display message, in Chinese); the frontend surfaces them via naive-ui `useMessage`.

### Frontend (`src/`)

- `views/` - one dir per feature with local `components/`: `keyboard/KeyboardView.vue`, `mac-id/MacId.vue`, `hotkey/HotkeyView.vue`; routes defined in `router/index.ts`, all wrapped by `components/layout/MainLayout.vue`
- `services/` - `invoke` wrappers, re-exported from `services/index.ts`
- `composables/` - `useKeyboard`/`useMac` hold feature state as **module-level singleton refs** (shared across components); persistent bits use VueUse `useStorage` (localStorage: keyboard base path, saved templates)
- `stores/hotkey.ts` - the only Pinia store; fetches config/status and subscribes to the Tauri event `hotkey://status` for live status pushes from Rust
- Theme system: `assets/theme.css` (ink-paper CSS variable tokens, light + dark via `.dark` class), `composables/useTheme.ts` (three-state mode), `src/theme/naive.ts` (naive-ui overrides — keep color values in sync with theme.css), `components/layout/PageHeader.vue` (unified page header)
- `@/` alias → `src/` (in `vite.config.ts` and `tsconfig.app.json`)
- `src/types/shims/shuimo-ui.d.ts` - type shim mapped via tsconfig `paths`; shuimo-ui-nightly ships broken type packaging (its d.ts imports raw `.tsx` sources), so TS resolves `shuimo-ui` to this shim while Vite still bundles the real package

Components and icons are auto-imported: `unplugin-vue-components` (with `NaiveUiResolver`) and `unplugin-icons` generate `components.d.ts`; icons import as `~icons/<collection>/<name>`. shuimo-ui components are globally registered in `main.ts`, not auto-imported.

### Backend (`src-tauri/src/`)

- `lib.rs` - builds the app: panic hook, plugins, `AppState::initialize` in setup, all commands in `invoke_handler`
- `main.rs` - also a CLI: `jx3-tools --restore-mac` restores the MAC headlessly (used by the scheduled task)
- `build.rs` + `jx3-tools.manifest` - embeds a custom Windows manifest that requests `requireAdministrator` (UAC prompt every launch). Required for MAC mutation **and** so global key simulation works against JX3's elevated anti-cheat (Windows UIPI drops synthesized input from a lower-integrity process to a higher-integrity foreground window). The manifest fully replaces Tauri's default, so it must also carry DPI awareness, supportedOS, and Common-Controls v6 (the dialog plugin needs it)
- `app_state.rs` - `AppState { Arc<HotkeyService>, Arc<MacService> }`, accessed by commands via `tauri::State`
- `commands/` - thin IPC layer (`mac.rs`, `keyboard.rs`, `hotkey.rs`)
- `services/hotkey/` - `keymap.rs` (key label → scancode/VK/shortcut-string mapping, the single source of truth), `keys.rs` (Interception kernel injection — opens the `\\.\interception00`–`09` **keyboard** devices directly and sends `IOCTL_WRITE` + `KEYBOARD_INPUT_DATA`; `interception.dll` is deliberately NOT used, see below), `driver.rs` (manual keyboard-only driver install/uninstall/state detection + legacy mouse-filter cleanup, REG_MULTI_SZ helpers unit-tested cross-platform), `window.rs` (window enumeration / PostMessage), `config.rs` (validation + JSON persistence), `types.rs`. **Driver install is in-app, user-initiated, and keyboard-only** (`install_hotkey_driver`, hotkey-page banner) — we do NOT run the official `install-interception.exe` at all (it always installs keyboard **and** mouse class filters with no keyboard-only switch, and the mouse filter once bricked a user's mouse). Instead `driver.rs` does the keyboard half by hand: copy the bundled signed `keyboard.sys` (amd64-win7 variant carved from the official installer, shipped under `resources/interception/`) to `%SystemRoot%\System32\drivers\`, register a `keyboard` kernel-driver service via SCM (`SERVICE_KERNEL_DRIVER` / `SERVICE_DEMAND_START` / `SERVICE_ERROR_NORMAL` — ERROR_NORMAL means a load failure is skipped, so the keyboard never bricks), and add `keyboard` to the **Keyboard** class `{4D36E96B-…}` `UpperFilters`. **No mouse registry/service/.sys is ever written.** Failure at any step rolls back; uninstall removes the keyboard filter/service/file and also cleans legacy full-install mouse leftovers. `interception.dll` can't be used because its `create_context` requires all 20 devices (10 kbd + 10 mouse) to open — impossible under keyboard-only install; hence the direct-device client in `keys.rs`. Needs a reboot to take effect (the filter loads when the keyboard device stack rebuilds)
- `services/mac/` - PowerShell-driven (`scripts/*.ps1` assembled by `scripts.rs`): writes the `NetworkAddress` registry override, restarts the adapter, then **reads the MAC back to verify** the driver accepted it (rolls back + errors if not — many drivers, esp. wireless, silently ignore the override); restore clears overrides on all physical adapters (falls back to `PermanentAddress`); needs admin (errors map to `PermissionDenied`); no local state files — the registry and the Task Scheduler task `JX3ToolsMacRestore` (onlogon, `/rl HIGHEST`) are the source of truth
- `services/keyboard.rs` - directory tree + copy. Encodes the JX3 userdata layout: tree depth 4 = a character dir (returned with `is_dir: false` to mark it selectable); `userpreferences` dirs are skipped; copy is **swap-replace** (`swap_replace_dir`: copy source to a sibling tmp dir, move old target to a backup, rename into place — any failure leaves the target intact); symlinks are rejected/skipped
- `services/plugin_data.rs` - plugin config sync (改键页的"同步插件配置"开关). Locates `interface/` next to the `userdata` ancestor of the selected role paths, scans `*#data` dirs, and reverse-maps role name+server → role UID by parsing each `<uid>@<edition>/info.jx3dat` (**GBK-encoded** one-line Lua table, parsed by substring extraction via `encoding_rs`; duplicate name+server picks the latest `time`). Framework-style dirs (my#data/lm#data): swap-replace only the `config/` subdir — never `info.jx3dat` (role identity) or `userdata/` (chat logs/stats). Single-file dirs (SG#data): copy `<src_uid>.jx3dat` → `<tgt_uid>.jx3dat` via tmp+rename. Global dirs (JX#DATA, no per-UID entries) are ignored. Per-dir failures land in `PluginSyncReport.skipped` (frontend warnings); unknown source/target role is a hard error (target must log in once to generate its UID dir). Cross-platform, fully unit-tested
- `services/cloud/` - WebDAV cloud sync (改键页"云同步"弹窗): users bind their own drive (Nutstore/坚果云 preset; any WebDAV works) via server URL + username + app password — deliberately no vendor open-platform credentials (Aliyun Drive suspended individual developer onboarding 2025-07; WebDAV has no developer role to revoke). `webdav.rs`: `CloudStorage` trait (get/put/check) + reqwest blocking impl using only GET/PUT/MKCOL/PROPFIND — cloud listing reads `jx3-tools/manifest.json` instead of PROPFIND traversal, so no XML parsing; URL building percent-encodes Chinese per segment. `pack.rs`: dir↔zip (skips symlinks). `sync.rs`: upload packs the userdata role dir (keybinding.zip) + per-data-dir plugin configs (plugins.zip: `<dir>/config/**` framework-style, `<dir>/data.jx3dat` single-file; UIDs never enter the archive — download re-resolves the _target_ role's UID via plugin_data, making archives account-portable); download unpacks to temp then `swap_replace_dir` into place. `config.rs`: plaintext JSON in `config_dir/jx3-tools/cloud_config.json` (app password is revocable on the drive side). Sync logic is fully unit-tested against an in-memory `CloudStorage`
- `error.rs` - `AppError` (thiserror) + `AppResult<T>`; user-facing messages are Chinese

Backend persistent state lives in `dirs::config_dir()/jx3-tools/` (`hotkey_config.json`).

### Hotkey runtime model (the most intricate part)

`HotkeyService` keeps a `Mutex<HotkeyInner>` (config + status + optional `Runner`). On init/save it (re)registers the start/stop shortcuts via `tauri-plugin-global-shortcut` (cross-platform; combos like `Ctrl+Alt+F5` supported); the handlers run on the event loop and dispatch start/stop to a **new thread** (never block the event loop). The runner is a loop thread pressing the trigger key every `interval_ms` (20–60000ms validated), in `Global` mode (Interception simulate) or `Window` mode (PostMessage to a stored HWND, revalidated before start). Threads are stopped via `AtomicBool` + join **with 500ms timeout** (detach on timeout) — this pattern exists to fix real freeze bugs; keep it. Every status change is emitted to the frontend via `app.emit(HOTKEY_STATUS_EVENT)`.

### Platform gating

Key simulation and MAC mutation are Windows-only (`windows` crate under `[target.'cfg(windows)'.dependencies]`); hotkey listening itself is cross-platform. Non-Windows code paths are `#[cfg]`-gated stubs that return "仅支持 Windows" errors or empty lists — when touching gated code, make sure **both** cfg branches still compile (macOS dev machine builds the non-Windows side; CI builds the Windows side). Test real hotkey behavior on Windows only.

### Tauri Commands (IPC)

- MAC: `get_mac_info`, `randomize_mac_address`, `restore_mac_cmd`, `get_auto_restore_setting`, `set_auto_restore_setting`
- Keyboard: `list_directory_contents`, `cp_source_to_target`, `sync_plugin_config`, `open_folder`
- Hotkey: `get_hotkey_config`, `save_hotkey_config`, `get_hotkey_status`, `stop_hotkey_task`, `list_windows`, `check_window_valid`, `install_hotkey_driver`, `uninstall_hotkey_driver`, `remove_mouse_filter`
- Cloud: `get_cloud_config`, `save_cloud_config`, `test_cloud_connection`, `cloud_upload_role`, `cloud_list_roles`, `cloud_download_role`

## Code Style

- @antfu/eslint-config with `1tbs` brace style (`curly` off); max line length 120 (URLs ignored); `console.log` allowed
- UI text and Rust error messages are Chinese; code identifiers and comments mostly English
- `AGENTS.md` is git-ignored and ESLint-ignored

## Commits & Release

Conventional Commits in English: `<type>(<scope>): <subject>` (e.g. `fix(hotkey): wrap KeyFilter in Filter enum`); body uses `-` bullets. Husky enforces this: `commit-msg` runs commitlint, `pre-commit` runs lint-staged (`eslint --fix`) — lint errors block the commit. Co-Author trailer pins the exact model, e.g. `Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>`.

`pnpm release` (bumpp, config in `.bumpp.config.cjs`) bumps `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml`, commits (history uses `chore: release vX.Y.Z`) and tags `vX.Y.Z`; it does **not** push — run `git push --follow-tags`. Release builds are manual: the `release.yml` workflow (workflow_dispatch, takes an existing `v*` tag) builds the Windows NSIS installer (perMachine) via tauri-action. See `docs/RELEASE.md`.
