# Contributing to FileSyncHub

Thank you for your interest in contributing to FileSyncHub! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Contributions](#making-contributions)
- [Code Style](#code-style)
- [Testing](#testing)
- [Documentation](#documentation)
- [Pull Request Process](#pull-request-process)

## Code of Conduct

We follow the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). Please read it before contributing.

## Getting Started

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/your-username/FileSyncHub.git
   cd FileSyncHub
   ```
3. Add the upstream remote:
   ```bash
   git remote add upstream https://github.com/original-owner/FileSyncHub.git
   ```

## Development Setup

### Prerequisites

- Rust 1.70.0 or higher
- Cargo
- Git

### Setting Up the Development Environment

1. Install development dependencies:
   ```bash
   cargo install cargo-watch cargo-audit cargo-tarpaulin
   ```

2. Set up pre-commit hooks:
   ```bash
   cp hooks/pre-commit .git/hooks/
   chmod +x .git/hooks/pre-commit
   ```

3. Create test credentials:
   ```bash
   mkdir -p credentials
   cp examples/credentials/* credentials/
   ```

## Making Contributions

### Branch Naming Convention

- Feature: `feature/description`
- Bug fix: `fix/issue-description`
- Documentation: `docs/description`
- Performance: `perf/description`

### Commit Messages

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- feat: New feature
- fix: Bug fix
- docs: Documentation changes
- style: Code style changes
- refactor: Code refactoring
- perf: Performance improvements
- test: Adding or modifying tests
- chore: Maintenance tasks

## Code Style

We follow the Rust standard style guide with some additional rules:

### General Guidelines

- Use meaningful variable names
- Keep functions small and focused
- Document public APIs
- Use type inference when obvious
- Prefer early returns

### Specific Rules

1. Error Handling:
   ```rust
   // Good
   let result = operation().context("Failed to perform operation")?;

   // Avoid
   let result = operation().unwrap();
   ```

2. Async/Await:
   ```rust
   // Good
   async fn process_file(path: &Path) -> Result<()> {
       let data = read_file(path).await?;
       process_data(data).await
   }
   ```

3. Error Types:
   ```rust
   // Good
   #[derive(Debug, thiserror::Error)]
   pub enum Error {
       #[error("IO error: {0}")]
       Io(#[from] std::io::Error),
   }
   ```

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with coverage
cargo tarpaulin
```

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_file_upload() -> Result<()> {
        // Arrange
        let client = create_test_client().await?;
        let test_file = create_temp_file()?;

        // Act
        let result = client.upload_file(&test_file).await;

        // Assert
        assert!(result.is_ok());
        Ok(())
    }
}
```

## Documentation

### Code Documentation

- Document all public items
- Include examples in doc comments
- Use markdown in doc comments

Example:
```rust
/// Uploads a file to the cloud storage.
///
/// # Arguments
///
/// * `path` - Path to the file to upload
/// * `chunk_size` - Size of each chunk in bytes
///
/// # Examples
///
/// ```
/// # use filesynchub::upload_file;
/// # async fn example() -> Result<()> {
/// let result = upload_file("path/to/file", 1024).await?;
/// # Ok(())
/// # }
/// ```
pub async fn upload_file(path: &Path, chunk_size: usize) -> Result<()> {
    // Implementation
}
```

### User Documentation

- Keep it clear and concise
- Include examples
- Update when adding features
- Use proper formatting

## Pull Request Process

1. Update documentation
2. Add tests for new features
3. Ensure CI passes
4. Update CHANGELOG.md
5. Request review
6. Address feedback

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Documentation update
- [ ] Performance improvement

## Testing
Describe how you tested the changes

## Checklist
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Formatting checked
```

## Additional Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Async Book](https://rust-lang.github.io/async-book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust Style Guide](https://github.com/rust-dev-tools/fmt-rfcs/blob/master/guide/guide.md) 