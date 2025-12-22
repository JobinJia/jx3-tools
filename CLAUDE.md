# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

JX3-Tools is a Tauri v2 desktop application for the game JX3 (剑网三). It provides utility features including:

- **Keyboard remapping** - Copy keyboard configuration files between game accounts/characters
- **MAC address management** - View, modify, and restore network MAC addresses
- **Hotkey automation** - Configure and run automated key sequences with global shortcuts

## Tech Stack

- **Frontend**: Vue 3 + TypeScript + Vite 7 + Pinia + Vue Router
- **Backend**: Tauri 2 (Rust)
- **UI**: shuimo-ui (水墨 UI) + UnoCSS
- **HTTP Client**: Alova
- **Testing**: Vitest (unit), Cypress (e2e)
- **Linting**: ESLint with @antfu/eslint-config

## Common Commands

```bash
pnpm install          # Install dependencies
pnpm tauri:dev        # Run Tauri app in development mode
pnpm tauri:build      # Build production Tauri app
pnpm dev              # Run Vite dev server only (no Tauri)
pnpm build            # Type-check and build frontend
pnpm test:unit        # Run unit tests with Vitest
pnpm lint             # Lint and auto-fix
pnpm release patch    # Bump version (patch/minor/major), updates both package.json and tauri.conf.json
```

## Architecture

### Frontend (`src/`)

- `views/` - Page components: KeyboardView (改键), MacId (MAC地址), HotkeyView (按键)
- `stores/` - Pinia stores (counter.ts, hotkey.ts)
- `composables/` - Vue composables for shared logic
- `request/` - Alova HTTP client configuration
- `router/index.ts` - Vue Router with MainLayout wrapping all views

### Backend (`src-tauri/src/`)

- `lib.rs` - Main Tauri app entry point, registers all commands and plugins
- `app_state.rs` - Application state management
- `services/` - Business logic modules:
  - `mac.rs` - MAC address operations
  - `hotkey.rs` - Hotkey configuration and automation runner
- `keyboard/mod.rs` - Directory reading and file copy operations for keyboard configs
- `error.rs` - Custom error types

### Tauri Commands (IPC)

Commands exposed to frontend via `tauri::command`:

- MAC: `get_mac_address`, `change_mac_address`, `restore_mac_cmd`,
  `get_auto_restore_setting`, `set_auto_restore_setting`
- Keyboard: `list_directory_contents`, `cp_source_to_target`
- Hotkey: `get_hotkey_config`, `save_hotkey_config`, `get_hotkey_status`, `stop_hotkey_task`

## Code Style

- ESLint uses @antfu/eslint-config with `1tbs` brace style
- Max line length: 120 characters
- `console.log` is allowed
- UnoCSS for utility-first styling
