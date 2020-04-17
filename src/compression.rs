/*use tempfile::TempDir;
use std::fs::{File, create_dir};
use tar::Builder;
use libflate::gzip::{Encoder, Decoder};
use crate::project::Project;
use std::io::{self, Read, Write, Result};
use walkdir::WalkDir;
use tempfile::{tempdir, tempfile};
pub struct CodeArchive{
    temp: TempDir,
}
impl CodeArchive{
    pub fn new() -> Result<Self>{
        let temp = match tempdir(){
            Ok(dir) => dir,
            Err(e) => return Err(e),
        };
        Ok(Self{temp})
    }
    pub fn compress(src: File){
        let mut encoder = Encoder::new(Vec::new()).unwrap();
        let mut data: Vec<u8> = Vec::new();
        //io::copy(, &mut encoder).unwrap();
        let encoded_data = encoder.finish().into_result().unwrap();

    }
    pub fn consolidate(self, project: &Project) -> Result<File>{
        let tarfile = match File::open(format!("{}-{}",project.get_name(), project.get_version()).as_str()){
            Ok(file) => file,
            Err(e) => return Err(e),
        };
        let mut archive = Builder::new(tarfile);
        for file in WalkDir::new(self.temp){
            archive.append_path(file.unwrap().path());
        }
        Ok(archive.into_inner().unwrap())
    }
    pub fn decompress(&mut self){

    }
}*/
