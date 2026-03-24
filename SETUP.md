```markdown
<!-- dokkaebi:generated -->
# Setup Guide

## Prerequisites

| Tool | Minimum Version | Notes |
|------|----------------|-------|
| [Rust](https://rustup.rs/) | 1.70.0 | Includes `cargo`; edition 2021 required |
| [Git](https://git-scm.com/) | 2.x | Required for version control operations |

Verify your installation:

```bash
rustc --version
cargo --version
```

## Installation

1. **Clone the repository**

   ```bash
   git clone https://github.com/your-org/git-ghosts.git
   cd git-ghosts
   ```

2. **Build the project**

   ```bash
   cargo build --release
   ```

3. **Verify the build**

   ```bash
   cargo check
   ```

## Environment Variables

No environment variables are required for the default configuration.

> If you add environment-specific configuration, document each variable here in the format below.

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| _(none)_ | — | — | — |

## Running

Start the application in development mode:

```bash
cargo run
```

Run the release build:

```bash
./target/release/git-ghosts
```

## Testing

Run the full test suite:

```bash
cargo test
```

Run a targeted subset of tests:

```bash
cargo test <test_name>
```

Check formatting and lint before submitting:

```bash
cargo fmt --check
cargo clippy
```
```