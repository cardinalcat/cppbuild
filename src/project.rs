use serde_derive::{Deserialize, Serialize};
use std::fs::{create_dir, File};
use std::io::Read;
use std::path::Path;
#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    package: Package,
    owners: Vec<Owner>,
    pub dependency: Option<Vec<Dependency>>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Package {
    name: String,
    version: String,
    standard: Option<String>,
    project_type: Option<String>,
    url: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
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
                None,
            ),
            owners: own,
            dependency: None,
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
            Ok(len) => (),
            Err(e) => return Err(e),
        }
        //println!("content: {}", content);
        Ok(toml::from_str(content.as_str()).unwrap())
    }
}

impl Package {
    pub fn new(
        name: String,
        version: String,
        standard: Option<String>,
        project_type: Option<String>,
        url: Option<String>,
    ) -> Self {
        Self {
            name,
            version,
            standard: Some("c++17".to_string()),
            project_type,
            url,
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
