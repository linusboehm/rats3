# Testing Guide for rats3

This document describes how to run tests and generate coverage reports for rats3.

## Running Tests

### Run all tests
```bash
cargo test
```

### Run tests with S3 features
```bash
cargo test --features s3
```

### Run tests with output
```bash
cargo test -- --nocapture
```

## Code Coverage

This project uses `cargo-llvm-cov` for code coverage analysis.

### Prerequisites

Install cargo-llvm-cov:
```bash
cargo install cargo-llvm-cov
```

### Generate Coverage Reports

#### Quick Summary (Terminal Output)
```bash
./coverage.sh
# or
cargo llvm-cov --all-features
```

#### HTML Report (Opens in Browser)
```bash
./coverage.sh --html
# or
cargo llvm-cov --all-features --html --open
```

#### LCOV Report (For CI/CD)
```bash
./coverage.sh --lcov
# or
cargo llvm-cov --all-features --lcov --output-path lcov.info
```

### Understanding Coverage Output

The coverage report shows:
- **Lines**: Percentage of code lines executed during tests
- **Functions**: Percentage of functions called during tests
- **Regions**: More granular coverage of code blocks

Look for:
- Red/uncovered lines: Code not tested
- Yellow/partially covered: Code with some paths not tested
- Green/covered: Fully tested code

### CI/CD Integration

For GitHub Actions or other CI systems, add:

```yaml
- name: Install coverage tool
  run: cargo install cargo-llvm-cov

- name: Generate coverage
  run: cargo llvm-cov --all-features --lcov --output-path lcov.info

- name: Upload to codecov
  uses: codecov/codecov-action@v3
  with:
    files: lcov.info
```

## Writing Tests

### Unit Tests
Place unit tests in the same file as the code:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // Test code here
    }
}
```

### Integration Tests
Create files in `tests/` directory:

```rust
// tests/integration_test.rs
use rats3::*;

#[test]
fn test_feature() {
    // Test code here
}
```

## Best Practices

1. **Test public interfaces**: Focus on testing public APIs
2. **Test edge cases**: Include boundary conditions and error cases
3. **Keep tests fast**: Mock external dependencies (S3, filesystem)
4. **Use descriptive names**: Test names should describe what they test
5. **One assertion per test**: When possible, test one thing at a time

## Current Test Status

Check current test coverage with:
```bash
cargo test
./coverage.sh
```

Target coverage goal: 70%+ for critical paths
