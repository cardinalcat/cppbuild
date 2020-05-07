use serde_derive::{Deserialize, Serialize};
use std::fs::{create_dir, File};
use std::io::Read;
use std::path::Path;
use clang::*;
#[derive(Debug)]
pub struct Project{
    package: Package,
    examples: Option<Vec<Example>>,
    tests: Option<Test>
}
//thinking of moving deps, and owners to package then putting examples and tests in Project
#[derive(Debug, Serialize, Deserialize)]
pub struct Package {
    name: String,
    version: String,
    standard: Option<String>,
    project_type: Option<String>,
    url: Option<String>,
    owners: Vec<Owner>,
    pub dependency: Option<Vec<Dependency>>,
}
#[derive(Debug)]
pub struct Example{
    name: String,
    exec_path: String,
}
impl Example{
    pub fn new(name: String, exec_path: String) -> Self{
        Self {name, exec_path}
    }
}
#[derive(Debug, Clone)]
pub struct Item{
    name: String,
    comment: Option<String>,
}
impl Item{
    pub fn new(name: String, comment: Option<String>) -> Self{
        Self {name, comment }
    }
}
#[derive(Debug)]
pub struct Test{
    name: String,
    entities: Vec<Item>,
}
impl Test{
    pub fn from_file(path: &str) -> Self{
        let clang = Clang::new().unwrap();
        let index = Index::new(&clang, true, false);
        let tu = index.parser(path).parse().unwrap();
        let functions = tu.get_entity().get_children().into_iter().filter(|e| {
            e.get_kind() == EntityKind::FunctionDecl
        }).collect::<Vec<_>>();
        for item in tu.get_entity().get_children().into_iter(){
            println!("child: {:?}", item);
        }
        let mut funcs = Vec::with_capacity(functions.len());
        for func in functions.iter(){
            //println!("func: {:?}", func);
            funcs.push( Item::new( func.get_display_name().unwrap(), func.get_comment() ) );
        }
        Self {name: path.to_string(), entities: funcs}
    }
    pub fn get_entities(&self) -> Vec<Item>{
        self.entities.clone()
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
                Some("c++17".to_string()),
                project_type,
                own,
                None,
            ),
            examples: None,
            tests: None,
        }
    }
    pub fn get_package(self) -> Package{
        self.package
    }
    pub fn get_dependencies(&self) -> Option<Vec<Dependency>>{
        match &self.package.dependency{
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
        println!("content: {}", content);
        //println!("content: {}", content);
        Ok(Self { package: toml::from_str(content.as_str()).unwrap(), examples: None, tests: None})
    }
}

impl Package {
    pub fn new(
        name: String,
        version: String,
        _standard: Option<String>,
        project_type: Option<String>,
        owners: Vec<Owner>,
        url: Option<String>,
    ) -> Self {
        Self {
            name,
            version,
            standard: Some("c++17".to_string()),
            project_type,
            url,
            owners,
            dependency: None,
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
    fn ne(&self, other: &Self) -> bool {
        if self.version == other.version && self.name == other.name {
            return false;
        }
        true
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
