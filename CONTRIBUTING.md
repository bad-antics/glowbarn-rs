# Contributing to GlowBarn

Thank you for your interest in contributing to GlowBarn! This document provides guidelines for contributing.

## Getting Started

1. Fork the repository
2. Clone your fork locally
3. Install Rust toolchain (1.75+)
4. Run `cargo build` to verify setup

## Development Setup

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/glowbarn-rs.git
cd glowbarn-rs

# Build the project
cargo build

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run
```

## Code Style

- Follow Rust standard style guidelines
- Use `cargo fmt` before committing
- Run `cargo clippy` and address warnings
- Keep functions small and focused
- Document public APIs with doc comments

## Pull Request Process

1. Create a feature branch from `main`
2. Make your changes with clear commit messages
3. Update documentation if needed
4. Add tests for new functionality
5. Ensure all tests pass: `cargo test`
6. Submit PR with description of changes

## Commit Message Format

```
type(scope): brief description

Longer description if needed.

Fixes #issue_number
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

## Reporting Issues

- Search existing issues first
- Use issue templates when available
- Include reproduction steps
- Provide system information (OS, Rust version)

## Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help newcomers feel welcome

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
