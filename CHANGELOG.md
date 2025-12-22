# Changelog

All notable changes to this project will be documented in this file.

## [2.0.0] - 2024-12-22

### Breaking Changes
- Refactored frontend state management architecture
- Updated error response format from backend with error codes
- Renamed package from `vite-vue3-template` to `jx3-tools`

### Added
- **Frontend**:
  - Structured error handling with error codes (`src/utils/error.ts`)
  - Composables index for cleaner imports (`src/composables/index.ts`)
  - Utils index for utility exports (`src/utils/index.ts`)

- **Backend**:
  - New error variants: `Validation`, `PlatformNotSupported`
  - Error codes for frontend categorization
  - Command validation layer with logging
  - Validation helpers: `validate_mac_address`, `validate_path_not_empty`
  - PowerShell scripts externalized to separate files

### Changed
- **Frontend**:
  - MacId.vue now uses `useMac` composable (eliminated code duplication)
  - HotkeyView.vue imports types from `@/types` instead of store
  - State management standardized: Pinia for hotkey, composables for keyboard/MAC

- **Backend**:
  - HotkeyService split into modules: `config.rs`, `keys.rs`, `shortcuts.rs`, `types.rs`
  - MacService split into modules with external PowerShell scripts
  - All commands now include debug logging
  - Input validation at command layer before service calls

### Removed
- Obsolete composables: `accountTree.ts`, `basePath.ts`, `keyboardDir.ts`
- Unused counter store: `counter.ts`
- Unused Cargo dependencies: `regex`, `rand`

### Fixed
- Version synchronization between package.json, tauri.conf.json, and Cargo.toml
- CI pnpm version mismatch (10.6.2 -> 10.17.0)
- Package metadata in Cargo.toml (name, authors, license, repository)

## [1.0.0] - 2024-12-XX

### Added
- Initial release with keyboard remapping, MAC address management, and hotkey automation
