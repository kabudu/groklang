# Contributors Guide

Welcome to GrokLang! We're excited to have you contribute to this open-source project. This document outlines how to contribute effectively to the project.

## Code of Conduct

We are committed to providing a welcoming and inclusive environment for all contributors. Please treat all community members with respect and follow these principles:

- **Be respectful**: Disagreements are natural. Discuss ideas constructively and respectfully.
- **Be inclusive**: Welcome people of all backgrounds and experience levels.
- **Be professional**: Keep conversations focused on the project and avoid personal attacks.
- **Report issues**: If you witness or experience harassment, please report it to the project maintainers.

## Getting Started

### 1. Fork and Clone

```bash
# Fork the repository on GitHub
git clone https://github.com/kabudu/groklang.git
cd groklang
git remote add upstream https://github.com/ORIGINAL-OWNER/groklang.git
```

### 2. Create a Branch

Always create a new branch for your work:

```bash
git checkout -b feature/your-feature-name
# or for bug fixes:
git checkout -b fix/issue-description
```

Use descriptive branch names:

- `feature/lexer-improvements`
- `fix/type-inference-bug`
- `docs/add-examples`
- `test/add-memory-tests`

### 3. Set Up Development Environment

```bash
# Create virtual environment
python -m venv venv
source venv/bin/activate  # or: venv\Scripts\activate on Windows

# Install dependencies
pip install -r requirements-dev.txt

# Install pre-commit hooks (optional but recommended)
pre-commit install
```

## Making Changes

### Coding Standards

