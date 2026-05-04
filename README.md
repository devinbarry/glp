# glp

A command-line tool for monitoring and debugging GitLab CI/CD pipelines.

glp gives you quick access to pipeline status, job details, and logs directly from your terminal -- no browser tabs or `glab api` + `jq` chains required.

[![CI](https://github.com/devinbarry/glp/actions/workflows/release.yml/badge.svg)](https://github.com/devinbarry/glp/actions/workflows/release.yml)
[![crates.io](https://img.shields.io/crates/v/glp.svg)](https://crates.io/crates/glp)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Why glp?

The GitLab REST API works fine with `glab api ... | jq ...`, but every common pipeline question — "did my push pass CI?", "what failed?", "show me the log" — turns into a custom command chain. glp packages those questions into typed subcommands with clean output, JSON support, and zero glue.

## Installation

### From crates.io

```sh
cargo install glp
```

### From source

Requires [Rust](https://rustup.rs/) 1.70+.

```sh
cargo install --path . --root ~/.local --force
```

Or with [just](https://github.com/casey/just):

```sh
just install
```

## Authentication

glp needs a GitLab personal access token with `read_api` scope (plus `api` scope if you want to retry jobs). It looks for a token in this order:

1. `GITLAB_TOKEN` environment variable
2. `GITLAB_PRIVATE_TOKEN` environment variable
3. [glab](https://gitlab.com/gitlab-org/cli) config file (`~/.config/glab-cli/config.yml` on Linux, `~/Library/Application Support/glab-cli/config.yml` on macOS)

If you already use glab and have authenticated with `glab auth login`, glp will pick up your token automatically.

## Usage

Run glp from within a GitLab-hosted git repository. It detects the project and host from your `origin` remote.

### Check pipeline status

```sh
# Current branch
glp status

# Specific ref
glp status --ref main

# JSON output
glp status --json
```

Output:

```
Pipeline #12345 (main) - success [3m 42s]

STAGE        JOB                  STATUS     DURATION
build        compile              success    1m 23s
test         unit-tests           success    2m 05s
test         lint                 success    0m 34s
deploy       deploy-staging       skipped    -
```

### List jobs in a pipeline

```sh
glp jobs 12345

glp jobs 12345 --json
```

Output:

```
JOB                  ID       STATUS     DURATION   STAGE
compile              67890    success    1m 23s     build
unit-tests           67891    success    2m 05s     test
lint                 67892    success    0m 34s     test
deploy-staging       67893    skipped    -          deploy
```

### View job logs

```sh
# Full log
glp log 67891

# Last 50 lines
glp log 67891 --tail 50
```

### Retry a failed job

```sh
glp retry 67891
```

Output:

```
Retried job 67891 (unit-tests) - new job ID: 67900
```

### Targeting a different project

All commands accept `--project` to override the auto-detected project:

```sh
glp status --project my-group/my-project
glp jobs 12345 --project my-group/subgroup/my-project
```

## Configuration

All configuration is resolved automatically with fallbacks:

| Setting   | 1st (highest priority)     | 2nd                      | 3rd                       | 4th         |
|-----------|----------------------------|--------------------------|---------------------------|-------------|
| **Token** | `GITLAB_TOKEN` env         | `GITLAB_PRIVATE_TOKEN` env | glab config              |             |
| **Host**  | `GITLAB_HOST` env          | git remote origin        | glab config default host  | `gitlab.com` |
| **Project** | `--project` flag         | git remote origin        |                           |             |

### Self-hosted GitLab

glp works with self-hosted GitLab instances. If your git remote points to a self-hosted instance, glp detects the host automatically. Otherwise, set `GITLAB_HOST`:

```sh
export GITLAB_HOST=gitlab.example.com
```

## Contributing

Issues and PRs are welcome on GitHub. Note that primary development happens
upstream and this repo is updated when releases are cut, so PR review may take
longer than typical. For bug reports, please include the glp version
(`glp --version`), GitLab host (self-hosted vs gitlab.com), and a redacted
command transcript.

## License

MIT — see [LICENSE](LICENSE).
