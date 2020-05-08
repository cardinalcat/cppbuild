use crate::project::Project;
use std::cell::RefCell;
use std::io::Write;
use std::process::Command;
use std::time::SystemTime;
use walkdir::WalkDir;
pub struct Program {
    sources: Box<RefCell<Vec<String>>>,
    dependencies: Vec<String>,
    include: Vec<String>,
    program_type: String,
    standard: String,
}
impl Program {
    ///responsible for building the program, outputs file into target. if it is a binary it is saved to target/main otherwise target/main.o
    ///path specifies the root of the project so the project can be build from other directories
    pub fn build(&mut self, path: &str) -> std::result::Result<(), std::io::Error> {
        let mut extra_args = Vec::new();
        if self.program_type == "lib"{
            extra_args.push("-c".to_string());
            extra_args.push("-o".to_string());
            extra_args.push(format!("{}/target/lib.o", path));
        }else{
            extra_args.push("-o".to_string());
            extra_args.push(format!("{}/target/main", path));
        }
        extra_args.push(format!("-std={}", self.standard.clone()));
        let op = Command::new("g++")
            .args(&["-Iheaders/"])
            .args(extra_args.iter())
            .args(self.sources.get_mut())
            .args(self.dependencies.iter())
            .args(self.include.iter()).output()?;
        std::io::stdout().write_all(&op.stdout)?;
        std::io::stderr().write_all(&op.stderr)?;
        Ok(())
    }
    ///runs the program, checking first to see if any source code has been updated since last build
    pub fn run(&mut self, path: &str) -> std::result::Result<(), std::io::Error>{
        if self.program_type != "bin"{
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "not an executable file"));
        }
        if last_modified(path, std::fs::metadata(format!("{}/target/main", path).as_str())?.modified()?)?{
            self.build(path)?;
        }
        let mut command = Command::new(format!("{}/target/main", path).as_str());
        if let Ok(mut child) = command.spawn() {
            child.wait().expect("command wasn't running");
        } else {
            println!("command didn't start");
        }
        Ok(())
    }
    ///creates the Program instance based on a project instance
    pub fn new(project: &Project, path: &str) -> Program {
        let sources; // = Box::new(RefCell::new(Vec::new()));
        let mut dependencies = Vec::new();
        let mut include = Vec::new();
        match &project.get_dependencies() {
            Some(dep) => {
                for depend in dep.iter() {
                    let mut config = pkg_config::Config::new();
                    if depend.get_version() != "*"{
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
            program_type: project.get_type(),
            standard: project.get_standard(),
        }
    }
    ///lists flags passed to g++ except for special flags
    pub fn get_flags(&mut self) -> String {
        format!(
            "include paths: {:?}\n libraries: {:?}\n source files: {:?}",
            self.include,
            self.dependencies,
            self.sources.get_mut()
        )
    }
}
///adds files to the Program struct so they can be bassed to g++
pub fn add_file(
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
///checks if any file in a given path has been modified more recently then the given SystemTime passed in
pub fn last_modified(path: &str, time: SystemTime) -> std::result::Result<bool, std::io::Error>{
    for file in WalkDir::new(path).into_iter(){
        let file = file?;
        let metadata = std::fs::metadata(file.path()).unwrap();
        if metadata.modified().unwrap() > time{
            return Ok(true);
        }
    }
    Ok(false)
}