- **Python Style**: Follow [PEP 8](https://www.python.org/dev/peps/pep-0008/)
- **Type Hints**: Add type hints to all functions (`def func(x: int) -> str:`)
- **Docstrings**: Use Google-style docstrings for all public functions and classes
- **Line Length**: Keep lines to 100 characters maximum
- **Imports**: Use `isort` to organize imports

Example:

```python
def calculate_type_constraints(expr: Expression, env: TypeEnv) -> List[Constraint]:
    """Generate type constraints from an expression.

    Args:
        expr: The expression to analyze.
        env: The type environment with available bindings.

    Returns:
        A list of type constraints that must be satisfied.

    Raises:
        TypeError: If the expression contains undefined variables.
    """
    # Implementation here
    pass
```

### Commit Messages

Write clear, descriptive commit messages:

```
[Feature] Add bidirectional type inference

- Implement constraint generation algorithm
- Add unification with occurs check
- Support polymorphic type inference

Closes #123
```

**Format**:

- Start with a tag: `[Feature]`, `[Fix]`, `[Docs]`, `[Test]`, `[Refactor]`
- First line: 50 characters or less
- Blank line
- Detailed explanation (wrap at 72 characters)
- Reference issues: `Closes #123`, `Fixes #456`

Tags:

- `[Feature]` - New feature implementation
- `[Fix]` - Bug fix
- `[Docs]` - Documentation changes
- `[Test]` - Adding/improving tests
- `[Refactor]` - Code refactoring without behavior change
- `[Perf]` - Performance improvements

### Testing

All contributions must include tests. Run tests before submitting:

```bash
# Run all tests
pytest tests/ -v

# Run specific test file
pytest tests/test_lexer.py -v

# Run with coverage
pytest --cov=groklang tests/

# Run specific test
pytest tests/test_lexer.py::test_tokenize_numbers -v
```

**Test Requirements**:

- Write tests for new features
- Update tests for modified behavior
- Maintain >80% code coverage
- All tests must pass before PR submission

Test file naming:

- `tests/test_COMPONENT.py` for unit tests
- `tests/integration/test_FEATURE.py` for integration tests

### Documentation

Update documentation for changes:

1. **Code Comments**: Explain the _why_, not the _what_
2. **Docstrings**: Add/update for all public APIs
3. **README**: Update if user-facing behavior changes
4. **Spec Documents**: Update relevant specification if design changes

## Submission Process

### 1. Update Your Branch

```bash
git fetch upstream
git rebase upstream/main
```

### 2. Push to Your Fork

```bash
git push origin feature/your-feature-name
```

### 3. Create a Pull Request

On GitHub:

- **Title**: Clear, concise description
- **Description**: Fill out the PR template completely
- **Link Issues**: Reference any related issues
- **Specify Type**: Mark as Feature, Fix, Docs, Test, or Refactor

PR template:

```markdown
## Description

Brief description of changes.

## Type

- [ ] Feature
- [ ] Bug Fix
- [ ] Documentation
- [ ] Test
- [ ] Refactor

## Related Issues

Closes #123

## Changes

- List of specific changes
- Made by this PR

## Testing

- [ ] Tests added/updated
- [ ] All tests passing
- [ ] Coverage maintained

## Breaking Changes

- [ ] No breaking changes
- [ ] Breaking change (describe)

## Checklist

- [ ] Code follows style guidelines
- [ ] Docstrings added/updated
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] No new warnings generated
```

### 4. Address Review Comments

When reviewers request changes:

```bash
# Make the changes
git add .
git commit -m "[Review] Address feedback on X

- Fixed issue mentioned by reviewer
"
git push origin feature/your-feature-name
```

Don't force-push unless requested. Reviewers will see the new commits.

### 5. Merge

Once approved:

- Squash small fixes: Keep history clean
- Rebase if requested: Keep linear history
- Delete branch after merge

## Contribution Areas

### High Priority

- **Phase 1**: Lexer/Parser implementation
- **Phase 2**: Type inference engine
- **Tests**: Any test coverage improvements
- **Docs**: Clarifications and examples
- **Bugs**: Any reported issues

### Ways to Contribute

- **Code**: Implement features, fix bugs
- **Tests**: Improve test coverage and add edge cases
- **Documentation**: Improve guides, add examples, fix typos
- **Issues**: Report bugs or suggest features
- **Review**: Help review pull requests
- **Discussions**: Answer questions in GitHub Discussions

## Communication

### Before Starting

For substantial changes:

1. **Check Issues**: Look for existing discussions
2. **Open Issue**: Describe your idea
3. **Get Feedback**: Get approval before implementation
4. **Discuss Approach**: Ensure alignment with project goals

This prevents duplicate work and ensures your contribution aligns with the project direction.

### Discussions

- **Questions**: Use GitHub Discussions
- **Bugs**: File Issues with reproducible examples
- **Features**: Open an issue for discussion first
- **Design**: Discuss in issue before implementation

## Review Process

### Timeline

- **Acknowledgment**: Within 48 hours
- **Initial Review**: Within 1 week
- **Feedback**: Ongoing as needed
- **Merge**: Once approved and tests pass

### What Reviewers Check

1. ✅ **Code Quality**: Follows standards, well-structured
2. ✅ **Tests**: Adequate coverage, all passing
3. ✅ **Documentation**: Clear and complete
4. ✅ **Specification Compliance**: Matches requirements
5. ✅ **Backward Compatibility**: No breaking changes (unless necessary)
6. ✅ **Performance**: No significant regressions

### Common Feedback

- **"Add tests"**: Include tests for edge cases
- **"Update docs"**: Update docstrings and relevant docs
- **"Refactor"**: Simplify or improve code clarity
- **"Add type hints"**: Full type annotation required
- **"Rebase on main"**: Update your branch with latest changes

## Recognition

We recognize contributors in multiple ways:

### In Code

- All commits are attributed to their author
- Pull request #XXXX appears in commit message

### In Project

- Added to [CONTRIBUTORS.md](#contributors) file
- Mentioned in release notes for significant contributions
- Listed in project documentation

### How We Recognize

- **Minor fixes**: Issue/PR acknowledgment
- **Features**: Added to CONTRIBUTORS.md
- **Major contributions**: Considered for maintainer status

## Development Phases

Contributions map to development phases:

| Phase       | Timeline    | Focus           | Contributors            |
| ----------- | ----------- | --------------- | ----------------------- |
| **Phase 1** | Weeks 1-3   | Lexer/Parser    | Compiler experts        |
| **Phase 2** | Weeks 4-8   | Type System     | Type theory experts     |
| **Phase 3** | Weeks 9-12  | Code Generation | Backend developers      |
| **Phase 4** | Weeks 13-17 | Runtime         | Runtime/systems experts |
| **Phase 5** | Weeks 18-20 | FFI/AI          | Systems/Python experts  |

**For Phase 1 contributors**: See [Phase-1-Lexer-Parser.md](docs/Implementation/Phase-1-Lexer-Parser.md)

## Tools & Infrastructure

### Required Tools

- **Python 3.9+**: Language implementation
- **Git**: Version control
- **pytest**: Testing framework
- **GitHub**: Hosting and PRs

### Optional Tools

- **pre-commit**: Automated checks before commits
- **Black**: Code formatter
- **isort**: Import organizer
- **mypy**: Type checker
- **pylint**: Linter

### Setting Up Tools

```bash
# Install optional tools
pip install black isort mypy pylint pre-commit

# Run formatters
black src/ tests/
isort src/ tests/

# Check types
mypy src/

# Lint
pylint src/
```

## Common Questions

### Q: How long does review take?

**A**: Usually 1 week for minor changes, 2-3 weeks for features. Complex changes may take longer.

### Q: What if my PR gets rejected?

**A**: We provide detailed feedback. You can revise and resubmit, or discuss in the issue.

### Q: Can I work on Phase 2 if Phase 1 isn't done?

**A**: Yes! Start with the base framework from Phase 1 docs, or coordinate with other contributors.

### Q: How do I report security issues?

**A**: Email security@groklang.dev (or contact maintainers privately). Do NOT open public issues for security vulnerabilities.

### Q: What about intellectual property?

**A**: By contributing, you agree that your contributions are licensed under the MIT License (see [LICENSE](LICENSE)).

## Resources

- **Implementation Guides**: [docs/Implementation/](docs/Implementation/)
- **Specifications**: [docs/Specifications/](docs/Specifications/)
- **Validation Checklist**: [docs/Validation/Master-Validation-Checklist.md](docs/Validation/Master-Validation-Checklist.md)
- **Type System**: [02-Type-System-Specification.md](docs/Specifications/02-Type-System-Specification.md)
- **Architecture**: [ARCHITECTURAL-DECISIONS.md](ARCHITECTURAL-DECISIONS.md)

## Getting Help

- **Questions**: Open a GitHub Discussion
- **Bugs**: File an Issue with reproducible example
- **Stuck**: Ask in PR/Issue or discussion
- **Ideas**: Start a discussion before implementation

---

## Contributor Recognition

### Project Maintainers

<!-- This section will be updated as contributors join -->

- **Project Lead**: [To be assigned]
- **Phase Leads**: [To be assigned]

### Core Contributors

<!-- Contributors will be listed here -->

### Community Contributors

All contributors are valued! First-time contributions are especially welcome.

---

## License

By contributing to GrokLang, you agree that your contributions will be licensed under the [MIT License](LICENSE).

---

**Thank you for contributing to GrokLang! 🚀**

Your contributions help make GrokLang a better language for everyone.

For questions or suggestions about this guide, please open an issue or discussion.
