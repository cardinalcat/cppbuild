use cppbuild::upstream::download_packages;

fn main() {
    let packages = vec![
        ("opencv".to_string(), "4.3.0".to_string()),
        ("libpng".to_string(), "0.1.0".to_string()),
    ];
    download_packages(&packages);
}
