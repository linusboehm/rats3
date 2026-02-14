use syntect::highlighting::ThemeSet;

fn main() {
    let theme_set = ThemeSet::load_defaults();

    println!("Available syntect themes:");
    println!("========================");
    let mut themes: Vec<_> = theme_set.themes.keys().collect();
    themes.sort();
    for name in themes {
        println!("  - {}", name);
    }
}
