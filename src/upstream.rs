use crate::project::Package;
use crate::project::Project;
use libflate::gzip::Encoder;
use std::io::{Read, Write};
use walkdir::WalkDir;

use crate::compiling::Program;
use serde_derive::{Deserialize, Serialize};
use std::fs::{create_dir, File};
use std::io;
use std::io::copy;
use std::path::Path;
use std::sync::Mutex;
use tar::Archive;

lazy_static! {
    #[derive(Debug)]
    pub static ref PROGRAM_DATA: Mutex<String> = {
        let home_dir = match dirs::home_dir(){
            Some(dir) => format!("{}", dir.as_path().display()),
            None => panic!("this program won't work without a homedir"),
        };
        let cppdir = format!("{}/{}", home_dir, ".cppbuild");
        let path = Path::new(cppdir.as_str());
        if !path.exists(){
            match create_dir(cppdir.as_str()){
                Ok(()) => (),
                Err(e) => panic!("error: {}", e),
            }
        }
        Mutex::new(cppdir)
    };
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Index {
    packages: Vec<Package>,
}
impl Index {
    pub fn contains(&self, package: &Package) -> bool {
        for pack in self.packages.iter() {
            if pack == package {
                return true;
            }
        }
        false
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct Version {
    raw: String,
}
impl Version {
    pub fn new(version: String) -> Self {
        Self { raw: version }
    }
    pub fn is_less(first: Version, second: Version) -> bool {
        false
    }
    pub fn is_greater(first: Version, second: Version) -> bool {
        false
    }
    pub fn is_equal(first: Version, second: Version) -> bool {
        if first == second {
            return true;
        }
        false
    }
}
pub enum PackageType {
    CppBuild,
    Make,
    Shell,
    Raw,
    PkgConfig,
}
pub fn download_packages(packages: &Vec<(String, String)>) -> io::Result<()> {
    /*let tmp_dir = match Builder::new().prefix("cppbuild").tempdir() {
        Ok(dir) => dir,
        Err(e) => panic!(e),
    };*/
    /*let res = reqwest::blocking::get("http://honeybucket.kittenz.pdx.edu/sources/index.json")
        .unwrap()
        .text()
        .unwrap();
    let packs: Index = toml::from_str(res.as_str()).unwrap();*/
    for (name, version) in packages.iter() {
        /*if packs.contains(&Package::new(
            name.clone(),
            version.clone(),
            None,
            None,
            None,
        )) {*/
            println!("arch: {}", get_arch());
            let mut response = reqwest::blocking::get(
                format!(
                    "http://localhost/{}/sources/{}-{}.tar.gz",
                    get_arch(), name, version
                )
                .as_str(),
            )
            .unwrap();
            let mut a = Archive::new(response);
            for file in a.entries().unwrap() {
                let file = file.unwrap();
                let path = file.path();
                println!("path: {:?}", path);
            }
       // }
    }
    Ok(())
}

pub fn generate_package(kind: PackageType, path: &str) -> std::io::Result<()> {
    match kind {
        PackageType::CppBuild => {
            let project = Project::from_file(".")?;
            compress(".", &project.get_name(), &project.get_version())?;
            Ok(())
        }
        PackageType::Make => Ok(()),
        PackageType::Shell => Ok(()),
        PackageType::Raw => Ok(()),
        PackageType::PkgConfig => {
            let mut conf = pkg_config::Config::new();
            match conf.probe(path) {
                Ok(lib) => {
                    for l in lib.link_paths.iter() {
                        // get link paths and for each file in them match it against library name
                    }
                    for i in lib.include_paths.iter() {
                        // for every file in include_paths copy the file to the archive
                    }
                }
                Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
            }
            Ok(())
        }
    }
}
pub fn compress(path: &str, name: &str, version: &str) -> std::io::Result<()> {
    use tar::Builder;
    let tarpath = format!("{}/target/{}-{}.tar", path, name, version);
    let mut tarfile = File::create(tarpath.as_str())?;
    let mut archive = Builder::new(tarfile);
    for entry in WalkDir::new(path).follow_links(true) {
        let entry = match entry {
            Ok(ent) => ent,
            Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        };
        let path = entry.path();
        let filename = format!("{}", path.display());
        if !filename.contains("target") {
            if path.is_file() {
                println!("{}", filename);
                archive.append_file(path, &mut File::open(path)?)?;
            }
        }
    }
    let mut tarfile = File::open(tarpath.as_str())?;
    let mut encoder = match Encoder::new(Vec::new()) {
        Ok(ent) => ent,
        Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
    };
    let mut data = Vec::new();
    tarfile.read_to_end(&mut data)?;
    println!("size of data: {}", data.len());
    let mut cursor = std::io::Cursor::new(data);
    io::copy(&mut cursor, &mut encoder)?;
    let encoded_data = match encoder.finish().into_result() {
        Ok(data) => data,
        Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
    };
    let mut targz = File::create(format!("{}.gz", tarpath).as_str())?;
    targz.write_all(&encoded_data)?;
    std::fs::remove_file(tarpath.as_str())
}
pub fn generate_pc(program: &Program) -> std::io::Result<()> {
    Ok(())
}
pub fn build_to_pc(project: &Project) -> std::io::Result<()> {
    Ok(())
}
pub fn get_arch() -> String {
    let rust_info = rust_info::get();
    format!("{}-{}", rust_info.target_arch.unwrap(), rust_info.target_os.unwrap())
}
