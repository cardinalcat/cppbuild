//extern crate argparse;
#[macro_use]
extern crate lazy_static;
extern crate toml;
mod compiling;
mod project;
mod upstream;
use compiling::Program;
use project::Project;
use std::fs::create_dir;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use upstream::*;

pub fn print_help(code: i32) -> ! {
    println!(
        "\tUSAGE:
        new \t <name> \t creates new project.
        -h, --help \t\t displays this message.
        --get-flags \t\t prints flags sent to g++.
        fetch name version \t downloads the package and saves it to ~/.cppbuild
        publish \t\t generates a tar ball with the source code of the project"
    );
    println!("error code: {}", code);
    std::process::exit(code);
}
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    //let mut project: Option<Arc<Mutex<Project>>> = None;
    let mut index = 0;
    while index < args.len() {
        let arg = args.get(index).unwrap().as_str();
        match arg {
            "-h" | "--help" => {
                print_help(0);
            }
            "--get-flags" => {
                println!(
                    "{}",
                    Program::new(&Project::from_file(".").unwrap(), ".").get_flags()
                );
            }
            "new" => {
                index = index + 1;
                let project_name = match args.get(index) {
                    Some(arg) => arg,
                    None => print_help(1),
                };
                match create_dir(project_name) {
                    Ok(()) => (),
                    _ => print_help(3),
                }
                match create_dir(format!("{}/{}", project_name, "src")) {
                    Ok(()) => (),
                    _ => print_help(4),
                }
                match create_dir(format!("{}/{}", project_name, "headers")) {
                    Ok(()) => (),
                    _ => print_help(5),
                }
                let mut file = match File::create(format!("{}/{}", project_name, "build.toml")) {
                    Ok(f) => f,
                    _ => panic!("couldn't create initial toml file"),
                };

                let project = Project::new(project_name.clone(), Some("bin".to_string()), None);
                file.write_all(&toml::to_string(&project.get_package()).unwrap().into_bytes())
                    .unwrap();
            }
            "build" => {
                Program::new(&Project::from_file(".").unwrap(), ".")
                    .build(".")
                    .expect("unable to build the program due to an io error");
            }
            "upload" => {
                index = index + 1;
                let project_name = match args.get(index) {
                    Some(arg) => arg,
                    None => print_help(1),
                };
                generate_package(PackageType::PkgConfig, "opencv4").unwrap();
            }
            "publish" => {
                generate_package(PackageType::CppBuild, ".").unwrap();
            }
            "fetch" => {
                index = index + 1;
                let project_name = match args.get(index) {
                    Some(arg) => arg,
                    None => print_help(6),
                };
                index = index + 1;
                let project_version = match args.get(index) {
                    Some(arg) => arg,
                    None => print_help(7),
                };
                download_packages(&vec![(project_name.to_owned(), project_version.to_owned())]).unwrap();
            }
            _ => print_help(2),
        }
        index = index + 1;
    }
}
