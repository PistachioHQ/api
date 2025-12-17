# Pistachio API

Protobuf definitions and generated clients for the Pistachio API.

## Setup

```bash
task setup
```

## Common Tasks

```bash
task build      # Build everything
task test       # Run tests
task check      # Run all checks (fmt, lint, audit, deny)
task fmt        # Format code
task lint       # Run linters
```

## Proto Tasks

```bash
task buf:lint       # Lint proto files
task buf:format     # Format proto files
task buf:breaking   # Check for breaking changes
task buf:deps       # Update buf dependencies
```

## Rust Tasks

```bash
task rust:build     # Build Rust code
task rust:test      # Run Rust tests
task rust:clippy    # Run clippy
task rust:fmt       # Format Rust code
task rust:audit     # Security vulnerability check
task rust:deny      # Dependency policy check
```

## Structure

```
api/
├── proto/              # Protobuf definitions
├── gen/
│   ├── rust/           # Generated Rust crate (pistachio-api)
│   └── proto-deps/     # Vendored buf dependencies
└── rust/               # Rust client libraries
```

## License

MIT OR Apache-2.0
