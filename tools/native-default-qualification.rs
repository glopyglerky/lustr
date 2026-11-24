//! Native default-path regression checks. The second compiler is an explicit
//! differential oracle; no failed application is retried through another engine.
use std::{fs::{self,OpenOptions},io::Write,path::Path,process::{Command,Output}};
fn save(path:&Path,bytes:&[u8]){let mut f=OpenOptions::new().create_new(true).write(true).open(path).unwrap();f.write_all(bytes).unwrap();f.sync_all().unwrap();}
fn observed(command:&mut Command,dir:&Path,label:&str)->Output{save(&dir.join(format!("{label}.command")),format!("{command:?}\n").as_bytes());let output=command.output().unwrap();save(&dir.join(format!("{label}.stdout")),&output.stdout);save(&dir.join(format!("{label}.stderr")),&output.stderr);save(&dir.join(format!("{label}.status")),format!("{}\n",output.status).as_bytes());output}
fn main(){let args:Vec<_>=std::env::args_os().skip(1).collect();assert_eq!(args.len(),3,"NATIVE_CORE ORACLE OUTPUT_DIRECTORY");let dir=Path::new(&args[2]);fs::create_dir(dir).unwrap();
let cases=[
    ("const-fn-closure",true,"const fn make()->fn(fn()->i64)->i64{|f:fn()->i64|f()} fn value()->i64{42} fn main(){println!(\"{}\",make()(value));}"),
    ("const-fn-nested-closure",true,"const fn make()->fn(fn()->i64)->i64{|f:fn()->i64|{let nested=|g:fn()->i64|g();nested(f)}} fn value()->i64{42} fn main(){println!(\"{}\",make()(value));}"),
    ("runtime-closure",true,"fn value()->i64{42} fn main(){let call=|f:fn()->i64|f();println!(\"{}\",call(value));}"),
    ("const-pointer-call",false,"const fn call(f:fn()->i64)->i64{f()} fn main(){}"),
    ("const-pointer-dead-call",false,"const fn call(f:fn()->i64)->i64{if false{f()}else{42}} fn main(){}"),
    ("const-after-closure",false,"const fn make(f:fn()->i64)->fn()->i64{let c=||42;if false{let _=f();}c} fn main(){}"),
    ("const-block-inside-closure",false,"fn value()->i64{42} const fn make()->fn()->i64{||const{let f:fn()->i64=value;if false{f()}else{42}}} fn main(){}"),
];
for(name,valid,source)in cases{let source_path=dir.join(format!("{name}.rs"));save(&source_path,format!("{source}\n").as_bytes());let mut runs=Vec::new();for(native,compiler)in[(true,&args[0]),(false,&args[1])]{let engine=if native{"native"}else{"oracle"};let app=dir.join(format!("{name}-{engine}"));let mut command=Command::new(compiler);if native{command.arg("build");}else{command.arg("--edition=2024");}command.arg(&source_path).arg("-o").arg(&app).env_remove("LUST_LIBRARY").env_remove("LUST_STD_SRC").env("LUST_NO_CACHE","1").env("LUST_NO_DAEMON","1");let output=observed(&mut command,dir,&format!("{name}-{engine}-compile"));assert_eq!(output.status.success(),valid,"{name} {engine}: {}",String::from_utf8_lossy(&output.stderr));if valid{runs.push((engine,app));}else{assert!(!app.exists(),"invalid Rust emitted an application; it must remain NEVEREXECUTED");}}
let mut answers=Vec::new();for(engine,app)in runs{let output=observed(&mut Command::new(app),dir,&format!("{name}-{engine}-run"));assert!(output.status.success(),"{name} {engine}: {}",String::from_utf8_lossy(&output.stderr));assert_eq!(output.stdout,b"42\n");assert!(output.stderr.is_empty());answers.push(output.stdout);}if valid{assert_eq!(answers[0],answers[1]);}println!("{name}: native/oracle {}",if valid{"runtime parity"}else{"rejection/no application"});}
fs::File::open(dir).unwrap().sync_all().unwrap();}
