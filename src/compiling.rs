use crate::project::Project;
use std::cell::RefCell;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::time::SystemTime;
use walkdir::WalkDir;
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum BuildMode {
    Debug,
    Release,
    Normal,
    Example,
    Test,
}
impl BuildMode {
    pub fn is_normal(&self) -> bool {
        self == &BuildMode::Debug || self == &BuildMode::Release || self == &BuildMode::Normal
    }
}
pub struct Program {
    name: String,
    sources: Box<RefCell<Vec<String>>>,
    dependencies: Vec<String>,
    include: Vec<String>,
    program_type: String,
    standard: String,
}
impl Program {
    ///responsible for building the program, outputs file into target. if it is a binary it is saved to target/main otherwise target/main.o
    ///path specifies the root of the project so the project can be build from other directories
    pub fn build(
        &mut self,
        path: &str,
        mode: BuildMode,
    ) -> std::result::Result<(), std::io::Error> {
        let mut extra_args = Vec::new();
        if mode == BuildMode::Release {
            extra_args.push("-O3".to_string());
        } else if mode == BuildMode::Debug {
            extra_args.push("-g".to_string());
        } //this will be used to link the .o file from this project
        else if mode == BuildMode::Example {
            extra_args.push(format!("{}/target/lib{}.so", path, self.name));
            self.program_type = "bin".to_string();
            //shoud check if object file exists if not build it
        }
        if self.program_type == "lib" {
            extra_args.push("-c".to_string());
            extra_args.push("-o".to_string());
            extra_args.push(format!("{}/target/lib{}.so", path, self.name));
        } else {
            extra_args.push("-o".to_string());
            extra_args.push(format!("{}/target/{}", path, self.name));
        }
        if self.standard == "nostd"{
            extra_args.push("-nostd".to_string());
        }else{
            extra_args.push(format!("-std={}", self.standard.clone()));
        }
        extra_args.push("-static".to_string());
        let op = Command::new("g++")
            .args(&["-Iheaders/"])
            .args(extra_args.iter())
            .args(self.sources.get_mut())
            .args(self.dependencies.iter())
            .args(self.include.iter())
            .output()?;
        std::io::stdout().write_all(&op.stdout)?;
        std::io::stderr().write_all(&op.stderr)?;
        if mode == BuildMode::Example{
            self.program_type = "lib".to_string();
        }
        Ok(())
    }
    ///runs the program, checking first to see if any source code has been updated since last build
    pub fn run(&mut self, path: &str, mode: BuildMode) -> std::result::Result<(), std::io::Error> {
        if self.program_type != "bin"
            && (mode == BuildMode::Normal || mode == BuildMode::Release || mode == BuildMode::Debug)
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "not an executable file",
            ));
        }
        if self.program_type == "bin" && mode == BuildMode::Example {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "binary projects are not allowed to have examples",
            ));
        }
        if self.program_type == "bin" && mode == BuildMode::Test {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "binary projects are not allowed to have tests",
            ));
        }
        let pa = if self.program_type == "bin" {
            format!("{}/target/{}", path, self.name)
        } else {
            format!("{}/target/{}.o", path, self.name)
        };
        let p = Path::new(pa.as_str());
        if !p.exists() {
            self.build(path, mode)?;
        }
        if last_modified(path, std::fs::metadata(p)?.modified()?)? || mode == BuildMode::Release {
            self.build(path, mode)?;
        }
        if mode != BuildMode::Debug {
            let mut command = Command::new(pa.as_str());
            if let Ok(mut child) = command.spawn() {
                child.wait().expect("command wasn't running");
            } else {
                println!("command didn't start");
            }
        } else {
            let mut command = Command::new("gdb");
            if let Ok(mut child) = command
                .arg(pa.as_str())
                .spawn()
            {
                child.wait().expect("command wasn't running");
            } else {
                println!("command didn't start");
            }
        }
        Ok(())
    }
    fn create(project: &Project, path: &str, mode: BuildMode, file: Option<&str>) -> Self {
        let sources; // = Box::new(RefCell::new(Vec::new()));
        let mut dependencies = Vec::new();
        let mut include = Vec::new();
        match &project.get_dependencies() {
            Some(dep) => {
                for depend in dep.iter() {
                    let mut config = pkg_config::Config::new();
                    if depend.get_version() != "*" {
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
            }
            None => (),
        }
        if mode.is_normal() {
            let walk: RefCell<walkdir::IntoIter> =
                RefCell::new(WalkDir::new(format!("{}/src", path).as_str()).into_iter());
            let build: RefCell<Vec<String>> = RefCell::new(Vec::new());
            sources = add_file(walk, build);
        } else {
            sources = Box::new(RefCell::new(vec![format!("{}/{}", path, file.unwrap())]));
        }
        Self {
            name: project.get_package().get_name(),
            sources,
            dependencies,
            include,
            program_type: project.get_type(),
            standard: project.get_standard(),
        }
    }
    pub fn example(project: &Project, path: &str, file: &str) -> Self {
        Self::create(project, path, BuildMode::Example, Some(file))
    }
    pub fn test(project: &Project, path: &str, file: &str) -> Self {
        Self::create(project, path, BuildMode::Test, Some(file))
    }
    ///creates the Program instance based on a project instance
    pub fn new(project: &Project, path: &str) -> Program {
        Self::create(project, path, BuildMode::Normal, None)
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
pub fn last_modified(path: &str, time: SystemTime) -> std::result::Result<bool, std::io::Error> {
    for file in WalkDir::new(path).into_iter() {
        let file = file?;
        let metadata = std::fs::metadata(file.path()).unwrap();
        if metadata.modified().unwrap() > time {
            return Ok(true);
        }
    }
    Ok(false)
}
