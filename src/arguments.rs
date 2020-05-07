
//for every flag there will be child arguments
use std::cell::RefCell;
pub struct Arguments{
    args: Vec<Arg>,
    operations: Vec<Operation>,
    flags: Option<Vec<Flag>>,
    //callback: Option<Vec<&'static Fn(Option<&Vec<Flag>>) -> ()>>,
}
#[derive(Clone)]
pub struct Operation{
    arg: String,
    func: RefCell<&'static Fn(&[Arg], &Arguments) -> ()>,
}
impl Operation{
    pub fn new(arg: String, func: &'static Fn(&[Arg], &Arguments)) -> Self{
        Self {arg, func: RefCell::new(func)}
    }
    pub fn select(name: &String, ops: &Vec<Operation>) -> Option<Self>{
        for op in ops.iter(){
            if name == &op.arg {
                return Some(op.clone());
            }
        }
        None
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Flag{
    flag: String,
    values: Vec<String>,
}
#[derive(Debug, Clone)]
pub enum ArgType{
    OPTION,
    VALUE,
    FLAG,
}
#[derive(Debug, Clone)]
pub struct Arg{
    name: String,
    kind: ArgType,
}
impl Arg{
    pub fn new(name: String, kind: ArgType) -> Self{
        Self {name, kind}
    }
}
impl Flag{
    pub fn new(flag: String, values: Vec<String>) -> Self{
        Self {flag, values}
    }
    pub fn get_name(&self) -> String{
        self.flag.clone()
    }
}
impl Arguments{
    pub fn new() -> Self{
        let mut args: Vec<String> = std::env::args().skip(1).collect();
        let mut flags = Vec::new();
        let option: String = args.remove(0);
        flags.push(Arg::new(option, ArgType::OPTION));
        for arg in args.iter(){
            if(exists(arg.find("--"))){
                flags.push(Arg::new(arg.clone(), ArgType::FLAG));
            }else{
                flags.push(Arg::new(arg.clone(), ArgType::VALUE));
            }
        }
        Self {args: flags, operations: Vec::new(), flags: None}
    }
    pub fn invoke_callback(&mut self, flag: &str, func: &'static Fn(&[Arg], &Arguments) -> ()){
        self.operations.push(
            Operation::new(flag.to_string(), func)
        );
    }
    pub fn parse(mut self){
        //seperate out flags and their children
        //then iterate through again to invoke each operation
        // convert args to slice so direct access is safer
        let mut flags_index: Vec<usize> = Vec::new();
        let mut index: usize = 0;
        for a in self.args.iter(){
            if exists(a.name.find("--")){
                flags_index.push(index);
            }
            index = index + 1;
        }
        let slice_args = self.args.as_slice();
        let mut ind: usize = 0;
        for flag_index in flags_index.iter(){
            let flag = &slice_args[flag_index.clone()];
            if let Some(mut op) = Operation::select(&flag.name, &self.operations){
                let start: usize = flag_index+1;
                let end: usize = match flags_index.get(ind+1){
                    Some(index) => *index,
                    None => slice_args.len(),
                };
                let mut fun = op.func.get_mut();
                fun(&slice_args[start..end], &self);
            }
            ind = ind + 1;
        }
    }
    pub fn get_flag(&self, flag_name: &str) -> Option<Flag>{
        match &self.flags {
            Some(fs) => {
                for flag in fs.iter(){
                    if flag.get_name() == flag_name {
                        return Some(flag.clone());
                    }
                }
                None
            },
            None => None,
        }
    }
    pub fn get_flags(&self) -> Vec<Arg>{
        self.args.clone()
    }
}
pub fn exists<T>(arg: Option<T>) -> bool{
    match arg{
        Some(_) => true,
        None => false,
    }
}