# Changelog

All notable changes to this project will be documented in this file.

## [0.1.3] - 2026-05-04

First release published to crates.io. Now installable with `cargo install glp`.

### Added

- LICENSE file (MIT) at repo root
- Crate metadata for crates.io: `repository`, `homepage`, `keywords`, `categories`
- README badges (CI status, crates.io version, license), "Why glp?" section, `cargo install glp` instructions, Contributing section
- GitHub Actions release workflow that publishes the crate and creates a GitHub Release on `v*` tags

### Changed

- Build pipeline rewritten to run `cargo fmt --check` and `cargo clippy -- -D warnings` in addition to `cargo test` on every push

## [0.1.2] - 2026-04-03

### Fixed

- `glp status` no longer fails with "No pipeline found for ref 'HEAD'" in detached HEAD state — now falls back to querying by commit SHA

### Changed

- Release process is now two-step (`release-prep` + `release-finish`) to allow changelog review before tagging

## [0.1.1] - 2026-02-14


### Changed

- skip cargo-husky hook installation in CI
- add README and CLAUDE.md
- add comprehensive test coverage across all modules
- add release workflow to CLAUDE.md

## [0.1.0] - 2026-02-01


### Added

- scaffold CLI with clap subcommands
- add error types
- add config resolution from env and glab
- add GitLab API client
- add Pipeline and Job models
- add output formatting with colored tables
- implement status, jobs, log, retry commands


### Changed

- add initial design document for glp
- update design with yaml_serde and API endpoints
- add implementation plan
- add .gitignore
- clippy fixes and formatting
- add unit tests for config and models
- add GitLab CI for running tests
- use latest rust image instead of 1.82
- add version control infrastructure
- add changelog for v0.1.0


### Fixed

- correct .gitignore to ignore settings.local.json
- improve glab config resolution for macOS and multi-host

