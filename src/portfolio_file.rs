use std::fs;

pub fn list() {
    let paths = fs::read_dir("./portfolios").unwrap();
    for path in paths {
        println!("{}", path.unwrap().file_name().to_string_lossy());
    }
}

pub fn new(name: &str) {
    let path = format!("./portfolios/{}", name);
    // TODO check if path exists and what to do in that case?
    // should we accpet --force argument to overwrite?
    // currently: if name exists, it will overwrite the existing file
    let _ = fs::File::create(path).unwrap();
    // TODO list full dir after completed succ?
}
