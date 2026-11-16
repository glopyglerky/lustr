//! Standalone integration gate. Bootstrap this harness independently; every
//! candidate application below is compiled by the named native entry.
#[path = "../src/native_link.rs"]
mod native_link;
use std::{ffi::OsString, fs::{self, OpenOptions}, io::Write, path::{Path, PathBuf}, process::{Command, Output}};

fn overlap(root:&Path,library:&Path) {
    use std::ffi::{CString,c_char,c_void};
    use std::os::unix::ffi::OsStrExt;
    type Sink=unsafe extern "C" fn(*mut c_void,i32,*const u8,usize);
    type Object=unsafe extern "C" fn(usize,*const *const c_char,Sink,*mut c_void)->i32;
    type Session=unsafe extern "C" fn(usize,*const *const c_char,usize,*const *const c_char,*const c_char,Sink,*mut c_void)->i32;
    unsafe extern "C" {fn dlopen(path:*const c_char,flags:i32)->*mut c_void;fn dlsym(handle:*mut c_void,name:*const c_char)->*mut c_void;}
    unsafe extern "C" fn discard(_: *mut c_void,_:i32,_:*const u8,_:usize) {}
    struct Observed {object:Object,overlap:Option<i32>}
    unsafe extern "C" fn receive(context:*mut c_void,_:i32,_:*const u8,_:usize) {
        let observed=unsafe{&mut *context.cast::<Observed>()};
        if observed.overlap.is_some(){return;}
        let object=observed.object;
        observed.overlap=Some(std::thread::spawn(move||{
            let arg=CString::new("--sentinel-overlap-probe").unwrap();let ptrs=[arg.as_ptr()];
            // Valid framing reaches native argument rejection (1) if it can
            // enter. Entry-lock refusal is 3 and consumes no object call.
            unsafe{object(1,ptrs.as_ptr(),discard,std::ptr::null_mut())}
        }).join().unwrap_or(-999));
    }
    let path=CString::new(library.as_os_str().as_bytes()).unwrap();
    let handle=unsafe{dlopen(path.as_ptr(),2)};assert!(!handle.is_null());
    let object=unsafe{dlsym(handle,c"lustr_native_object_v1".as_ptr())};let session=unsafe{dlsym(handle,c"lustr_native_session_v1".as_ptr())};assert!(!object.is_null()&&!session.is_null());
    let object:Object=unsafe{std::mem::transmute(object)};let session:Session=unsafe{std::mem::transmute(session)};
    let source=root.join("check.rs");fs::write(&source,"fn main() {}\n").unwrap();
    let args=[CString::new("check").unwrap(),CString::new(source.as_os_str().as_bytes()).unwrap()];let ptrs:Vec<_>=args.iter().map(|s|s.as_ptr()).collect();
    let namespace=CString::new(fs::read_to_string(library.with_file_name("lustr-native.id")).unwrap().trim()).unwrap();
    let mut observed=Observed{object,overlap:None};
    let code=unsafe{session(ptrs.len(),ptrs.as_ptr(),0,std::ptr::null(),namespace.as_ptr(),receive,(&mut observed as *mut Observed).cast())};
    put(&root.join("session.status"),format!("{code}\n").as_bytes());assert_eq!(code,0);
    put(&root.join("overlap.status"),format!("{:?}\n",observed.overlap).as_bytes());assert_eq!(observed.overlap,Some(3));
    let arg=CString::new("--sentinel-overlap-probe").unwrap();let ptrs=[arg.as_ptr()];
    let code=unsafe{object(1,ptrs.as_ptr(),discard,std::ptr::null_mut())};put(&root.join("object-after-overlap.status"),format!("{code}\n").as_bytes());assert_eq!(code,1);
    println!("cross-thread session/object ABI overlap refused by shared entry lock; subsequent object call reached normal argument rejection");
}

const SOURCE: &str = r#"use std::cell::Cell;
use std::rc::Rc;
use std::panic::{catch_unwind, AssertUnwindSafe};
mod numbers;
struct Guard(Rc<Cell<usize>>);
impl Drop for Guard { fn drop(&mut self) { self.0.set(self.0.get()+1); } }
fn identity<T>(value:T)->T { value }
fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let drops=Rc::new(Cell::new(0));
    let failed=catch_unwind(AssertUnwindSafe(|| { let _g=Guard(drops.clone()); panic!("unwind"); }));
    assert!(failed.is_err()); assert_eq!(drops.get(),1);
    let answer:u64=identity(numbers::step(6));
    println!("{}:{}:{}:{}",answer,drops.get(),include_str!("input.txt"),option_env!("LUSTR_SHARED_LABEL").unwrap_or("unset"));
}
"#;

