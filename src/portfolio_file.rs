/// List all files in ./portfolios dir
pub fn list() {
    let paths = std::fs::read_dir("./portfolios").unwrap();
    for path in paths {
        println!("{}", path.unwrap().file_name().to_string_lossy());
    }
}

pub fn new(name: &str) {
    let path = format!("./portfolios/{}", name);
    // v1: if name exists, it will overwrite the existing file
    let _ = std::fs::File::create(path).unwrap();
    // v2: TODO if exists, propt for action or check for --force cli param flat
}
