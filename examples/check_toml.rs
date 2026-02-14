use syntect::parsing::SyntaxSet;

fn main() {
    let syntax_set = SyntaxSet::load_defaults_newlines();

    println!("Searching for TOML syntax...");
    
    // Check by extension
    if let Some(syntax) = syntax_set.find_syntax_by_extension("toml") {
        println!("✓ Found TOML syntax by extension: {}", syntax.name);
        println!("  File extensions: {:?}", syntax.file_extensions);
    } else {
        println!("✗ No TOML syntax found by extension");
    }
    
    // List all syntaxes that might be related
    println!("\nAll available syntaxes:");
    let mut syntaxes: Vec<_> = syntax_set.syntaxes().iter().collect();
    syntaxes.sort_by_key(|s| &s.name);
    for syntax in syntaxes {
        println!("  - {} (extensions: {:?})", syntax.name, syntax.file_extensions);
    }
}
