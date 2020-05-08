use crate::project::Package;
use crate::project::Project;
use libflate::gzip::{Decoder, Encoder};
use std::io::{Read, Write};
use walkdir::WalkDir;

use serde_derive::{Deserialize, Serialize};
use std::fs::{create_dir, File};
use std::io;
use std::io::{ErrorKind};
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
pub struct RepoEntry{
    name: String,
    version: String,
    build_type: PackageType,
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
/*impl Version {
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
}*/
#[derive(Debug, Serialize, Deserialize)]
pub enum PackageType {
    CppBuild,
    Make,
    Shell,
    Raw,
    PkgConfig,
}
pub fn download_packages(packages: &[(String, String)]) -> io::Result<()> {
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
        println!("arch: {}", get_arch()?);
        let response = reqwest::blocking::get(
            format!(
                "http://localhost/cppbuild/{}/{}-{}.tar.gz",
                get_arch()?,
                name,
                version
            )
            .as_str(),
        )
        .unwrap();
        //let mut file = File::create(format!("{}/{}-{}.tar.gz", PROGRAM_DATA.lock().unwrap(), name, version).as_str()).unwrap();
        let bytes: Vec<u8> = response.bytes().unwrap().to_vec();
        println!("bytes recv: {}", bytes.len());
        //file.write_all(&bytes).unwrap();
        //file.sync_all().unwrap();
        let mut decoder = Decoder::new(&*bytes)?;
        let mut decoded_data = Vec::new();
        decoder.read_to_end(&mut decoded_data)?;
        let mut a = Archive::new(&*decoded_data);
        a.unpack(format!("{}/{}-{}/", PROGRAM_DATA.lock().unwrap(), name, version).as_str())?;
        // }
    }
    Ok(())
}

pub fn generate_package(kind: PackageType, path: &str) -> std::io::Result<()> {
    match kind {
        PackageType::CppBuild => {
            let project = Project::from_file(path)?;
            println!("path: {}", path);
            compress(path, &project.get_name(), &project.get_version())?;
            Ok(())
        }
        PackageType::Make => Ok(()),
        PackageType::Shell => Ok(()),
        PackageType::Raw => Ok(()),
        PackageType::PkgConfig => {
            let conf = pkg_config::Config::new();
            match conf.probe(path) {
                Ok(lib) => {
                    for link in lib.link_paths.iter() {
                        // get link paths and for each file in them match it against library name
                        for l in lib.libs.iter(){
                            let path_name = format!("{}/{}", link.display(), l);
                            let file: &Path = &Path::new(&path_name);
                            if file.exists() && file.is_file(){
                                // the object file exists
                                //let mut src = File::open(file)?;
                                //create the new file
                                //let mut dst = File::create("")?;
                            }
                        }
                    }
                    /*for i in lib.include_paths.iter() {
                        // for every file in include_paths copy the file to the archive
                    }*/
                }
                Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
            }
            Ok(())
        }
    }
}
pub fn compress(path: &str, name: &str, version: &str) -> std::io::Result<String> {
    use tar::Builder;
    let tarpath = format!("{}/target/{}-{}.tar", path, name, version);
    let tarfile = File::create(tarpath.as_str())?;
    let mut archive = Builder::new(tarfile);
    for entry in WalkDir::new(path).follow_links(true) {
        let entry = match entry {
            Ok(ent) => ent,
            Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        };
        let path = entry.path();
        let filename = format!("{}", path.display());
        if !filename.contains("target") && path.is_file() {
                println!("{}", filename);
                archive.append_file(path, &mut File::open(path)?)?;
        }
    }
    archive.finish()?;
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
    std::fs::remove_file(tarpath.as_str())?;
    Ok(format!("{}.gz", tarpath))
}
/*
pub fn generate_pc(program: &Program) -> std::io::Result<()> {
    Ok(())
}
pub fn build_to_pc(package: &Package) -> std::io::Result<()> {

    Ok(())
}*/
pub fn get_arch() -> std::io::Result<String> {
    let rust_info = rust_info::get();
    Ok(format!(
        "{}-{}",
        match rust_info.target_arch {
            Some(arch) => arch,
            None =>
                return Err(std::io::Error::new(
                    ErrorKind::Other,
                    "Failed to get architecture"
                )),
        },
        match rust_info.target_os {
            Some(arch) => arch,
            None =>
                return Err(std::io::Error::new(
                    ErrorKind::Other,
                    "Failed to get architecture"
                )),
        }
    ))
}