fn put(path:&Path,bytes:&[u8]) { let mut f=OpenOptions::new().create_new(true).write(true).open(path).unwrap();f.write_all(bytes).unwrap(); }
fn save(root:&Path,label:&str,output:&Output) {
    put(&root.join(format!("{label}.stdout")),&output.stdout);
    put(&root.join(format!("{label}.stderr")),&output.stderr);
    put(&root.join(format!("{label}.status")),format!("{}\n",output.status).as_bytes());
}
fn args(source:&Path,out:&Path)->Vec<String> {vec!["build".into(),source.to_str().unwrap().into(),"-o".into(),out.to_str().unwrap().into(),"--timings".into()]}

fn lifecycle(root:&Path,library:&Path,mode:&str) {
    let source=root.join("check.rs");fs::write(&source,"fn main() {}\n").unwrap();
    let args=vec!["check".into(),source.to_str().unwrap().into()];
    let environment:Vec<_>=std::env::vars_os().collect();
    let call=|label:&str,args:&[String],expected:i32| {
        let before:Vec<_>=std::env::vars_os().collect();
        let (code,stdout,stderr)=native_link::compile_session(library,args,&environment).unwrap().unwrap();
        assert_eq!(std::env::vars_os().collect::<Vec<_>>(),before);
        put(&root.join(format!("{label}.stdout")),&stdout);put(&root.join(format!("{label}.stderr")),&stderr);put(&root.join(format!("{label}.status")),format!("{code}\n").as_bytes());
        assert_eq!(code,expected,"{label}: {}",String::from_utf8_lossy(&stderr));
    };
    if mode=="runtime" {
        let mutable=PathBuf::from(std::env::var_os("LUSTR_SESSION_MUTABLE_RUNTIME").unwrap());
        fs::create_dir(&mutable).unwrap();
        call("runtime-initial",&args,0);
        put(&mutable.join("changed-input"),b"runtime generation changed\n");
        call("runtime-changed",&args,2);call("runtime-retired",&args,2);
    } else {
        assert_eq!(mode,"lifecycle");call("owner-initial",&args,0);
        let child_library=library.to_owned();let child_args=args.clone();let child_env=environment.clone();
        let (code,stdout,stderr)=std::thread::spawn(move||native_link::compile_session(&child_library,&child_args,&child_env).unwrap().unwrap()).join().unwrap();
        put(&root.join("other-thread.status"),format!("{code}\n").as_bytes());assert_eq!(code,2);assert!(stdout.is_empty()&&stderr.is_empty());
        call("owner-resumed",&args,0);
        // Two valid calls consumed the owner's allowance. The rejected other
        // thread consumed none. Invalid protocol frames do not compile code.
        for n in 0..62 {call(&format!("bounded-invalid-{n}"),&[],2);}
        call("call-limit",&args,3);
    }
    println!("native session {mode} gate passed");
}

fn closure(root:&Path,input:&Path) {
    for name in ["initial","noop","body-edit","include-edit","recovery","source-recovery","refused-header-recovery","failed-collect-recovery","parse-failure-recovery","complement","after-complement"] {
        let path=input.join(format!("{name}.elf"));
        let output=Command::new("readelf").args(["-W","-d"]).arg(&path).output().unwrap();save(root,name,&output);assert!(output.status.success());
        let text=String::from_utf8(output.stdout).unwrap();
        assert!(!text.contains("libbase-")&&!text.contains("(RPATH)")&&!text.contains("(RUNPATH)"),"{name} depends on compiler-owned runtime artifacts: {text}");
    }
    println!("eleven actual shared-native standalone ELF dependency closures passed after compiler process exit; no application replay");
}

