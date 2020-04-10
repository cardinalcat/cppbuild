use serde_derive::{Deserialize, Serialize};
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
    pub fn new(name: String) -> Self {
        let mut own = Vec::new();
        own.push(Owner::new(whoami::username()));
        let mut dep = Vec::new();
        dep.push(Dependency::new(
            "opencv4".to_string(),
            "4.3.0".to_string(),
            None,
        ));
        Self {
            package: Package::new(name),
            owners: own,
            dependency: None,
        }
    }
}

impl Package {
    pub fn new(name: String) -> Self {
        Self {
            name,
            version: "0.1.0".to_string(),
            standard: Some("c++17".to_string()),
        }
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
