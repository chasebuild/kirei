# Contributing

Thanks for helping improve this Rust CLI template. Contributions should keep the project lean and easy to copy-paste into downstream projects.

## Local setup

- `cargo fetch`
- `cargo fmt` (or `cargo format` using the provided alias)
- `cargo lint` to run `clippy` with workspace-wide settings
-
- `cargo test`

## Development workflow

1. Work on a feature branch with focused commits.
2. Keep changes localized to a single behavior (e.g., add a command, update a prompt).
3. Run `cargo format` followed by `cargo lint` and `cargo test` before submitting.

## Testing

- Use `cargo test` for the regular suite.
- Provide unit coverage for new commands when reasonable; avoid relying on network or provider credentials.