struct Gate { root:PathBuf, library:PathBuf, source:PathBuf, out:PathBuf, environment:Vec<(OsString,OsString)>, seq:usize }
impl Gate {
    fn shared(&mut self,label:&str,expected:i32) -> String {
        self.seq+=1;
        let before:Vec<_>=std::env::vars_os().collect();
        let (code,stdout,stderr)=native_link::compile_session(&self.library,&args(&self.source,&self.out),&self.environment).unwrap().unwrap();
        assert_eq!(std::env::vars_os().collect::<Vec<_>>(),before);
        put(&self.root.join(format!("{label}.source.rs")),&fs::read(&self.source).unwrap());
        put(&self.root.join(format!("{label}.stdout")),&stdout);
        put(&self.root.join(format!("{label}.stderr")),&stderr);
        put(&self.root.join(format!("{label}.status")),format!("{code}\n").as_bytes());
        assert_eq!(code,expected,"{label}: {}",String::from_utf8_lossy(&stderr));
        let text=String::from_utf8(stderr).unwrap();
        assert!(!text.contains("verified no-op artifact reuse"));
        if code==0 {
            assert_eq!(text.matches("lust-stats:").count(),1,"supported compile must enter one native driver phase: {text}");
            assert!(text.contains("unit_cache_hits=0"));
            assert!(!text.contains("link libbase.so"),"standalone request linked an intermediate image: {text}");
            assert_eq!(text.lines().filter(|line|line.split_whitespace().next()==Some("link")).count(),1,"standalone request needs one native final link: {text}");
        } else { assert!(!self.out.exists(),"rejected source produced an artifact"); }
        text
    }
    fn runtime(&self,label:&str)->Vec<u8> {
        let output=Command::new(&self.out).output().unwrap(); save(&self.root,&format!("{label}-run"),&output);
        assert!(output.status.success(),"{label}: application failed; never replay it");
        fs::rename(&self.out,self.root.join(format!("{label}.elf"))).unwrap();
        output.stdout
    }
    fn controls(&self,label:&str,expected:i32,pristine:bool)->Option<Vec<u8>> {
        let mut expected_output=None;
        let cold=self.root.join(format!("{label}-cold.elf"));
        let mut native=Command::new(std::env::var_os("LUSTR_SESSION_COLD_PROBE").unwrap());
        native.arg(&self.library).args(args(&self.source,&cold));
        let oracle=self.root.join(format!("{label}-stock.elf"));
        let mut stock=Command::new(std::env::var_os("LUSTR_SESSION_ORACLE").unwrap());
        stock.arg(&self.source).args(["--edition=2024","-Cpanic=unwind","-Coverflow-checks=on"]).arg("-o").arg(&oracle);
        let mut commands=vec![("cold",native,cold),("stock",stock,oracle)];
        if pristine {
            let target=self.root.join(format!("{label}-pristine.elf"));
            let mut native=Command::new(std::env::var_os("LUSTR_SESSION_PRISTINE").unwrap());
            native.args(args(&self.source,&target)).env("LUST_LIBRARY","/package/lib/lust/library");
            commands.push(("pristine",native,target));
        }
        for (kind,mut command,out) in commands {
            for(k,v)in &self.environment {command.env(k,v);}
            let output=command.output().unwrap();save(&self.root,&format!("{label}-{kind}"),&output);
            assert_eq!(output.status.code(),Some(expected),"{label}-{kind}: {}",String::from_utf8_lossy(&output.stderr));
            if expected==0 {
                let runtime=Command::new(&out).output().unwrap();save(&self.root,&format!("{label}-{kind}-run"),&runtime);
                assert!(runtime.status.success(),"{label}-{kind}: runtime failure; never replay");
                if let Some(previous)=&expected_output {assert_eq!(&runtime.stdout,previous);} else {expected_output=Some(runtime.stdout);}
            } else {assert!(!out.exists());}
        }
        expected_output
    }
}

