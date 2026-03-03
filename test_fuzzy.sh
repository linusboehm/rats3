#!/bin/bash

# Test script to verify fuzzy matching works

cd "$(dirname "$0")"

echo "Testing fuzzy matcher..."

cat > test_fuzzy_main.rs << 'EOF'
use std::path::PathBuf;

fn main() {
    // Test the fuzzy matcher directly
    let entries = vec![
        "main.rs".to_string(),
        "lib.rs".to_string(),
        "mod.rs".to_string(),
        "README.md".to_string(),
        "index.html".to_string(),
        "test.rs".to_string(),
        "nested.txt".to_string(),
    ];

    let mut matcher = rats3::fuzzy::FuzzyMatcher::new();

    println!("Testing empty query:");
    let results = matcher.match_entries(&entries, "");
    println!("  Results: {:?}", results);
    println!("  Count: {}", results.len());

    println!("\nTesting 'rs' query:");
    let results = matcher.match_entries(&entries, "rs");
    println!("  Results: {:?}", results);
    println!("  Matched entries:");
    for &idx in &results {
        println!("    - {}", entries[idx]);
    }

    println!("\nTesting 'md' query:");
    let results = matcher.match_entries(&entries, "md");
    println!("  Results: {:?}", results);
    println!("  Matched entries:");
    for &idx in &results {
        println!("    - {}", entries[idx]);
    }

    println!("\nTesting 'main' query:");
    let results = matcher.match_entries(&entries, "main");
    println!("  Results: {:?}", results);
    println!("  Matched entries:");
    for &idx in &results {
        println!("    - {}", entries[idx]);
    }
}
EOF

# Compile and run the test
rustc --edition 2021 -L target/debug/deps test_fuzzy_main.rs -o test_fuzzy --extern rats3=target/debug/librats3.rlib 2>&1 | tail -10

if [ -f test_fuzzy ]; then
    ./test_fuzzy
    rm test_fuzzy
else
    echo "Failed to compile test"
fi

rm -f test_fuzzy_main.rs
