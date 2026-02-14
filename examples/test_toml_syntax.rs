use syntect::parsing::{SyntaxSet, SyntaxSetBuilder, SyntaxDefinition};

fn main() {
    // Load TOML syntax
    const TOML_SYNTAX: &str = include_str!("../themes/TOML.tmLanguage");
    
    match SyntaxDefinition::load_from_str(TOML_SYNTAX, true, None) {
        Ok(syntax) => {
            println!("✓ TOML syntax loaded successfully!");
            println!("  Name: {}", syntax.name);
            println!("  File extensions: {:?}", syntax.file_extensions);
            println!("  Scope: {:?}", syntax.scope);
        }
        Err(e) => {
            println!("✗ Failed to load TOML syntax: {}", e);
        }
    }
    
    // Try to build a syntax set with defaults + TOML
    let mut builder = SyntaxSetBuilder::new();
    builder.add_plain_text_syntax();
    
    if let Ok(toml) = SyntaxDefinition::load_from_str(TOML_SYNTAX, true, None) {
        builder.add(toml);
    }
    
    let set = builder.build();
    println!("\nSyntaxes in custom set: {}", set.syntaxes().len());
    
    if let Some(syntax) = set.find_syntax_by_extension("toml") {
        println!("✓ Can find TOML syntax by extension in custom set!");
        println!("  Name: {}", syntax.name);
    } else {
        println!("✗ Cannot find TOML syntax by extension");
    }
}
