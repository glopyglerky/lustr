//! SDK-bootstrapped cold/warm complement boundary gate; native apps run once.
#[path="../src/native_link.rs"] mod native_link;
use std::{fs::{self,OpenOptions},io::Write,path::{Path,PathBuf},process::Command};
fn put(p:&Path,b:&[u8]) {let mut f=OpenOptions::new().create_new(true).write(true).open(p).unwrap();f.write_all(b).unwrap();}
fn main() {
 let mut args=std::env::args_os().skip(1);let root=PathBuf::from(args.next().unwrap());let library=PathBuf::from(args.next().unwrap());fs::create_dir(&root).unwrap();
 let source=root.join("main.rs");let output=root.join("current");
 // Same types and declarations; only a body expression changes, so native
 // kept-program reuse can expose a rejection without fresh collection.
 let supported="type Flag=bool; fn this_is_an_original_long_assert_condition_function_name_with_more_than_seventy_eight_bytes()->Flag{true}\nfn main(){assert!(true);println!(\"done\");}\n";
 let complement=supported.replace("assert!(true)","assert!(this_is_an_original_long_assert_condition_function_name_with_more_than_seventy_eight_bytes() && true)");
 let environment:Vec<_>=std::env::vars_os().collect();
 let request=vec!["build".into(),source.to_str().unwrap().into(),"-o".into(),output.to_str().unwrap().into(),"--timings".into()];
 let mut native_outputs=Vec::new();
 for (label,text,code) in [("supported",supported,0),("warm-complement",complement.as_str(),1),("recover",supported,0)] {
  fs::write(&source,text).unwrap();put(&root.join(format!("{label}.source.rs")),text.as_bytes());
  let (actual,stdout,stderr)=native_link::compile_session(&library,&request,&environment).unwrap().unwrap();
  put(&root.join(format!("{label}.stdout")),&stdout);put(&root.join(format!("{label}.stderr")),&stderr);put(&root.join(format!("{label}.status")),format!("{actual}\n").as_bytes());
  assert_eq!(actual,code,"{label}: {}",String::from_utf8_lossy(&stderr));
  if code==0 {let dest=root.join(format!("{label}.elf"));fs::rename(&output,&dest).unwrap();native_outputs.push((label.to_owned(),dest));}
  else {assert!(!output.exists());assert!(!String::from_utf8_lossy(&stderr).contains("constraint-derived conversion sites"));}
 }
 fs::write(&source,&complement).unwrap();
 let cold=root.join("cold-complement.elf");let mut command=Command::new(std::env::var_os("LUSTR_SESSION_COLD_PROBE").unwrap());command.arg(&library).args(["build",source.to_str().unwrap(),"-o",cold.to_str().unwrap()]);
 let stock=root.join("stock-complement.elf");let mut oracle=Command::new(std::env::var_os("LUSTR_SESSION_ORACLE").unwrap());oracle.arg(&source).args(["--edition=2024","-Cpanic=unwind","-Coverflow-checks=on","-o"]).arg(&stock);
 for (label,mut command,out) in [("cold-complement",command,cold),("stock-complement",oracle,stock)] {
  let result=command.output().unwrap();put(&root.join(format!("{label}.stdout")),&result.stdout);put(&root.join(format!("{label}.stderr")),&result.stderr);put(&root.join(format!("{label}.status")),format!("{}\n",result.status).as_bytes());assert!(result.status.success(),"{label}: {}",String::from_utf8_lossy(&result.stderr));native_outputs.push((label.to_owned(),out));
 }
 let mut expected=None;
 for (label,out) in native_outputs {let result=Command::new(out).output().unwrap();put(&root.join(format!("{label}.run.stdout")),&result.stdout);put(&root.join(format!("{label}.run.stderr")),&result.stderr);put(&root.join(format!("{label}.run.status")),format!("{}\n",result.status).as_bytes());assert!(result.status.success(),"failed application must not replay");if let Some(expected)=&expected{assert_eq!(&result.stdout,expected)}else{expected=Some(result.stdout)}}
 println!("warm complement remains rejected without fresh collection; supported recovery and cold native/stock complement runtime parity; four distinct apps once");
}
