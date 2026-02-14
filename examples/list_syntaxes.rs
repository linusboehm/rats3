use syntect::parsing::SyntaxSet;

fn main() {
    let syntax_set = SyntaxSet::load_defaults_newlines();

    println!("Searching for CSV syntax...");
    
    // Check by extension
    if let Some(syntax) = syntax_set.find_syntax_by_extension("csv") {
        println!("✓ Found CSV syntax by extension: {}", syntax.name);
    } else {
        println!("✗ No CSV syntax found by extension");
    }
    
    // List all syntaxes that might be related
    println!("\nAll available syntaxes with 'csv', 'comma', or 'table':");
    for syntax in syntax_set.syntaxes() {
        let name_lower = syntax.name.to_lowercase();
        if name_lower.contains("csv") || name_lower.contains("comma") || name_lower.contains("table") {
            println!("  - {} (file types: {:?})", syntax.name, syntax.file_extensions);
        }
    }
}
