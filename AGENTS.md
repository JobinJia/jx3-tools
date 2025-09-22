# Repository Guidelines

## Project Structure & Module Organization
Source code lives in `src/`, with feature directories such as `views/HotkeyView.vue` and shared utilities under `composables/`, `stores/`, and `utils/`. Frontend entry points (`main.ts`, `App.vue`) mount the Vue app, while static assets stay in `public/`. The Tauri backend is in `src-tauri/`, where `src/main.rs` wires Rust commands and plugins; avoid editing generated code under `src-tauri/gen/`. End-to-end specs sit in `cypress/e2e/`, and packaging artifacts land in `dist/` after builds.

## Build, Test, and Development Commands
- `pnpm install` installs JavaScript dependencies and wires Husky hooks.
- `pnpm dev` runs the Vite dev server; use `pnpm tauri:dev` when you need the Rust shell.
- `pnpm build` executes type-checking plus `vite build` to emit `dist/`; follow with `pnpm preview` for a local production server.
- `pnpm lint` applies the Antfu ESLint preset; `pnpm type-check` runs `vue-tsc` standalone.
- `pnpm test:unit` launches Vitest in watch mode; `pnpm test:e2e` boots the preview server and executes Cypress headlessly.

## Coding Style & Naming Conventions
We use TypeScript, `<script setup>` SFCs, and two-space indentation (enforced by `@antfu/eslint-config`). Components and views stay in PascalCase (`HotkeyView.vue`), stores/composables in camelCase (`accountTree.ts`), and route paths in kebab-case. Prefer auto-imported APIs via `unplugin-vue-components` and UnoCSS utilities for styling. Run `pnpm lint` or rely on the pre-commit hook (lint-staged + Husky) to keep formatting consistent.

## Testing Guidelines
Unit tests live beside components inside `__tests__` folders with `*.spec.ts` suffixes; mock external services via Pinia or local fixtures. Trigger `pnpm test:unit --coverage` when evaluating regressions, and ensure new features ship with at least one focused spec. Cypress specs belong in `cypress/e2e/` following feature-based subfolders; use `pnpm test:e2e:dev` for an interactive runner before committing.

## Commit & Pull Request Guidelines
Commit messages follow Conventional Commits (`feat: add keyboard shortcut import`), which is enforced by `commitlint`. Keep commits scoped and include chore/build changes separately from feature code. Pull requests should describe user-visible changes, list test commands run, and attach screenshots or recordings when the UI shifts. Confirm both `pnpm lint` and the relevant test suite pass before requesting review, and mention linked issues using `Closes #123` when applicable.

## Tauri Runtime Notes
Rust-side modules under `src-tauri/src/` mirror Vue features (`keyboard/`, `mac_addr/`); keep command names synchronized with the frontend API wrappers in `src/request/`. Adjust build-time config in `src-tauri/tauri.conf.json`, and store machine-specific secrets in local `.env` files rather than the repo.
