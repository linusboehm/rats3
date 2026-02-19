#!/bin/bash
# Coverage test script for rats3

set -e

echo "Running tests with coverage..."

# Clean previous coverage data
cargo llvm-cov clean

# Run tests with coverage
if [ "$1" == "--html" ]; then
    echo "Generating HTML coverage report..."
    cargo llvm-cov --all-features --html --open
elif [ "$1" == "--lcov" ]; then
    echo "Generating lcov report..."
    cargo llvm-cov --all-features --lcov --output-path lcov.info
elif [ "$1" == "--summary" ]; then
    echo "Generating coverage summary..."
    cargo llvm-cov --all-features
else
    echo "Generating text coverage report..."
    cargo llvm-cov --all-features
    echo ""
    echo "For HTML report, run: ./coverage.sh --html"
    echo "For lcov report, run: ./coverage.sh --lcov"
fi
