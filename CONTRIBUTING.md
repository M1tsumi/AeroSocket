# Contributing to AeroSocket

Thank you for your interest in contributing to AeroSocket! This document provides guidelines and information for contributors.

## 🚀 Getting Started

### Prerequisites

- Rust 1.70 or higher
- Git
- Basic familiarity with Rust and WebSocket concepts

### Development Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/M1tsumi/AeroSocket.git
   cd AeroSocket
   ```

2. **Install development dependencies**
   ```bash
   make setup
   # or manually:
   rustup component add rustfmt clippy llvm-tools-preview
   cargo install cargo-audit cargo-deny cargo-llvm-cov cargo-watch cargo-expand
   ```

3. **Verify the setup**
   ```bash
   make check-all
   ```

## 🏗️ Project Structure

```
AeroSocket/
├── aerosocket/           # Main library (user-facing)
├── aerosocket-core/      # Core protocol implementation
├── aerosocket-server/    # Server implementation
├── aerosocket-client/    # Client implementation
├── aerosocket-transport-tcp/  # TCP transport
├── aerosocket-transport-tls/  # TLS transport
├── aerosocket-wasm/      # WASM support
├── examples/             # Example applications
├── benches/              # Performance benchmarks
├── tests/                # Integration tests
└── docs/                 # Documentation
```

## 🧪 Development Workflow

### Running Tests

```bash
# Run all tests
make test

# Run tests with coverage
make test-coverage

# Run specific test categories
make integration-test
make doc-test
make example-test
```

### Code Quality

```bash
# Format code
make fmt

# Check formatting
make fmt-check

# Run lints
make clippy

# Security audit
make audit

# Run all checks
make check-all
```

### Development

```bash
# Watch for changes and run tests
make watch

# Watch for changes and run clippy
make watch-clippy

# Build all crates
make build

# Build release
make build-release
```

## 📝 Contributing Guidelines

### Code Style

- Use `cargo fmt` for formatting
- Follow clippy recommendations
- Write comprehensive documentation
- Include examples in doc comments
- Use meaningful variable and function names

### Testing

- Write unit tests for all public functions
- Include integration tests for major features
- Add benchmarks for performance-critical code
- Test error handling paths
- Ensure 90%+ test coverage

### Documentation

- Document all public APIs
- Include usage examples
- Update CHANGELOG.md for significant changes
- Add inline comments for complex logic
- Keep README.md up to date

### Commit Messages

Use the following format:
```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes
- `refactor`: Code refactoring
- `test`: Test changes
- `chore`: Maintenance tasks

Examples:
```
feat(server): add connection pooling
fix(client): handle reconnection errors
docs(readme): update installation instructions
```

## 🐛 Bug Reports

When reporting bugs, please include:

1. **Description**: Clear description of the issue
2. **Steps to reproduce**: Minimal reproduction case
3. **Expected behavior**: What should happen
4. **Actual behavior**: What actually happens
5. **Environment**: OS, Rust version, relevant dependencies
6. **Logs**: Error messages, stack traces if available

Use the provided bug report template when creating issues.

## ✨ Feature Requests

When requesting features, please include:

1. **Use case**: Why this feature is needed
2. **Proposed API**: How the feature should work
3. **Alternatives**: Other approaches considered
4. **Implementation**: Any implementation ideas

Use the provided feature request template when creating issues.

## 🔧 Pull Request Process

### Before Submitting

1. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes**
   - Write code following the guidelines
   - Add tests for new functionality
   - Update documentation
   - Ensure all tests pass

3. **Run quality checks**
   ```bash
   make check-all
   ```

4. **Commit your changes**
   ```bash
   git commit -m "feat: add your feature"
   ```

5. **Push and create PR**
   ```bash
   git push origin feature/your-feature-name
   ```

### Pull Request Requirements

- [ ] All tests pass
- [ ] Code is properly formatted
- [ ] No clippy warnings
- [ ] Documentation is updated
- [ ] CHANGELOG.md is updated
- [ ] Feature has tests
- [ ] Security audit passes

### Review Process

1. **Automated checks**: CI/CD pipeline runs tests and quality checks
2. **Code review**: Maintainers review the code
3. **Testing**: Additional testing if needed
4. **Approval**: Merge approval from maintainers
5. **Merge**: Code is merged to main branch

## 🏷️ Release Process

Releases follow semantic versioning:

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

Release steps:
1. Update version numbers
2. Update CHANGELOG.md
3. Create release tag
4. Automated release process publishes crates
5. Documentation is deployed

## 🤝 Community Guidelines

### Code of Conduct

- Be respectful and inclusive
- Welcome newcomers and help them learn
- Focus on constructive feedback
- Assume good intentions
- Report issues privately if needed

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and discussions
- **Discord**: Real-time chat and community support
- **Email**: Private security concerns

### Getting Help

- Check existing issues and documentation
- Ask questions in GitHub Discussions
- Join our Discord community
- Consult the API documentation

## 🎯 Areas for Contribution

We welcome contributions in these areas:

### Core Features
- Protocol implementation improvements
- Performance optimizations
- New transport implementations
- Extension support

### Server Features
- Connection management
- Load balancing
- Metrics and monitoring
- Security features

### Client Features
- Reconnection strategies
- Connection pooling
- Error handling
- Performance improvements

### Documentation
- API documentation
- Tutorials and guides
- Examples and demos
- Performance guides

### Tooling
- Benchmark suites
- Testing utilities
- Development tools
- CI/CD improvements

## 📋 Development Tasks

Check our GitHub Projects for current development tasks and priorities. We maintain several project boards:

- **Bug Fixes**: High-priority bugs and issues
- **Features**: Planned features and enhancements
- **Documentation**: Documentation improvements
- **Performance**: Performance optimizations
- **Infrastructure**: Tooling and CI/CD

## 🏆 Recognition

Contributors are recognized in several ways:

- **README.md**: Listed as contributors
- **CHANGELOG.md**: Credited for specific contributions
- **Release Notes**: Mentioned for significant contributions
- **Community Recognition**: Highlighted in community channels

## 📞 Getting in Touch

If you have questions about contributing:

- **Issues**: Use GitHub Issues for questions
- **Discussions**: Use GitHub Discussions for general topics
- **Email**: Contact maintainers directly for private matters
- **Discord**: Join our community for real-time help

---

Thank you for contributing to AeroSocket! Your contributions help make WebSocket development better for everyone. 🚀
