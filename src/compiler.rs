use std::mem;
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use clang::*;
use std::marker::Sized;
use clang::source::Location;
pub struct EntityLocation{
    pub file_name: Option<String>,
    pub line: u32,
    pub column: u32,
    pub offset: u32,
}
pub trait ASTEntry{
    fn get_children(&self) -> Vec<Self> where Self: Sized;
    fn get_display_name(&self) -> Option<String>;
    fn get_name(&self) -> Option<String>{
        self.get_display_name()
    }
    fn get_comment(&self) -> Option<String>{
        None
    }
    fn get_location(&self) -> Option<EntityLocation>;
    fn get_kind(&self) -> EntityKind;
}
impl ASTEntry for Entity<'_>{
    fn get_children(&self) -> Vec<Self>{
        self.get_children()
    }
    fn get_display_name(&self) -> Option<String>{
        self.get_display_name()
    }
    fn get_name(&self) -> Option<String>{
        self.get_name()
    }
    fn get_comment(&self) -> Option<String>{
        self.get_comment()
    }
    fn get_location(&self) -> Option<EntityLocation>{
        match self.get_location(){
            Some(local) => {
                let loc = local.get_file_location();
                let file_name = match loc.file {
                    Some(file) => {
                        Some(format!("{}", file.get_path().as_path().display()))
                    }
                    None => None,
                };
                Some(EntityLocation {file_name, line: loc.line, column: loc.column, offset: loc.offset})
            },
            None => None,
        }
    }
    fn get_kind(&self) -> EntityKind{
        self.get_kind()
    }
}
pub struct Compiler{
    includes: Vec<String>,
    source_files: Vec<String>,
    context: LLVMContextRef,
    modules: Vec<LLVMModuleRef>,
    arguments: Vec<String>,
}
impl Compiler{
    pub fn new(files: &[String], arguments: &[String]) -> Self{
        let mut source_files = Vec::new();
        let context = unsafe { LLVMContextCreate() };
        source_files.extend_from_slice(files);
        let mut args = Vec::new();
        args.extend_from_slice(arguments);
        Self {includes: Vec::new(), source_files, context, modules: Vec::new(), arguments: args}
    }
    pub fn append_includes(&mut self, includes: &[String]){
        self.includes.extend_from_slice(includes);
    }
    pub fn generate_module<T: ASTEntry>(entry: T){

    }
}