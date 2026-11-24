//! Explicit experimental native object interface; no alternate compiler.
use std::{ffi::{CString,c_char,c_void},io::Write,os::unix::ffi::OsStrExt};
unsafe extern "C" { fn dlopen(path:*const c_char,flags:i32)->*mut c_void; fn dlsym(handle:*mut c_void,name:*const c_char)->*mut c_void; }
type Sink=unsafe extern "C" fn(*mut c_void,i32,*const u8,usize);
type Compile=unsafe extern "C" fn(usize,*const *const c_char,Sink,*mut c_void)->i32;
unsafe extern "C" fn receive(context:*mut c_void,stream:i32,bytes:*const u8,len:usize){
    let out=unsafe{&mut *context.cast::<(Vec<u8>,Vec<u8>)>()};let b=unsafe{std::slice::from_raw_parts(bytes,len)};
    match stream{1=>out.0.extend_from_slice(b),2=>out.1.extend_from_slice(b),_=>()}
}
fn main(){
    let mut args=std::env::args_os().skip(1);let library=args.next().expect("LIBRARY COMPILER_ARGS...");
    let library=CString::new(library.as_os_str().as_bytes()).expect("NUL in library path");
    let arguments:Vec<CString>=args.map(|a|CString::new(a.as_os_str().as_bytes()).expect("NUL in argument")).collect();
    let pointers:Vec<_>=arguments.iter().map(|a|a.as_ptr()).collect();let mut output=(Vec::new(),Vec::new());
    let handle=unsafe{dlopen(library.as_ptr(),2)};assert!(!handle.is_null(),"native object library unavailable");
    let symbol=unsafe{dlsym(handle,c"lustr_native_object_v1".as_ptr())};assert!(!symbol.is_null(),"native object ABI unavailable");
    let compile:Compile=unsafe{std::mem::transmute(symbol)};
    let code=unsafe{compile(pointers.len(),pointers.as_ptr(),receive,(&mut output as *mut (Vec<u8>,Vec<u8>)).cast())};
    std::io::stdout().write_all(&output.0).unwrap();std::io::stderr().write_all(&output.1).unwrap();std::process::exit(code);
}
