//for every flag there will be child arguments
use std::cell::RefCell;
pub struct Arguments {
    args: Vec<Arg>,
    operations: Vec<Operation>,
    flags: Option<Vec<Flag>>,
    //callback: Option<Vec<&'static Fn(Option<&Vec<Flag>>) -> ()>>,
}
#[derive(Clone)]
pub struct Operation {
    arg: String,
    func: RefCell<&'static dyn Fn(&[Arg], &Arguments) -> ()>,
}
///associates a given flag with a callback method
impl Operation {
    pub fn new(arg: String, func: &'static dyn Fn(&[Arg], &Arguments)) -> Self {
        Self {
            arg,
            func: RefCell::new(func),
        }
    }
    pub fn select(name: &str, ops: &[Operation]) -> Option<Self> {
        for op in ops.iter() {
            if name == op.arg {
                return Some(op.clone());
            }
        }
        None
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Flag {
    flag: String,
    values: Vec<Arg>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgType {
    OPTION,
    VALUE,
    FLAG,
}
///holds all arguments, allowing them to be sorted into types aka flag, value, option where values are children of flags
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Arg {
    name: String,
    kind: ArgType,
}

impl Arg {
    pub fn new(name: String, kind: ArgType) -> Self {
        Self { name, kind }
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}
///flag as something containing --
impl Flag {
    pub fn new(flag: String, values: Vec<Arg>) -> Self {
        Self { flag, values }
    }
    pub fn get_name(&self) -> String {
        self.flag.clone()
    }
    pub fn extend_from_slice(&mut self, slice: &[Arg]){
        self.values.extend_from_slice(slice);
    }
    pub fn get_values(&self) -> &[Arg]{
        self.values.as_slice()
    }
    pub fn get_value_vec(&self) -> Vec<Arg>{
        self.values.clone()
    }
}
//let fun = op.func.get_mut();
//fun(&slice_args[start..end], &self);
///main struct that organizes argument parsing
impl Arguments {
    ///constructs the parser
    pub fn new() -> Self {
        let mut args: Vec<String> = std::env::args().skip(1).collect();
        let mut arglist = Vec::new();
        let mut flags = Vec::new();
        let option: String = args.remove(0);
        let reop = option.clone();
        arglist.push(Arg::new(option, ArgType::OPTION));
        flags.push(Flag::new(reop, Vec::new()));
        for arg in args.iter() {
            if arg.find("--").is_some() {
                arglist.push(Arg::new(arg.clone(), ArgType::FLAG));
                flags.push(Flag::new(arg.clone(), Vec::new()));
            } else {
                arglist.push(Arg::new(arg.clone(), ArgType::VALUE));
            }
        }
        Self {
            args: arglist,
            operations: Vec::new(),
            flags: Some(flags),
        }
    }
    ///adds a callback for a flag when it is found
    pub fn invoke_callback(&mut self, flag: &str, func: &'static dyn Fn(&[Arg], &Arguments) -> ()) {
        self.operations.push(Operation::new(flag.to_string(), func));
    }
    ///invokes methods associated with the flags saved above
    pub fn parse(&mut self) {
        //seperate out flags and their children
        //then iterate through again to invoke each operation
        // convert args to slice so direct access is safer
        let mut flags_index: Vec<usize> = Vec::new();
        flags_index.push(0);
        for (index, a) in self.args.iter().enumerate() {
            if a.name.find("--").is_some() && index != 0 {
                flags_index.push(index);
            }
        }
        let slice_args = self.args.as_slice();
        for (ind, flag_index) in flags_index.iter().enumerate() {
            let flag = &slice_args[*flag_index];
                let start: usize = *flag_index + 1;
                let end: usize = match flags_index.get(ind + 1) {
                    Some(index) => *index,
                    None => slice_args.len(),
                };
                let f = self.get_flag_index(&flag.name).expect("flag parsing error");
                let slice = self.flags.as_mut().unwrap().as_mut_slice();
                slice[f].extend_from_slice(&slice_args[start..end]);
                
        }
        for flag in self.flags.as_ref().unwrap().iter(){
            if let Some(mut op) = Operation::select(&flag.get_name(), &self.operations){
                let fun = op.func.get_mut();
                fun(flag.get_values(), &self);
            }
        }
    }
    ///this function is broken for now
    pub fn get_flag(&self, flag_name: &str) -> Option<Flag> {
        match &self.flags {
            Some(fs) => {
                for flag in fs.iter() {
                    if flag.get_name() == flag_name {
                        return Some(flag.clone());
                    }                }
                None
            }
            None => None,
        }
    }
    pub fn get_flag_index(&self, flag_name: &str) -> Option<usize>{
        match &self.flags {
            Some(fs) => {
                for (index, flag) in fs.iter().enumerate() {
                    if flag.get_name() == flag_name {
                        return Some(index);
                    }
                }
                None
            }
            None => None,
        }
    }
    ///this allows a callback to get a specific argument or flag
    pub fn get_arg(&self, flag_name: &str) -> Option<Arg> {
        for arg in self.args.iter() {
            if arg.get_name() == flag_name {
                return Some(arg.clone());
            }
        }
        None
    }
    ///checks if a flag exists at the same level as a registered operation
    pub fn has_arg(&self, flag_name: &str) -> bool {
        self.get_arg(flag_name).is_some()
    }
    pub fn get_flags(&self) -> Vec<Arg> {
        self.args.clone()
    }
}
