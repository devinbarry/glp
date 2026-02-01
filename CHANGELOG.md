# Changelog

All notable changes to this project will be documented in this file.

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


### Fixed

- correct .gitignore to ignore settings.local.json
- improve glab config resolution for macOS and multi-host

