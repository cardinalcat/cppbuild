//extern crate argparse;
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
        --get-flags \t\t prints flags sent to g++."
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
                    Program::new(&Project::from_file(".").unwrap()).get_flags()
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
                file.write_all(&toml::to_string(&project).unwrap().into_bytes())
                    .unwrap();
            }
            "build" => {
                Program::new(&Project::from_file(".").unwrap())
                    .build()
                    .expect("unable to build the program due to an io error");
            }
            "upload" => {
                index = index + 1;
                let project_name = match args.get(index) {
                    Some(arg) => arg,
                    None => print_help(1),
                };
                generate_package(PackageType::PkgConfig, "opencv4");
            }
            "publish" => {
                generate_package(PackageType::CppBuild, ".");
            }
            _ => print_help(2),
        }
        index = index + 1;
    }
}
/*pub fn create_program() -> Program {
    let mut file = match File::open("build.toml") {
        Ok(file) => file,
        Err(e) => panic!("error: {:?}", e),
    };
    if !Path::new("target/").exists() {
        match create_dir("target") {
            Ok(()) => (),
            Err(e) => println!("error: {}", e),
        }
    }
    //crate the project
    //then create the program
    //then build the program
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("unable to read toml file");
    //println!("content: {}", content);
    let pro = toml::from_str(content.as_str()).unwrap();
    Program::new(pro)
    // handle dependencies and everything later
}*/