fn main() {
    let mut cli=std::env::args_os().skip(1);let root=PathBuf::from(cli.next().expect("NEW_OUTPUT_DIRECTORY"));
    let library=PathBuf::from(cli.next().expect("LIBRARY"));fs::create_dir(&root).unwrap();
    if let Some(mode)=cli.next() {
        let mode=mode.to_str().unwrap();
        if mode=="closure" {closure(&root,&library);} else if mode=="overlap" {overlap(&root,&library);} else {lifecycle(&root,&library,mode);}
        return;
    }
    let project=root.join("project");fs::create_dir(&project).unwrap();
    let source=project.join("main.rs");fs::write(&source,SOURCE).unwrap();
    let module=project.join("numbers.rs");fs::write(&module,"pub fn step(value:u64)->u64 {value+1}\n").unwrap();
    let input=project.join("input.txt");fs::write(&input,"first").unwrap();
    let mut environment:Vec<_>=std::env::vars_os().collect();environment.retain(|(k,_)|k!="LUSTR_SHARED_LABEL");environment.push(("LUSTR_SHARED_LABEL".into(),"alpha".into()));
    let mut gate=Gate {out:root.join("current"),root,source,library,environment,seq:0};
    gate.shared("initial",0);assert_eq!(gate.runtime("initial"),gate.controls("initial",0,true).unwrap());
    let noop=gate.shared("noop",0);assert!(noop.contains("rechecked=0"),"native semantic state was not retained: {noop}");
    assert_eq!(gate.runtime("noop"),b"7:1:first:alpha\n");
    fs::write(&module,"pub fn step(value:u64)->u64 {value+9}\n").unwrap();
    gate.shared("body-edit",0);assert_eq!(gate.runtime("body-edit"),gate.controls("body-edit",0,true).unwrap());
    fs::write(&input,"changed").unwrap();
    gate.shared("include-edit",0);assert_eq!(gate.runtime("include-edit"),gate.controls("include-edit",0,true).unwrap());
    fs::write(&module,"pub fn step(value:u64)->bool {value>0}\n").unwrap();
    gate.shared("signature-reject",1);assert!(gate.controls("signature-reject",1,true).is_none());
    fs::write(&module,"pub fn step(value:u64)->u64 {value+3}\n").unwrap();
    gate.shared("recovery",0);assert_eq!(gate.runtime("recovery"),gate.controls("recovery",0,true).unwrap());
    let invalid=SOURCE.replace("let answer:u64=identity(numbers::step(6));","let owner=String::from(\"moved\");let moved=owner;println!(\"{owner}\");let answer:u64=identity(numbers::step(6));");
    fs::write(&gate.source,invalid).unwrap();gate.shared("move-reject",1);gate.controls("move-reject",1,true);
    fs::write(&gate.source,SOURCE).unwrap();gate.shared("source-recovery",0);assert_eq!(gate.runtime("source-recovery"),gate.controls("source-recovery",0,true).unwrap());
    for (label,text) in [
        ("refused-header",SOURCE.replace("fn identity<T>(value:T)->T { value }","fn identity<T:MissingTrait>(value:T)->T { value }")),
        ("failed-collect",format!("type Broken=MissingType;\n{SOURCE}")),
        ("parse-failure",SOURCE.replace("fn identity<T>(value:T)->T { value }","fn identity<T>(value:T)->T { value")),
    ] {
        fs::write(&gate.source,text).unwrap();gate.shared(label,1);gate.controls(label,1,true);
        fs::write(&gate.source,SOURCE).unwrap();let recovery=format!("{label}-recovery");gate.shared(&recovery,0);assert_eq!(gate.runtime(&recovery),gate.controls(&recovery,0,true).unwrap());
    }
    let original=include_str!("../tests/fixtures/native-inference-unsafe-local-const-fn-original.rs");
    fs::write(&gate.source,original).unwrap();
    // Complement source has two full native rechecks today. It is separately
    // qualified below and is not covered by the one-driver supported-path assertion.
    let (code,stdout,stderr)=native_link::compile_session(&gate.library,&args(&gate.source,&gate.out),&gate.environment).unwrap().unwrap();
    put(&gate.root.join("complement.stdout"),&stdout);put(&gate.root.join("complement.stderr"),&stderr);put(&gate.root.join("complement.status"),format!("{code}\n").as_bytes());
    assert_eq!(code,0,"{}",String::from_utf8_lossy(&stderr));
    assert_eq!(gate.runtime("complement"),gate.controls("complement",0,false).unwrap());
    fs::write(&gate.source,SOURCE).unwrap();gate.shared("after-complement",0);assert_eq!(gate.runtime("after-complement"),gate.controls("after-complement",0,true).unwrap());
    println!("native session: supported/edit/include/signature/move/recovery and unchanged complete complement runtime parity passed; native crate artifacts/Cargo/full Rust/speed remain open");
}
