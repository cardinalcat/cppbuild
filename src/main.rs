//extern crate argparse;
#[macro_use]
extern crate lazy_static;
extern crate toml;
pub mod compiling;
pub mod project;
pub mod upstream;
pub mod arguments;
use compiling::Program;
use project::Project;
use std::fs::create_dir;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use project::Test;
use upstream::*;
use arguments::Arguments;

pub fn print_help(code: i32) -> ! {
    println!(
        "USAGE:
        --new \t <name> \t creates new project.
        --help \t\t displays this message.
        --get-flags \t\t prints flags sent to g++.
        --fetch name version \t downloads the package and saves it to ~/.cppbuild
        --publish \t\t generates a tar ball with the source code of the project
        --build \t\t builds project and outputs it in target/"
    );
    println!("error code: {}", code);
    std::process::exit(code);
}
fn main() {
    let mut args = Arguments::new();
    args.invoke_callback("--help", &move |args, flags|{
        print_help(0);
    });
    args.invoke_callback("--build", &move |flags, args|{
        Program::new(&Project::from_file(".").unwrap(), ".")
            .build(".")
            .expect("unable to build the program due to an io error");
    });
    args.invoke_callback("--flags", &move |flags, args|{
        println!("flags: {:?}", flags);
        println!("args other flags: {:?}", args.get_flags());
    });
    args.invoke_callback("--new", &move |flags, args|{
        println!("args: {:?} ", args.get_flags());
        let project_type = if args.has_arg("--lib"){
            "lib".to_string()
        }else{
            "bin".to_string()
        };
        println!("flags: {:?}", flags);
        let project_name = match flags.get(0) {
            Some(arg) => arg.get_name(),
            None => print_help(1),
        };
        match create_dir(&project_name) {
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
        println!("project_type: {}", project_type);
        let project = Project::new(project_name.clone(), Some(project_type), None);
        file.write_all(&toml::to_string(&project.get_package()).unwrap().into_bytes())
            .unwrap();
    });
    args.invoke_callback("--publish", &move |flags, _| {
        match flags.get(0){
            Some(f) => { 
                let buildtype = match flags.get(1){
                    Some(kind) => {
                        PackageType::PkgConfig
                        //PackageType::form_string(&kind.get_name());
                    },
                    None => PackageType::CppBuild,
                };
                generate_package(buildtype, f.get_name().as_str()).unwrap()
            },
            None => generate_package(PackageType::CppBuild, ".").unwrap(),
        }
    });
    args.invoke_callback("--get-flags", &move |_, _| {
        println!(
            "{}",
            Program::new(&Project::from_file(".").unwrap(), ".").get_flags()
        );
    });
    args.invoke_callback("--fetch", &move |vals, args|{
        let project_name = match vals.get(0) {
            Some(arg) => arg.get_name(),
            None => print_help(6),
        };
        let project_version = match vals.get(1) {
            Some(arg) => arg.get_name(),
            None => print_help(7),
        };
        download_packages(&vec![(project_name.to_owned(), project_version.to_owned())]).unwrap();
    });
    args.parse();
}
pub fn create_test(){
    println!("create test called");
    let test = Test::from_file("tests/tests.cpp");
    for t in test.get_entities().iter(){
        //println!("test function: {:?}", t);
    }
}