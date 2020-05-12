use llvm_sys::execution_engine::*;
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use libc::c_char;
use std::mem;
pub struct ExecutionEnv{
    engine: *mut LLVMExecutionEngineRef,
    err: *mut *mut c_char,
}
impl ExecutionEnv{
    pub fn from_module(module: LLVMModuleRef) -> Self{
        unsafe {
            let mut err = mem::zeroed();
            let mut engine = mem::uninitialized();
            let errorcode = LLVMCreateExecutionEngineForModule(&mut engine, module, &mut err);
            Self {engine: engine as *mut LLVMExecutionEngineRef, err: err as *mut *mut c_char}
        }
    }
}