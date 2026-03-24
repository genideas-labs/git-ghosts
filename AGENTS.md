<!-- dokkaebi:generated -->
# AGENTS.md — git-ghosts

Follow the human's latest instruction first. Keep changes small. Read before editing. Ask when unsure.

## Project Snapshot
- Project: `git-ghosts`
- Language: Rust (`rust`)
- Manifest: `Cargo.toml`
- Build tool: `cargo`
- Package manager: `cargo`
- Test framework: `cargo test`

## Repository Layout
- Source code lives in `src/`
- Use these source roots only. Do not add new top-level packages.

## Entry Points
- None detected

## Frameworks
- None detected

## Working Rules
- Read existing files before editing them.
- Preserve human instructions unless explicitly asked to replace them.
- Use the smallest safe change that satisfies the task.
- Verify with targeted tests and lint before claiming completion.
- Refresh generated files only when the marker is present.

## Coding Conventions
- Preserve the repository's existing style and naming.
- Keep diffs small, targeted, and reviewable.
- Add or update tests with every behavior change.
- Reuse existing modules and utilities before adding abstractions.

Language-specific conventions:
- Format: `cargo fmt`
- Lint: `cargo clippy`
- Test: `cargo test`
- Build: `cargo build --release`
- Edition: 2021+

## Testing
- Test directory: `tests/`
- Test framework: `cargo test`
- Run targeted tests first, then the broader suite.
- Cover new behavior with tests before handoff.
- Suggested checks: `cargo test` and `cargo fmt --check`

## Commands
- `cargo build --check`
- `cargo clippy --version`
- `cargo test`
- `dokkaebi scan`
- `dokkaebi intake <requirements-file>`
- `dokkaebi reconcile`

## Update Policy
- Generated files include the `<!-- dokkaebi:generated -->` marker.
- Dokkaebi may refresh these files during `scan` when the marker is present.
- Remove the marker to opt out of future refreshes.
