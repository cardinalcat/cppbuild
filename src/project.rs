use clang::*;
use serde_derive::{Deserialize, Serialize};
use std::fs::{create_dir, File};
use std::io::Read;
use std::path::Path;
use std::io::Write;
use itertools::Itertools;
use walkdir::WalkDir;
//thinking of moving deps, and owners to package then putting examples and tests in Project
#[derive(Debug, Serialize, Deserialize, Clone)]
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
    exec_paths: Vec<String>,
}
impl Example {
    pub fn new(path: &str) -> std::io::Result<Self> {
        let mut exec_paths = Vec::new();
        for file in WalkDir::new(format!("{}/examples", path)){
            exec_paths.push(format!("{}", file?.path().display()));
        }
        Ok( Self { exec_paths } )
    }
    pub fn find(&self, name: &str) -> Option<String>{
        let name = format!("examples/{}.cpp", name);
        if self.exec_paths.contains(&name){
            return Some(name);
        }
        None
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ItemType{
    FunctionDecl,
    UsingDirective,
    InclusionDirective,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Item {
    name: Option<String>,
    comment: Option<String>,
    full_text: String,
    kind: ItemType,
}
impl Item {
    pub fn new(
        name: Option<String>,
        comment: Option<String>,
        path: &str,
        start: usize,
        end: usize,
        kind: ItemType,
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
            kind
        })
    }
    pub fn get_type(&self) -> ItemType{
        self.kind.clone()
    }
    pub fn get_text(&self) -> String{
        self.full_text.clone()
    }
    pub fn get_name(&self) -> Option<String>{
        self.name.clone()
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Test {
    name: String,
    entities: Vec<Item>,
    dir: String,
}
impl Test {
    ///reads from the file given in path and matches doc comments found to see if an identifier can be found, in the binary that is ///test
    pub fn from_file(path: &str, ident: &str) -> std::result::Result<Self, std::io::Error> {
        let project = Project::from_file(path)?;
        let clang = Clang::new().unwrap();
        let index = Index::new(&clang, false, false);
        let mut tests: Vec<std::io::Result<String>> = WalkDir::new(format!("{}/src",path)).into_iter().filter(|e|{
            e.as_ref().unwrap().path().is_file()
        }).map(|e| {
            Ok(format!("{}", e?.path().display()))
        }).collect();
        /*tests.append(&mut WalkDir::new(format!("{}/headers",path)).into_iter().filter(|e|{
            e.as_ref().unwrap().path().is_file()
        }).map(|e|{
            Ok(format!("{}", e?.path().display()))
        }).collect());*/
        tests.append(&mut WalkDir::new(format!("{}/tests",path)).into_iter().filter(|e|{
            e.as_ref().unwrap().path().is_file()
        }).map(|e| {
            Ok(format!("{}", e?.path().display()))
        }).collect());
        let mut funcs = Vec::new();
        for test in tests.iter(){
            let test = match test{
                Ok(t) => t,
                Err(e) => return Err(std::io::Error::new(e.kind(), "error getting test")),
            };
            let tu = index
                .parser(test)
                .detailed_preprocessing_record(true)
                .parse()
                .unwrap();
            let functions = tu
                .get_entity()
                .get_children()
                .into_iter()
                .filter(|e|  { 
                    e.get_kind() == EntityKind::FunctionDecl && e.get_comment() == Some(ident.to_string()) ||
                    e.get_kind() == EntityKind::InclusionDirective && !e.is_in_system_header()
                        || e.get_kind() == EntityKind::UsingDirective && !e.is_in_system_header()
                })
                .collect::<Vec<_>>();
            for func in functions.iter() {
                //println!("func: {:?}", func);
                    //println!("start: {:?}, end: {:?}", func.get_range().unwrap().get_start(), func.get_range().unwrap().get_end());
                    let range = func.get_range().unwrap();
                    funcs.push(Item::new(
                        func.get_display_name(),
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
                        match func.get_kind(){
                            EntityKind::FunctionDecl => ItemType::FunctionDecl,
                            EntityKind::UsingDirective => ItemType::UsingDirective,
                            _ => ItemType::InclusionDirective
                        }
                    )?);
            }
        }
        Ok(Self {
            name: project.get_name(),
            entities: funcs.into_iter().unique().collect(),
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
        let mut file = File::create(format!("{}/target/test_{}.cpp", self.dir, self.name))?;
        let mut open: bool = false;
        let mut funcs = Vec::new();
        for item in self.entities.iter(){
            match item.get_type(){
                ItemType::InclusionDirective => {
                    file.write_all(format!("{}\n", item.get_text()).as_bytes())?
                },
                ItemType::UsingDirective => {
                    file.write_all(format!("{};\n", item.get_text()).as_bytes())?
                },
                ItemType::FunctionDecl => {
                    file.write_all(format!("int {};\n", item.get_name().unwrap()).as_bytes())?;
                    funcs.push(item);
                },
            }
        }
        for func in funcs.iter(){
            if !open{
                open = true;
                file.write_all(b"int main(){\n")?;
            }
            file.write_all(format!("\t{};\n", func.get_name().unwrap()).as_bytes())?;
        }
        file.write_all(b"}\n")?;
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
#[derive(Debug, Serialize, Deserialize, Clone)]
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
    pub fn get_package(&self) -> Package {
        self.package.clone()
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
