# Contributing to FileSyncHub

First off, thank you for considering contributing to FileSyncHub! It's people like you that make FileSyncHub such a great tool.

## Code of Conduct

This project and everyone participating in it is governed by our Code of Conduct. By participating, you are expected to uphold this code.

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check the issue list as you might find out that you don't need to create one. When you are creating a bug report, please include as many details as possible:

- Use a clear and descriptive title
- Describe the exact steps which reproduce the problem
- Provide specific examples to demonstrate the steps
- Describe the behavior you observed after following the steps
- Explain which behavior you expected to see instead and why
- Include logs if relevant

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion, please include:

- Use a clear and descriptive title
- Provide a step-by-step description of the suggested enhancement
- Provide specific examples to demonstrate the steps
- Describe the current behavior and explain which behavior you expected to see instead
- Explain why this enhancement would be useful

### Pull Requests

- Fill in the required template
- Do not include issue numbers in the PR title
- Include screenshots and animated GIFs in your pull request whenever possible
- Follow the Rust styleguide
- Include thoughtfully-worded, well-structured tests
- Document new code
- End all files with a newline

## Development Process

1. Fork the repo and create your branch from `main`
2. Run the tests to ensure they pass
3. Make your changes
4. Add tests for your changes
5. Ensure the tests still pass
6. Update documentation if needed
7. Create the pull request

### Setting Up Development Environment

1. Install Rust and Cargo:

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Clone your fork:

   ```bash
   git clone https://github.com/yourusername/filesynchub.git
   cd filesynchub
   ```

3. Install development dependencies:

   ```bash
   cargo install cargo-watch cargo-audit cargo-tarpaulin
   ```

4. Set up pre-commit hooks:
   ```bash
   cp hooks/pre-commit .git/hooks/
   chmod +x .git/hooks/pre-commit
   ```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with coverage
cargo tarpaulin

# Run specific test
cargo test test_name

# Run tests with logging
cargo test -- --nocapture
```

### Code Style

We follow the official Rust style guide. Please ensure your code:

- Uses 4 spaces for indentation
- Follows naming conventions
- Includes appropriate comments
- Has no unused imports
- Has no unnecessary allocations

You can use `rustfmt` to automatically format your code:

```bash
cargo fmt
```

And `clippy` to catch common mistakes:

```bash
cargo clippy
```

### Documentation

- Use doc comments (`///`) for public API documentation
- Include examples in doc comments
- Keep README.md up to date
- Update CHANGELOG.md with your changes

### Commit Messages

- Use the present tense ("Add feature" not "Added feature")
- Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
- Limit the first line to 72 characters or less
- Reference issues and pull requests liberally after the first line

### Branch Naming

- `feature/` - for new features
- `fix/` - for bug fixes
- `docs/` - for documentation changes
- `test/` - for test improvements
- `refactor/` - for code refactoring

## Project Structure

```
filesynchub/
├── src/
│   ├── bin/
│   │   └── filesynchub.rs    # Binary entry point
│   ├── plugins/
│   │   ├── mod.rs           # Plugin system
│   │   ├── google_drive.rs  # Google Drive plugin
│   │   └── onedrive.rs      # OneDrive plugin
│   ├── config.rs            # Configuration handling
│   ├── watcher.rs           # File system watcher
│   ├── service.rs           # Service management
│   └── tui/                 # Terminal UI
├── tests/                   # Integration tests
└── Cargo.toml              # Project manifest
```

## Performance Considerations

- Use iterators instead of loops where possible
- Avoid unnecessary allocations
- Use appropriate data structures
- Consider using async/await for I/O operations
- Profile your code with `flamegraph` when making performance-critical changes

## Security Best Practices

- Never commit credentials
- Use secure random number generation
- Validate all user input
- Handle errors appropriately
- Use secure defaults
- Keep dependencies up to date

## Release Process

1. Update version in `Cargo.toml`
2. Update CHANGELOG.md
3. Create a new git tag
4. Push changes and tag
5. Create GitHub release
6. Publish to crates.io

## Getting Help

- Join our Discord server
- Check the documentation
- Ask in GitHub Discussions
- Email the maintainers

## Recognition

Contributors will be recognized in:

- CONTRIBUTORS.md file
- GitHub releases
- Project documentation

Thank you for contributing to FileSyncHub! 🚀
