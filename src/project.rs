use clang::*;
use serde_derive::{Deserialize, Serialize};
use std::fs::{create_dir, File};
use std::io::Read;
use std::path::Path;
use walkdir::WalkDir;
//thinking of moving deps, and owners to package then putting examples and tests in Project
#[derive(Debug, Serialize, Deserialize)]
pub struct Package {
    name: String,
    version: String,
    standard: String,
    project_type: Option<String>,
    repository: Option<String>,
    owners: Vec<Owner>,
    pub dependency: Option<Vec<Dependency>>,
    dev_dependency: Option<Vec<Dependency>>,
    description: Option<String>,
    license: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    name: String,
    exec_path: String,
}
impl Example {
    pub fn new(name: String, exec_path: String) -> Self {
        Self { name, exec_path }
    }
}
/*pub struct Location{
    file: String,
    start: u32,
    end: u32,
}
impl Location{
    pub fn new()
}*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    name: String,
    comment: Option<String>,
    full_text: String,
}
impl Item {
    pub fn new(
        name: String,
        comment: Option<String>,
        path: &str,
        start: usize,
        end: usize,
    ) -> std::result::Result<Self, std::io::Error> {
        let mut file = File::open(path)?;
        let mut content: Vec<u8> = Vec::new();
        match file.read_to_end(&mut content) {
            Ok(_) => (),
            Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        };
        let full_text: String = match String::from_utf8(content[start..end].to_vec()) {
            Ok(text) => text,
            Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        };
        Ok(Self {
            name,
            comment,
            full_text,
        })
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Test {
    name: String,
    entities: Vec<Item>,
    includes: Vec<(String, bool)>,
    dir: String,
}
impl Test {
    ///reads from the file given in path and matches doc comments found to see if an identifier can be found, in the binary that is ///test
    pub fn from_file(path: &str, ident: &str) -> std::result::Result<Self, std::io::Error> {
        let clang = Clang::new().unwrap();
        let index = Index::new(&clang, false, false);
        let tu = index
            .parser(path)
            .detailed_preprocessing_record(true)
            .parse()
            .unwrap();
        let functions = tu
            .get_entity()
            .get_children()
            .into_iter()
            .filter(|e| e.get_kind() == EntityKind::FunctionDecl)
            .collect::<Vec<_>>();
        let incs = tu
            .get_entity()
            .get_children()
            .into_iter()
            .filter(|e| {
                e.get_kind() == EntityKind::InclusionDirective && !e.is_in_system_header()
                    || e.get_kind() == EntityKind::UsingDirective && !e.is_in_system_header()
            })
            .collect::<Vec<_>>();
        let mut includes = Vec::new();
        for inc in incs.iter() {
            includes.push(match inc.get_display_name() {
                Some(name) => {
                    let kind: bool = if inc.get_kind() == EntityKind::UsingDirective {
                        true
                    } else {
                        false
                    };
                    (name, kind)
                }
                None => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "display name for include directive doesn't exist",
                    ))
                }
            });
        }
        /*for item in tu.get_entity().get_children().into_iter(){
            println!("child: {:?}", item);
        }*/
        let mut funcs = Vec::with_capacity(functions.len());
        for func in functions.iter() {
            //println!("func: {:?}", func);
            if func.get_comment() == Some(ident.to_string()) {
                //println!("start: {:?}, end: {:?}", func.get_range().unwrap().get_start(), func.get_range().unwrap().get_end());
                let range = func.get_range().unwrap();
                funcs.push(Item::new(
                    func.get_display_name().unwrap(),
                    func.get_comment(),
                    &format!(
                        "{}",
                        func.get_location()
                            .unwrap()
                            .get_file_location()
                            .file
                            .unwrap()
                            .get_path()
                            .as_path()
                            .display()
                    ),
                    range.get_start().get_file_location().offset as usize,
                    range.get_end().get_file_location().offset as usize,
                )?);
            }
        }
        Ok(Self {
            name: path.to_string(),
            entities: funcs,
            includes,
            dir: path.to_string(),
        })
    }
    ///returns each test function from this Test instance
    pub fn get_entities(&self) -> Vec<Item> {
        self.entities.clone()
    }
    ///this function is responsible for consolidating all test functions in the project
    pub fn append(&mut self, second: &mut Vec<Item>) {
        self.entities.append(second);
    }
    ///this function builds the main function that calls each test function
    pub fn build_main(&self) -> std::result::Result<(), std::io::Error> {
        Ok(())
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dependency {
    name: String,
    version: String,
    url: Option<String>,
}
impl Dependency {
    pub fn new(name: String, version: String, url: Option<String>) -> Self {
        Self { name, version, url }
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    pub fn get_version(&self) -> String {
        self.version.clone()
    }
    pub fn get_url(&self) -> Option<String> {
        self.url.clone()
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Owner {
    name: String,
    email: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    package: Package,
    examples: Option<Vec<Example>>,
    tests: Option<Test>,
}
impl Project {
    pub fn new(name: String, project_type: Option<String>, owners: Option<Vec<Owner>>) -> Self {
        let mut own = Vec::new();
        match owners {
            Some(mut owns) => own.append(&mut owns),
            None => own.push(Owner::new(whoami::username())),
        }
        Self {
            package: Package::new(
                name,
                "0.1.0".to_string(),
                "c++17".to_string(),
                project_type,
                own,
                None,
            ),
            examples: None,
            tests: None,
        }
    }
    pub fn get_package(self) -> Package {
        self.package
    }
    pub fn get_dependencies(&self) -> Option<Vec<Dependency>> {
        match &self.package.dependency {
            Some(dep) => Some(dep.clone()),
            None => None,
        }
    }
    pub fn get_type(&self) -> String {
        self.package.project_type.as_ref().unwrap().clone()
    }
    pub fn get_version(&self) -> String {
        self.package.version.clone()
    }
    pub fn get_name(&self) -> String {
        self.package.name.clone()
    }
    pub fn get_standard(&self) -> String {
        self.package.standard.clone()
    }
    pub fn from_file(path: &str) -> std::io::Result<Self> {
        let mut file = match File::open(format!("{}/{}", path, "build.toml")) {
            Ok(file) => file,
            Err(e) => return Err(e),
        };
        if !Path::new("target/").exists() {
            match create_dir("target") {
                Ok(()) => (),
                Err(e) => return Err(e),
            }
        }
        let mut content = String::new();
        match file.read_to_string(&mut content) {
            Ok(_) => (),
            Err(e) => return Err(e),
        }
        Ok(Self {
            package: toml::from_str(content.as_str()).unwrap(),
            examples: None,
            tests: None,
        })
    }
}

impl Package {
    pub fn new(
        name: String,
        version: String,
        standard: String,
        project_type: Option<String>,
        owners: Vec<Owner>,
        repository: Option<String>,
    ) -> Self {
        Self {
            name,
            version,
            standard,
            project_type,
            repository,
            owners,
            description: None,
            license: None,
            dependency: None,
            dev_dependency: None,
        }
    }
    pub fn get_name(&self) -> String{
        self.name.clone()
    }
    pub fn get_version(&self) -> String{
        self.version.clone()
    }
    pub fn get_description(&self) -> Option<String>{
        match &self.description {
            Some(desc) => Some(desc.clone()),
            None => None
        }
    }
}
impl PartialEq for Package {
    fn eq(&self, other: &Self) -> bool {
        if self.version == other.version && self.name == other.name {
            return true;
        }
        false
    }
}
impl Owner {
    pub fn new(name: String) -> Self {
        Self {
            name,
            email: "unset".to_string(),
        }
    }
}
