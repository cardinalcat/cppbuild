use crate::project::Project;
use std::cell::RefCell;
use std::io::Write;
use std::process::Command;
use walkdir::WalkDir;
pub struct Program {
    sources: Box<RefCell<Vec<String>>>,
    dependencies: Box<Vec<String>>,
    include: Box<Vec<String>>,
}
impl Program {
    pub fn build(&mut self, path: &str) -> std::result::Result<(), std::io::Error> {
        let op = match Command::new("g++")
            .args(&["-o", format!("{}/target/main", path).as_str(), "-Iheaders/"])
            .args(self.sources.get_mut())
            .args(self.dependencies.iter())
            .args(self.include.iter())
            .output()
        {
            Ok(out) => out.stdout,
            Err(e) => return Err(e),
        };
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        match handle.write_all(&op) {
            Ok(()) => (),
            Err(e) => return Err(e),
        }
        Ok(())
    }
    pub fn new(project: &Project, path: &str) -> Program {
        let sources; // = Box::new(RefCell::new(Vec::new()));
        let mut dependencies = Box::new(Vec::new());
        let mut include = Box::new(Vec::new());
        match &project.dependency {
            Some(dep) => {
                for depend in dep.iter() {
                    let mut config = pkg_config::Config::new();
                    if depend.get_version() != "*".to_string() {
                        config.exactly_version(&depend.get_version());
                    }
                    //println!(depend.get_version());
                    match config.probe(depend.get_name().as_str()) {
                        Ok(lib) => {
                            for l in lib.libs {
                                let li = format!("{}{}", "-l", l);
                                //println!("li: {}", li);
                                dependencies.push(li);
                            }
                            for i in lib.include_paths {
                                let inc = format!(
                                    "{}{}",
                                    "-I",
                                    i.into_os_string().into_string().unwrap()
                                );
                                include.push(inc);
                            }
                            // add all includes to one vector of args
                        }
                        Err(e) => println!("error: {}", e),
                    }
                }
                let walk: RefCell<walkdir::IntoIter> =
                    RefCell::new(WalkDir::new(format!("{}/src", path).as_str()).into_iter());
                let build: RefCell<Vec<String>> = RefCell::new(Vec::new());
                sources = add_file(walk, build);
            }
            None => {
                let walk: RefCell<walkdir::IntoIter> =
                    RefCell::new(WalkDir::new("src").into_iter());
                let build: RefCell<Vec<String>> = RefCell::new(Vec::new());
                sources = add_file(walk, build);
            }
        }
        Self {
            sources,
            dependencies,
            include,
        }
    }
    pub fn get_flags(&mut self) -> String {
        format!(
            "include paths: {:?}\n libraries: {:?}\n source files: {:?}",
            self.include,
            self.dependencies,
            self.sources.get_mut()
        )
    }
}
pub fn add_file<'a>(
    mut walk: RefCell<walkdir::IntoIter>,
    mut build: RefCell<Vec<String>>,
) -> Box<RefCell<Vec<String>>> {
    let temppath = walk.get_mut().next().unwrap().unwrap();
    let path = temppath.path();
    if path.is_file() {
        build.get_mut().push(path.to_str().unwrap().to_string());
        return Box::new(build);
    }
    add_file(walk, build)
}
