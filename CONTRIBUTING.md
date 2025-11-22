# Contributing to typed-fsm

Thank you for your interest in contributing to typed-fsm! We welcome contributions from everyone.

## Getting Started

1. **Fork the repository**
   ```bash
   git clone https://github.com/YOUR_USERNAME/typed-fsm.git
   cd typed-fsm
   ```

2. **Create a new branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make your changes**
   - Write clear, readable code
   - Add tests for new features
   - Update documentation as needed

4. **Run the test suite**
   ```bash
   # Run all tests
   cargo test

   # Run with output
   cargo test -- --nocapture

   # Run clippy
   cargo clippy --all-targets --all-features

   # Format code
   cargo fmt
   ```

5. **Commit your changes**
   ```bash
   git add .
   git commit -m "Add feature: description"
   ```

6. **Push and create a Pull Request**
   ```bash
   git push origin feature/your-feature-name
   ```

## Code Style

- **Format:** Run `cargo fmt` before committing
- **Lint:** Run `cargo clippy` and fix all warnings
- **Tests:** All tests must pass (`cargo test`)
- **Documentation:** Add doc comments for public APIs
- **Examples:** Update examples if relevant

## Pull Request Guidelines

- **Title:** Clear and descriptive
- **Description:** Explain what changed and why
- **Tests:** Include tests for new functionality
- **Documentation:** Update README.md if needed
- **Changelog:** Add entry to CHANGELOG.md under "Unreleased"

## Testing

### Running Tests

```bash
# All tests
cargo test

# Specific test suite
cargo test --test integration_tests
cargo test --test coverage_tests
cargo test --test edge_cases_tests

# Doc tests
cargo test --doc

# With coverage (requires tarpaulin)
cargo tarpaulin --out Html
```

### Writing Tests

- Add unit tests in the same file as the code
- Add integration tests in `tests/` directory
- Add examples in `examples/` directory
- Ensure doc tests compile and run

## Documentation

### Doc Comments

Use Rust doc comments (`///`) for public APIs:

```rust
/// Brief description.
///
/// # Examples
///
/// ```rust
/// use typed_fsm::Transition;
/// let t = Transition::None;
/// ```
///
/// # Panics
///
/// Describe panic conditions if any.
pub enum Transition<S> { ... }
```

### README Updates

Update README.md for:
- New features
- API changes
- New examples
- Breaking changes

## Reporting Issues

When reporting bugs, please include:

- Rust version (`rustc --version`)
- Operating system
- Minimal reproduction code
- Expected vs actual behavior
- Error messages (if any)

## Feature Requests

For feature requests, please:

- Describe the use case
- Explain why it's useful
- Provide example API if possible
- Discuss alternatives

## Code of Conduct

- Be respectful and inclusive
- Welcome newcomers
- Focus on constructive feedback
- Assume good intentions

## Questions?

- Open an issue for questions
- Check existing issues and PRs
- Read the documentation at [docs.rs/typed-fsm](https://docs.rs/typed-fsm)

## License

By contributing, you agree that your contributions will be licensed under the same terms as the project (MIT OR Apache-2.0).
