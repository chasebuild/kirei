# CLI Template

Lightweight starter for a Rust command-line tool built with `clap` plus the interactive prompts in `cliclack`.

## Features

- `clap`-driven commands (`init`, `greet`, `config show`, `config path`) with a shared `core` library.
- `cliclack` prompts (`intro`, `input`, `confirm`, `note`, `outro`) that show how to do an interactive setup flow.
- `ConfigStore` helper that serializes a `Config` to the platform config directory used across commands.
- Workspace aliases `cargo format`/`cargo lint` plus CI that runs format, lint, build, and test on every push/PR.

## Using the CLI

```bash
cargo run -- init
cargo run -- greet
cargo run -- greet --user-name "Ada Lovelace"
cargo run -- config show
cargo run -- config path
```

`init` prompts for the user name (unless you pass `--user-name`) and persists it via `ConfigStore`. The other commands load the same config and either greet that user or dump the config state.

## Workspace Layout

- `core/`: reusable library that holds `Config`, `ConfigStore`, and the sample `greeting` helper.
- `cli/`: binary that wires `clap`, `cliclack`, and the core library into a tiny example CLI.

## Development

Use the provided workspace aliases:

```bash
cargo format   # runs `cargo fmt --all`
cargo lint     # runs `cargo clippy --workspace --all-targets --all-features`
cargo test
```

Every push/PR also triggers `.github/workflows/ci.yml`, which enforces the same format, lint, build, and test flow.
