//! Opt-in tests of the experimental native ABI, with no compatibility fallback.
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[test]
#[ignore = "requires native qualified local items and native/oracle controls"]
fn unsafe_local_const_fn_items_keep_qualifiers_scopes_and_const_rules() {
    let f=Fixture::new("unsafe-local-const-fn-corpus");
    let original=include_str!("fixtures/native-inference-unsafe-local-const-fn-original.rs");
    let path=f.source.join("tests/fixtures/native-inference-unsafe-local-const-fn-original.rs");
    let bytes=fs::read(&path).unwrap();f.parity("original",&path);assert_eq!(fs::read(&path).unwrap(),bytes);
    let local="const fn safe_value()->bool{false}";
    let end=";2])[0]));\n}";
    for(name,text)in[
        ("header-comments",original.replacen(local,"const /*qualifier*/ fn /*keyword*/ safe_value()->bool{false}",1)),
        ("forward-const-item",original.replacen(local,"",1).replace(end,";2])[0]));\nconst fn safe_value()->bool{false}\n}")),
        ("const-siblings",original.replacen(local,"const fn safe_value()->bool{id(false)} const fn id(x:bool)->bool{x}",1)),
        ("nested-const-item",original.replacen(local,"const fn safe_value()->bool{const fn id(x:bool)->bool{x} id(false)}",1)),
        ("const-block-value",original.replacen(local,"const fn safe_value()->bool{const {false}}",1)),
        ("local-const-evaluation",original.replacen(local,"const fn safe_value()->bool{false}{const CHECK:bool=safe_value();assert!(!CHECK);}",1)),
        ("nested-nonconst-scope",original.replacen(local,"const fn safe_value()->bool{fn unused()->bool{println!(\"runtime only\");false} false}",1)),
    ]{let path=f.root.join(format!("{name}.rs"));fs::write(&path,text).unwrap();f.parity(name,&path);}
    for(name,text)in[
        ("const-nonconst-call",format!("{}\nfn runtime_only()->bool{{false}}",original.replacen(local,"const fn safe_value()->bool{runtime_only()}",1))),
        ("const-nonconst-macro",original.replacen(local,"const fn safe_value()->bool{println!(\"not const\");false}",1)),
        ("const-destructor",original.replacen(local,"const fn safe_value()->bool{let _owned=String::new();false}",1)),
        ("local-const-false-assertion",original.replacen(local,"const fn safe_value()->bool{false}{const _:()=assert!(safe_value());}",1)),
        ("duplicate-qualified-item",original.replacen(local,"const fn safe_value()->bool{false} fn safe_value()->bool{true}",1)),
        ("const-captures-local",original.replacen(local,"let captured=false;const fn safe_value()->bool{captured}",1)),
        ("const-wrong-body",original.replacen(local,"const fn safe_value()->bool{0u8}",1)),
        ("const-wrong-result",original.replacen(local,"const fn safe_value()->u8{0}",1)),
        ("const-wrong-input",original.replacen(local,"const fn safe_value(x:bool)->bool{x}",1)),
        ("const-moved-owner",original.replace("let drops = Rc::new(Cell::new(0));","let drops = Rc::new(Cell::new(0));drop(drops);")),
        ("const-deny-unsafe",format!("#![deny(unsafe_code)]\n{original}")),
        ("duplicate-const-qualifier",original.replacen(local,"const const fn safe_value()->bool{false}",1)),
    ]{f.rejected(name,&text);}
    // Unsupported qualifiers remain failures, even when Rust accepts them.
    // Compile the oracle independently; do not execute these applications.
    for(name,header)in[("unsafe","const unsafe fn safe_value()->bool{false}"),("foreign-abi","const extern \"C\" fn safe_value()->bool{false}")]{
        let source=f.root.join(format!("unsupported-{name}.rs"));fs::write(&source,original.replacen(local,header,1)).unwrap();
        let out=f.root.join(format!("unsupported-{name}-native"));let native=f.run(&format!("unsupported-{name}-native"),&mut f.native(&source,&out));assert_eq!(native.status.code(),Some(1));assert!(!out.exists());
        let oracle=f.root.join(format!("unsupported-{name}-oracle"));assert!(f.run(&format!("unsupported-{name}-oracle"),&mut f.oracle(&source,&oracle)).status.success());
    }
}

#[test]
#[ignore = "requires native nested-item integration and native/oracle controls"]
fn unsafe_nested_fn_items_preserve_lexical_scopes_arenas_and_checks() {
    let f=Fixture::new("unsafe-nested-fn-assert-corpus");
    let original=include_str!("fixtures/native-inference-unsafe-nested-fn-assert-original.rs");
    let path=f.source.join("tests/fixtures/native-inference-unsafe-nested-fn-assert-original.rs");
    let bytes=fs::read(&path).unwrap();f.parity("original",&path);assert_eq!(fs::read(&path).unwrap(),bytes);
    let local="fn fail(drops: Rc<Cell<usize>>) {fn safe_value()->bool{false}";
    let plain="fn fail(drops: Rc<Cell<usize>>) {";
    let end=";2])[0]));\n}";
    for(name,text)in[
        ("forward-item",original.replace(local,plain).replace(end,";2])[0]));\nfn safe_value()->bool{false}\n}")),
        ("local-shadows-global",original.replacen("fn safe_value()->bool{false}","fn safe_value()->bool{println!(\"local target\");false}",1)),
        ("sibling-items",original.replacen("fn safe_value()->bool{false}","fn safe_value()->bool{id(false)} fn id(x:bool)->bool{println!(\"sibling {}\",x);x}",1)),
        ("nested-items",original.replacen("fn safe_value()->bool{false}","fn safe_value()->bool{fn id(x:bool)->bool{x} id(false)}",1)),
        ("nested-block",original.replace(local,"fn fail(drops: Rc<Cell<usize>>) {{fn safe_value()->bool{false}").replace(end,";2])[0]));\n}}")),
        ("string-arenas",original.replacen("fn safe_value()->bool{false}","fn safe_value()->bool{println!(\"item {{braces}} λ\");false}",1)),
        ("item-unwinds",original.replacen("fn safe_value()->bool{false}","fn safe_value()->bool{println!(\"local unwind\");panic!(\"local item panic\")}",1)),
    ]{let path=f.root.join(format!("{name}.rs"));fs::write(&path,text).unwrap();f.parity(name,&path);}
    for(name,text)in[
        ("duplicate-item",original.replace(local,"fn fail(drops: Rc<Cell<usize>>) {fn safe_value()->bool{false} fn safe_value()->bool{true}")),
        ("captured-local",original.replace(local,"fn fail(drops: Rc<Cell<usize>>) {let captured=false;fn safe_value()->bool{captured}")),
        ("local-wrong-return",original.replacen("fn safe_value()->bool{false}","fn safe_value()->u8{0}",1)),
        ("local-wrong-body",original.replacen("fn safe_value()->bool{false}","fn safe_value()->bool{0u8}",1)),
        ("local-wrong-input",original.replacen("fn safe_value()->bool{false}","fn safe_value(x:bool)->bool{x}",1)),
        ("local-shadows-item",original.replace("unsafe {safe_value()}","unsafe {let safe_value=false;safe_value()}")),
        ("moved-owner",original.replace("let drops = Rc::new(Cell::new(0));","let drops = Rc::new(Cell::new(0));drop(drops);")),
        ("const-nonconst-item",original.replace("unsafe {safe_value()}","const {unsafe {safe_value()}}")),
    ]{f.rejected(name,&text);}
}

#[test]
#[ignore = "requires native call/conversion composition and native/oracle controls"]
fn unsafe_call_and_delayed_conversion_preserve_complete_native_rechecks() {
    let f=Fixture::new("unsafe-call-conversion-corpus");
    let base=include_str!("fixtures/native-inference-owned-path.rs");
    let before=format!("{}\nfn safe_value()->bool{{true}}\n",base.replace("fn value() -> PathBuf {","fn value() -> PathBuf {let _boolean=unsafe {safe_value()};"));
    let after=format!("{}\nfn safe_value()->bool{{true}}\n",base.replace("    checked_target\n","    let _boolean=unsafe {safe_value()};\n    checked_target\n"));
    let nested=before.replace("fn value() -> PathBuf {","fn value() -> PathBuf {fn safe_value()->bool{true}");
    let qualified=nested.replace("{fn safe_value()->bool{true}","{const fn safe_value()->bool{true}");
    for(name,text)in[("before",&before),("after",&after),("nested-item",&nested),("local-const-item",&qualified)]{let path=f.root.join(format!("{name}.rs"));fs::write(&path,text).unwrap();f.parity(name,&path);let log=fs::read_to_string(f.root.join(format!("{name}-build.stderr"))).unwrap();assert!(log.contains("constraint diagnostics remain")&&log.contains("full native recheck follows"),"{log}");}
    f.rejected("moved-conversion-source",&before.replace("let inferred = source.into();","drop(source);let inferred = source.into();"));
    f.rejected("shadowed-call-target",&before.replace("let _boolean=unsafe", "let safe_value=false;let _boolean=unsafe"));
    f.rejected("nested-item-moved-conversion-source",&nested.replace("let inferred = source.into();","drop(source);let inferred = source.into();"));
}

#[test]
#[ignore = "requires original-context native safe-call checking and native/oracle controls"]
fn unsafe_call_block_assertions_check_targets_scopes_order_and_unwind() {
    let f=Fixture::new("unsafe-call-block-assert-corpus");
    let original=include_str!("fixtures/native-inference-unsafe-call-block-assert-original.rs");
    let path=f.source.join("tests/fixtures/native-inference-unsafe-call-block-assert-original.rs");
    let bytes=fs::read(&path).unwrap();f.parity("original",&path);assert_eq!(fs::read(&path).unwrap(),bytes);
    let item=f.root.join("body-item.rs");fs::write(&item,original.replace("fn fail(drops: Rc<Cell<usize>>) {","fn fail(drops: Rc<Cell<usize>>) {fn safe_value()->bool{false}")).unwrap();f.parity("body-item",&item);
    for (name,body,helper) in [
        ("local-init","unsafe {let x=safe_value();x}",""),
        ("typed-local","unsafe {let x:bool=safe_value();x}",""),
        ("nested-capture","unsafe {let x=safe_value();unsafe {id(x)}}","fn id(x:bool)->bool{x}"),
        ("nested-argument","unsafe {id(safe_value())}","fn id(x:bool)->bool{x}"),
        ("shadow-init","unsafe {let x=safe_value();let x=id(x);x}","fn id(x:bool)->bool{x}"),
        ("qualified","unsafe {module::value()}","mod module{pub fn value()->bool{false}}"),
        ("order-once","unsafe {both(first(),second())}","fn first()->bool{println!(\"first\");true} fn second()->bool{println!(\"second\");false} fn both(a:bool,b:bool)->bool{println!(\"both {} {}\",a,b);b}"),
        ("panic-unwind","unsafe {safe_value()}",""),
    ] {
        let mut text=format!("{}\n{helper}\n",original.replace("unsafe {safe_value()}",body));
        if name=="panic-unwind"{text=text.replace("fn safe_value()->bool{false}","fn safe_value()->bool{println!(\"call before unwind\");panic!(\"safe call panic\")}");}
        let source=f.root.join(format!("{name}.rs"));fs::write(&source,text).unwrap();f.parity(name,&source);
    }
    for (name,text) in [
        ("wrong-arity",original.replace("safe_value()}","safe_value(false)}")),
        ("wrong-input",original.replace("fn safe_value()->bool{false}","fn safe_value(x:u8)->bool{x==0}")),
        ("wrong-return",original.replace("fn safe_value()->bool{false}","fn safe_value()->u8{0}")),
        ("local-shadows-function",original.replace("unsafe {safe_value()}","unsafe {let safe_value=false;safe_value()}")),
        ("moved-owner",original.replace("let drops = Rc::new(Cell::new(0));","let drops = Rc::new(Cell::new(0));drop(drops);")),
        ("deny-unsafe",format!("#![deny(unsafe_code)]\n{original}")),
        ("const-nonconst-call",original.replace("unsafe {safe_value()}","const {unsafe {safe_value()}}")),
    ] {f.rejected(name,&text);}
    // Valid Rust outside the admitted target/ABI grammar stays unsupported.
    // The oracle only compiles these; no unsafe application is executed.
    for (name,text) in [
        ("unsafe-target",original.replace("fn safe_value()->bool{false}","unsafe fn safe_value()->bool{false}")),
        ("foreign-abi",original.replace("fn safe_value()->bool{false}","extern \"C\" fn safe_value()->bool{false}")),
        ("pointer-target",original.replace("unsafe {safe_value()}","unsafe {let f=safe_value;f()}")),
    ] {
        let source=f.root.join(format!("unsupported-{name}.rs"));fs::write(&source,text).unwrap();
        let out=f.root.join(format!("unsupported-{name}-native"));let native=f.run(&format!("unsupported-{name}-native"),&mut f.native(&source,&out));
        assert_eq!(native.status.code(),Some(1));assert!(!out.exists());
        let oracle_out=f.root.join(format!("unsupported-{name}-oracle"));assert!(f.run(&format!("unsupported-{name}-oracle"),&mut f.oracle(&source,&oracle_out)).status.success());
    }
}

#[test]
#[ignore = "requires owned unsafe-body lexical checking and native/oracle controls"]
fn unsafe_let_block_assertions_preserve_scopes_shadowing_types_and_payloads() {
    let f = Fixture::new("unsafe-let-block-assert-corpus");
    let original = include_str!("fixtures/native-inference-unsafe-let-block-assert-original.rs");
    let path = f.source.join("tests/fixtures/native-inference-unsafe-let-block-assert-original.rs");
    let before = fs::read(&path).unwrap();
    f.parity("original", &path);
    assert_eq!(fs::read(&path).unwrap(), before);
    for (name, value) in [
        ("typed", "unsafe {let x:bool=false;x}"),
        ("typed-comments", "unsafe {let x /*pattern*/ : /*type*/ bool /*value*/ =false;x}"),
        ("typed-long", "unsafe {let this_is_a_long_local_boolean_binding_that_requires_a_native_panic_text_line_break:bool=false;this_is_a_long_local_boolean_binding_that_requires_a_native_panic_text_line_break}"),
        ("alias", "unsafe {let x=false;let y=x;y}"),
        ("shadow-initializer", "unsafe {let x=false;let x=x;x}"),
        ("scope-restoration", "unsafe {let x=false;{let x=true;x};x}"),
        ("nested-safe-capture", "unsafe {let x=false;{let y=x;y}}"),
        ("nested-unsafe-capture", "unsafe {let x=false;unsafe {let y=x;y}}"),
        ("nested-unsafe-initializer", "unsafe {let x=unsafe {let y=false;y};x}"),
        ("nested-mutable", "unsafe {let mut x=unsafe {false};x}"),
        ("nested-typed-mutable", "unsafe {let mut x:bool=unsafe {false};x}"),
        ("wildcard", "unsafe {let _=true;let x=false;x}"),
        ("mutable-binding", "unsafe {let mut x=false;x}"),
        ("expression-statement", "unsafe {true;let x=false;x}"),
        ("long-binding", "unsafe {let this_is_a_long_local_boolean_binding_that_requires_a_native_panic_text_line_break=false;this_is_a_long_local_boolean_binding_that_requires_a_native_panic_text_line_break}"),
    ] {
        let source=f.root.join(format!("{name}.rs"));
        fs::write(&source,original.replace("unsafe {let x=false;x}",value)).unwrap();
        f.parity(name,&source);
    }
    for (name,value) in [
        ("self-initializer","unsafe {let x=x;x}"),
        ("forward-reference","unsafe {let x=y;let y=false;x}"),
        ("escaped-local","unsafe {{let x=false;x};x}"),
        ("escaped-nested-unsafe-local","unsafe {unsafe {let x=false;x};x}"),
        ("type-annotation","unsafe {let x:u8=false;x}"),
        ("alias-type","unsafe {let x:bool=1u8;x}"),
        ("missing-initializer","unsafe {let x:bool;x}"),
        ("mutable-wildcard","unsafe {let mut _=unsafe {false};false}"),
    ] { f.rejected(name,&original.replace("unsafe {let x=false;x}",value)); }
    // Item-pattern ambiguity is resolved by the complete native checker,
    // after the safety derivation; a spelling alone never proves a binding.
    f.rejected("const-pattern",&format!("const x:bool=true;\n{original}"));
    f.rejected("static-shadow",&format!("static x:bool=false;\n{original}"));
}

#[test]
#[ignore = "requires the owned original-token unsafe frontend and native/oracle controls"]
fn unsafe_block_assertions_preserve_original_context_checks_and_payloads() {
    let f = Fixture::new("unsafe-block-assert-corpus");
    let original = include_str!("fixtures/native-inference-unsafe-block-assert-original.rs");
    f.parity("original", &f.source.join("tests/fixtures/native-inference-unsafe-block-assert-original.rs"));
    let macros=f.root.join("unrelated-macro-context.rs");
    fs::write(&macros,format!("macro_rules! outside{{()=>{{1u8}}}}\n{}\nfn unrelated()->u8{{outside!()}}\n",original.replace("let drops = Rc::new(Cell::new(0));","let _ = outside!(); let drops = Rc::new(Cell::new(0));"))).unwrap();
    f.parity("unrelated-macro-context",&macros);
    for (name,value) in [
        ("wrapped", "(((unsafe {false})))"),
        ("comments", "unsafe /* keyword */ { /* body */ false /* end */ }"),
        ("nested", "unsafe { unsafe { false } }"),
        ("block-tail", "unsafe { {false} }"),
    ] {
        let source = f.root.join(format!("{name}.rs"));
        fs::write(&source,original.replace("unsafe {false}",value)).unwrap();
        f.parity(name,&source);
    }
    let passing = f.root.join("unsafe-true.rs");
    fs::write(&passing,original.replace("unsafe {false}","unsafe {true}")
        .replace("assert!(failed.is_err(),", "assert!(failed.is_ok(),")).unwrap();
    f.parity("unsafe-true",&passing);
    let parenthesized=f.root.join("parenthesized-statement.rs");
    fs::write(&parenthesized,"fn main(){(unsafe {true}) == false;}\n").unwrap();
    f.parity("parenthesized-statement",&parenthesized);
    for (name,text) in [
        ("block-comparison-boundary","fn main(){unsafe {true} == false;}"),
        ("block-call-boundary","fn main(){unsafe {true}();}"),
        ("block-index-boundary","fn main(){unsafe {true}[0];}"),
    ] { f.rejected(name,text); }
    for (name,text) in [
        ("condition-type",original.replace("unsafe {false}","unsafe {1u8}")),
        ("length-type",original.replace(";2]",";false]")),
        ("runtime-length",original.replace(";2]",";drops.get()]")),
        ("moved-owner",original.replace("let drops = Rc::new(Cell::new(0));","let drops = Rc::new(Cell::new(0)); drop(drops);")),
        ("outside-raw-deref",original.replace("let drops = Rc::new(Cell::new(0));","let value=1u8; let ptr=&value as *const u8; let copied=*ptr; let drops = Rc::new(Cell::new(0));")),
        ("deny-unsafe",format!("#![deny(unsafe_code)]\n{original}")),
    ] { f.rejected(name,&text); }
    // Valid Rust outside the proven body grammar remains unsupported natively.
    // Compilation of the oracle is independent; no unsafe runtime is probed.
    for (name,value) in [("closure","unsafe {(||false)()}"),("raw-deref","unsafe {*(&false as *const bool)}")] {
        let source=f.root.join(format!("unsupported-{name}.rs"));
        fs::write(&source,format!("{}\nfn safe_value()->bool{{false}}\n",original.replace("unsafe {false}",value))).unwrap();
        let out=f.root.join(format!("unsupported-{name}-native"));
        let native=f.run(&format!("unsupported-{name}-native"),&mut f.native(&source,&out));
        assert_eq!(native.status.code(),Some(1)); assert!(!out.exists());
        assert!(String::from_utf8_lossy(&native.stderr).contains("L0404"));
        let oracle_out=f.root.join(format!("unsupported-{name}-oracle"));
        assert!(f.run(&format!("unsupported-{name}-oracle"),&mut f.oracle(&source,&oracle_out)).status.success());
    }
}

#[test]
#[ignore = "requires the explicitly built original-AST assertion text ABI"]
fn long_assertions_preserve_original_and_exact_false_payloads() {
    let f = Fixture::new("long-assert-original-payload");
    for name in ["native-inference-long-assert-original", "native-inference-long-assert-payload", "native-inference-parenthesized-assert-original", "native-inference-binary-assert-original", "native-inference-struct-assert-original", "native-inference-tuple-assert-original", "native-inference-tuple-literal-assert-original"] {
        let source = f.source.join("tests/fixtures").join(format!("{name}.rs"));
        let bytes = fs::read(&source).unwrap();
        f.parity(name, &source);
        let stderr = fs::read_to_string(f.root.join(format!("{name}-build.stderr"))).unwrap();
        assert!(stderr.contains("original-AST assertion texts"), "{stderr}");
        assert_eq!(fs::read(source).unwrap(), bytes);
    }
}

#[test]
#[ignore = "requires original-AST const blocks and native/oracle runtime controls"]
fn const_block_assertions_preserve_const_checks_nesting_and_exact_payloads() {
    let f=Fixture::new("const-block-assert-corpus");
    let original=include_str!("fixtures/native-inference-const-block-assert-original.rs");
    f.parity("original",&f.source.join("tests/fixtures/native-inference-const-block-assert-original.rs"));
    for (name,value) in [
        ("wrapped","(((const {false})))"),
        ("comments","const /* keyword */ { /* body */ false /* end */ }"),
        ("nested","const { const { false } }"),
        ("let","const { let x=false; x }"),
        ("statements","const { true; false }"),
        ("block-tail","const { {false} }"),
        ("binary","const { 1u8 != 1u8 }"),
        ("tuple","const { (false,2u8).0 }"),
        ("array","const { [false,true][0] }"),
        ("long","const { let this_is_a_very_long_const_block_local_binding_that_forces_a_line_break=false; this_is_a_very_long_const_block_local_binding_that_forces_a_line_break }"),
    ] {
        let source=f.root.join(format!("{name}.rs"));
        fs::write(&source,original.replace("const {false}",value)).unwrap();
        f.parity(name,&source);
    }
    let passing=f.root.join("const-true.rs");
    fs::write(&passing,original.replace("const {false}","const {true}")
        .replace("assert!(failed.is_err(),", "assert!(failed.is_ok(),")).unwrap();
    f.parity("const-true",&passing);
    f.rejected("const-type",&original.replace("const {false}","const {1u8}"));
    f.rejected("const-runtime-capture",&original.replace("const {false}","const {drops.get() == 0}"));
    f.rejected("const-nonconst-call",&(original.replace("const {false}","const {not_const()}")+"\nfn not_const()->bool{false}\n"));
    f.rejected("const-mutability",&original.replace("const {false}","const {let x=false;x=true;x}"));
    f.rejected("const-evaluation-panic",&original.replace("const {false}","const {panic!(\"const failure\");false}"));
}

#[test]
#[ignore = "requires original-AST array repeats and native/oracle runtime controls"]
fn array_repeat_assertions_preserve_value_length_order_borrows_and_exact_payloads() {
    let f=Fixture::new("array-repeat-assert-corpus");
    let original=include_str!("fixtures/native-inference-array-repeat-assert-original.rs");
    let binary=include_str!("fixtures/native-inference-binary-assert-original.rs");
    f.parity("original",&f.source.join("tests/fixtures/native-inference-array-repeat-assert-original.rs"));
    for (name,suffix) in [
        ("pair","|| [false;2][0]"),
        ("singleton","|| [false;1][0]"),
        ("zero","|| empty_array([false;0])"),
        ("wrapped-array","|| ((([false;2])))[0]"),
        ("comments","|| [/* value */ false /* semi */ ; /* len */ 2 /* end */][0]"),
        ("wrapped-children","|| [(((false)));(((2)))][0]"),
        ("nested","|| [[false;2];2][0][0]"),
        ("tuple-value","|| [(false,2u8);2][0].0"),
        ("list-value","|| [[false,true];2][0][0]"),
        ("call","|| array_first([false;2])"),
        ("binary-length","|| [false;1+1][0]"),
        ("block-length","|| [false;{1+1}][0]"),
        ("named-length","|| [false;REPEAT_COUNT][0]"),
        ("long-length","|| [false;THIS_IS_A_VERY_LONG_ARRAY_REPEAT_LENGTH_CONSTANT_NAME_THAT_FORCES_A_LINE_BREAK][0]"),
        ("long-value","|| [this_is_a_very_long_array_repeat_value_function_name_that_forces_a_line_break(false);2][0]"),
        ("borrow","|| { let owner=String::from(\"x\"); let a=[&owner;2]; a[0].is_empty() }"),
        ("single-drop","|| { let a=[Guard(drops.clone());1]; false }"),
        ("zero-drop","|| { let a=[Guard(drops.clone());0]; false }"),
        ("value-unwind","|| { let a=[{panic!(\"repeat unwind\");false};2]; false }"),
        ("value-once","|| { let a=[{let n=drops.get();assert_eq!(n,1);drops.set(n+1);false};2]; a[0] }"),
        ("zero-value-once","|| { let a=[{let n=drops.get();assert_eq!(n,1);drops.set(n+1);false};0]; empty_array(a) }"),
    ] {
        let mut text=binary.replace("&& true",suffix);
        if matches!(name,"single-drop"|"zero-drop"|"value-once"|"zero-value-once") { text=text.replace("assert_eq!(drops.get(), 1);","assert_eq!(drops.get(), 2);"); }
        let source=f.root.join(format!("{name}.rs"));
        fs::write(&source,format!("{text}\nconst REPEAT_COUNT:usize=2;\nconst THIS_IS_A_VERY_LONG_ARRAY_REPEAT_LENGTH_CONSTANT_NAME_THAT_FORCES_A_LINE_BREAK:usize=2;\nfn empty_array(x:[bool;0])->bool{{x.len()!=0}}\nfn array_first(x:[bool;2])->bool{{x[0]}}\nfn this_is_a_very_long_array_repeat_value_function_name_that_forces_a_line_break(x:bool)->bool{{x}}\n")).unwrap();
        f.parity(name,&source);
    }
    let long="this_is_an_original_long_assert_condition_function_name_with_more_than_seventy_eight_bytes";
    let constant=f.root.join("const-true.rs");fs::write(&constant,format!("const fn {long}()->bool{{true}}const _:()={{assert!({long}() && [true;2][0]);}};fn main(){{}}")).unwrap();f.parity("const-true",&constant);
    f.rejected("const-false",&format!("const fn {long}()->bool{{true}}const _:()={{assert!({long}() && [false;2][0]);}};fn main(){{}}"));
    f.rejected("repeat-condition-type",&original.replace("[false;2]","[1u8;2]"));
    f.rejected("repeat-length-type",&original.replace("[false;2]","[false;false]"));
    f.rejected("repeat-runtime-length",&original.replace("[false;2]","[false;drops.get()]"));
    f.rejected("repeat-copy",&binary.replace("&& true","|| { let a=[String::from(\"x\");2]; a[0].is_empty() }"));
    f.rejected("repeat-moved",&binary.replace("&& true","|| { let s=String::from(\"x\"); let a=[s;1]; s.is_empty() }"));
    f.rejected("repeat-borrow",&binary.replace("&& true","|| { let mut s=String::from(\"x\"); let a=[&s;2]; s.clear(); a[0].is_empty() }"));
}

#[test]
#[ignore = "requires original-AST array literals and native/oracle runtime controls"]
fn array_literal_assertions_preserve_elements_wrappers_order_borrows_and_exact_payloads() {
    let f=Fixture::new("array-literal-assert-corpus");
    let original=include_str!("fixtures/native-inference-array-literal-assert-original.rs");
    let binary=include_str!("fixtures/native-inference-binary-assert-original.rs");
    f.parity("original",&f.source.join("tests/fixtures/native-inference-array-literal-assert-original.rs"));
    for (name,suffix) in [
        ("pair","|| [false,true][0]"),
        ("singleton","|| [false][0]"),
        ("singleton-comma","|| [false,][0]"),
        ("empty","|| empty_array([])"),
        ("wrapped-array","|| ((([false,true])))[0]"),
        ("trailing-comma","|| [false,true,][0]"),
        ("comments","|| [/* first */ false /* comma */, /* second */ true, /* end */][0]"),
        ("wrapped-element","|| [(((false))),true][0]"),
        ("nested","|| [[false,true],[true,false]][0][0]"),
        ("tuple-element","|| [(false,2u8),(true,3u8)][0].0"),
        ("call","|| array_first([false,true])"),
        ("long-element","|| [this_is_a_very_long_array_element_function_name_that_forces_a_line_break(false),true][0]"),
        ("long-list","|| [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false][0]"),
        ("array-borrow","|| { let owner=String::from(\"x\"); let a=[&owner,&owner]; a[0].is_empty() }"),
        ("array-drop","|| { let a=[Guard(drops.clone()),Guard(drops.clone())]; false }"),
        ("array-unwind","|| { let a=[Guard(drops.clone()),{panic!(\"array unwind\")}]; false }"),
        ("array-order-once","|| { let a=[{let n=drops.get();assert_eq!(n,1);drops.set(n+1);false},{let n=drops.get();assert_eq!(n,2);drops.set(n+1);true}]; a[0] }"),
    ] {
        let mut text=binary.replace("&& true",suffix);
        if name=="array-drop" || name=="array-order-once" { text=text.replace("assert_eq!(drops.get(), 1);","assert_eq!(drops.get(), 3);"); }
        if name=="array-unwind" { text=text.replace("assert_eq!(drops.get(), 1);","assert_eq!(drops.get(), 2);"); }
        let source=f.root.join(format!("{name}.rs"));
        fs::write(&source,format!("{text}\nfn empty_array(x:[bool;0])->bool{{x.len()!=0}}\nfn array_first(x:[bool;2])->bool{{x[0]}}\nfn this_is_a_very_long_array_element_function_name_that_forces_a_line_break(x:bool)->bool{{x}}\n")).unwrap();
        f.parity(name,&source);
    }
    let long="this_is_an_original_long_assert_condition_function_name_with_more_than_seventy_eight_bytes";
    let constant=f.root.join("const-true.rs");fs::write(&constant,format!("const fn {long}()->bool{{true}}const _:()={{assert!({long}() && [true][0]);}};fn main(){{}}")).unwrap();f.parity("const-true",&constant);
    f.rejected("const-false",&format!("const fn {long}()->bool{{true}}const _:()={{assert!({long}() && [false][0]);}};fn main(){{}}"));
    f.rejected("array-condition-type",&original.replace("[false,true]","[1u8,2u8]"));
    f.rejected("array-heterogeneous",&original.replace("[false,true]","[false,2u8]"));
    f.rejected("array-move-invalid",&binary.replace("&& true","|| { let s=String::from(\"x\"); let a=[s,s]; a[0].is_empty() }"));
    f.rejected("array-borrow-invalid",&binary.replace("&& true","|| { let mut s=String::from(\"x\"); let a=[&s,{s.clear();&s}]; a[0].is_empty() }"));
}

#[test]
#[ignore = "requires original-AST tuple literal formatting and native/oracle runtime controls"]
fn tuple_literal_assertions_preserve_elements_wrappers_order_borrows_and_exact_payloads() {
    let f=Fixture::new("tuple-literal-assert-corpus");
    let original=include_str!("fixtures/native-inference-tuple-literal-assert-original.rs");
    let binary=include_str!("fixtures/native-inference-binary-assert-original.rs");
    f.parity("original",&f.source.join("tests/fixtures/native-inference-tuple-literal-assert-original.rs"));
    for (name,suffix) in [
        ("pair","|| (false,2u8).0"),
        ("singleton","|| (false,).0"),
        ("wrapped-singleton","|| (((false,))).0"),
        ("trailing-comma","|| (false,2u8,).0"),
        ("comments","|| (/* first */ false /* comma */, /* second */ 2u8, /* end */).0"),
        ("wrapped-element","|| (((false)),2u8).0"),
        ("unit-element","|| ((),false).1"),
        ("nested-chain","|| ((false,2u8),).0.0"),
        ("nested-four-indexes","|| ((((false,2u8),),),).0.0.0.0"),
        ("tuple-call-closure","|| (|x| x)((false,2u8)).0"),
        ("closure-element","|| ((|| false),2u8).0()"),
        ("long-element","|| (this_is_a_very_long_tuple_element_function_name_that_forces_a_line_break(false),2u8).0"),
        ("tuple-borrow","|| { let owner=String::from(\"x\"); let p=(&owner,false); p.0.is_empty() }"),
        ("tuple-drop","|| { let p=(Guard(drops.clone()),false); p.1 }"),
        ("tuple-unwind","|| (Guard(drops.clone()),{panic!(\"tuple unwind\")}).1"),
        ("tuple-order-once","|| { let p=({let n=drops.get();assert_eq!(n,1);drops.set(n+1);false},{let n=drops.get();assert_eq!(n,2);drops.set(n+1);2u8}); p.0 }"),
    ] {
        let mut text=binary.replace("&& true",suffix);
        if name=="tuple-order-once" { text=text.replace("assert_eq!(drops.get(), 1);","assert_eq!(drops.get(), 3);"); }
        if name=="tuple-drop" || name=="tuple-unwind" { text=text.replace("assert_eq!(drops.get(), 1);","assert_eq!(drops.get(), 2);"); }
        let source=f.root.join(format!("{name}.rs"));
        fs::write(&source,format!("{text}\nfn this_is_a_very_long_tuple_element_function_name_that_forces_a_line_break(x:bool)->bool{{x}}\n")).unwrap();
        f.parity(name,&source);
    }
    let long="this_is_an_original_long_assert_condition_function_name_with_more_than_seventy_eight_bytes";
    let constant=f.root.join("const-true.rs");fs::write(&constant,format!("const fn {long}()->bool{{true}}const _:()={{assert!({long}() && (true,).0);}};fn main(){{}}")).unwrap();f.parity("const-true",&constant);
    f.rejected("const-false",&format!("const fn {long}()->bool{{true}}const _:()={{assert!({long}() && (false,).0);}};fn main(){{}}"));
    f.rejected("tuple-type",&original.replace("false,2u8","1u32,2u8"));
    f.rejected("tuple-index",&original.replace("((false,2u8)).0","((false,2u8)).2"));
    f.rejected("tuple-move",&binary.replace("&& true","|| { let s=String::from(\"x\"); let p=(s,s); p.0.is_empty() }"));
    f.rejected("tuple-borrow-invalid",&binary.replace("&& true","|| { let mut s=String::from(\"x\"); let p=(&s,{s.clear();false}); p.0.is_empty() }"));
}

#[test]
#[ignore = "requires original-AST tuple-field formatting and native/oracle runtime controls"]
fn tuple_field_assertions_preserve_indexes_chains_order_borrows_and_exact_payloads() {
    let f=Fixture::new("tuple-field-assert-corpus");
    let original=include_str!("fixtures/native-inference-tuple-assert-original.rs");
    let binary=include_str!("fixtures/native-inference-binary-assert-original.rs");
    f.parity("original",&f.source.join("tests/fixtures/native-inference-tuple-assert-original.rs"));
    for (name, suffix, declaration) in [
        ("comments", "|| (Pair{0:false,1:2u8}) /* base */ . /* index */ 0", "struct Pair(bool,u8);"),
        ("wrappers", "|| (((Pair{0:false,1:2u8}))).0", "struct Pair(bool,u8);"),
        ("second-index", "|| (Pair{0:2u8,1:false}).1", "struct Pair(u8,bool);"),
        ("call", "|| make_pair().0", "struct Pair(bool,u8);fn make_pair()->Pair{Pair(false,2)}"),
        ("float-token-chain", "|| (Nest{0:Pair{0:false,1:2u8}}).0.0", "struct Pair(bool,u8);struct Nest(Pair);"),
        ("spaced-chain", "|| (Nest{0:Pair{0:false,1:2u8}}).0 /* middle */ .0", "struct Pair(bool,u8);struct Nest(Pair);"),
        ("wrapped-chain", "|| ((Nest{0:Pair{0:false,1:2u8}}).0).0", "struct Pair(bool,u8);struct Nest(Pair);"),
        ("triple-chain", "|| (Outer{0:Nest{0:Pair{0:false,1:2u8}}}).0.0.0", "struct Pair(bool,u8);struct Nest(Pair);struct Outer(Nest);"),
        ("two-float-tokens", "|| (Outer{0:Mid{0:Nest{0:Pair{0:false,1:2u8}}}}).0.0.0.0", "struct Pair(bool,u8);struct Nest(Pair);struct Mid(Nest);struct Outer(Mid);"),
        ("named-then-tuple", "|| (Holder{value:Pair{0:false,1:2u8}}).value.0", "struct Pair(bool,u8);struct Holder{value:Pair}"),
        ("tuple-then-named", "|| (Wrap{0:Holder{value:false}}).0.value", "struct Holder{value:bool}struct Wrap(Holder);"),
        ("block-base", "|| { let p=Pair{0:false,1:2u8}; p }.0", "struct Pair(bool,u8);"),
        ("borrowed-field", "|| { let owner=String::from(\"x\"); let p=Text{0:&owner}; p.0.is_empty() }", "struct Text<'a>(&'a String);"),
        ("field-drop", "|| { let p=WithGuard{0:Guard(drops.clone()),1:false}; p.1 }", "struct WithGuard(Guard,bool);"),
        ("field-unwind", "|| (WithGuard{0:Guard(drops.clone()),1:{panic!(\"field unwind\")}}).1", "struct WithGuard(Guard,bool);"),
        ("field-order-once", "|| { let value=(Pair{0:{let n=drops.get();assert_eq!(n,1);drops.set(n+1);false},1:{let n=drops.get();assert_eq!(n,2);drops.set(n+1);2u8}}).0; value }", "struct Pair(bool,u8);"),
    ] {
        let mut text=binary.replace("&& true",suffix);
        if name=="field-order-once" { text=text.replace("assert_eq!(drops.get(), 1);","assert_eq!(drops.get(), 3);"); }
        if name=="field-drop" || name=="field-unwind" { text=text.replace("assert_eq!(drops.get(), 1);","assert_eq!(drops.get(), 2);"); }
        let source=f.root.join(format!("{name}.rs"));fs::write(&source,format!("{text}\n{declaration}\n")).unwrap();f.parity(name,&source);
    }
    let long="this_is_an_original_long_assert_condition_function_name_with_more_than_seventy_eight_bytes";
    let constant=f.root.join("const-true.rs");fs::write(&constant,format!("struct Pair(bool,u8);const fn {long}()->bool{{true}}const _:()={{assert!({long}() && (Pair{{0:true,1:2u8}}).0);}};fn main(){{}}")).unwrap();f.parity("const-true",&constant);
    f.rejected("const-false",&format!("struct Pair(bool,u8);const fn {long}()->bool{{true}}const _:()={{assert!({long}() && (Pair{{0:false,1:2u8}}).0);}};fn main(){{}}"));
    for (name, field) in [("unknown-index","2"),("leading-zero","00"),("suffix","0u32"),("overflow-index","4294967296"),("float-exponent","0.0e1")] {
        f.rejected(name,&original.replace("}).0)",&format!("}}).{field})")));
    }
    f.rejected("field-type",&original.replace("0:false","0:1u32"));
    f.rejected("field-move",&format!("{}\nstruct Text(String,String);",binary.replace("&& true","|| { let s=String::from(\"x\"); let p=Text{0:s,1:s}; p.0.is_empty() }")));
    f.rejected("field-borrow",&format!("{}\nstruct Text<'a>(&'a String,bool);",binary.replace("&& true","|| { let mut s=String::from(\"x\"); let p=Text{0:&s,1:{s.clear();false}}; p.0.is_empty() }")));
}

#[test]
#[ignore = "requires original-AST struct formatting and native/oracle runtime controls"]
fn struct_assertions_preserve_fields_rest_order_borrows_and_exact_payloads() {
    let f=Fixture::new("struct-assert-corpus");
    let original=include_str!("fixtures/native-inference-struct-assert-original.rs");
    let binary=include_str!("fixtures/native-inference-binary-assert-original.rs");
    f.parity("original",&f.source.join("tests/fixtures/native-inference-struct-assert-original.rs"));
    for (name, fields, declaration) in [
        ("explicit", "value:false", "struct Holder{value:bool}"),
        ("trailing-comma", "value:false,", "struct Holder{value:bool}"),
        ("shorthand", "value", "struct Holder{value:bool}"),
        ("raw-ident", "r#type:false", "struct Holder{r#type:bool}"),
        ("multifield", "value:false, second:2u8", "struct Holder{value:bool,second:u8}"),
        ("broken-field", "this_is_a_very_long_field_name_that_forces_the_struct_box_to_break_across_lines:false", "struct Holder{this_is_a_very_long_field_name_that_forces_the_struct_box_to_break_across_lines:bool}"),
        ("rest", "value:false,..base", "struct Holder{value:bool,second:u8}"),
        ("rest-only", "..base", "struct Holder{value:bool,second:u8}"),
        ("nested", "value:(Holder{value:false}).value", "struct Holder{value:bool}"),
        ("comments", "/* name */ value /* colon */ : /* expression */ false /* end */ ,", "struct Holder{value:bool}"),
    ] {
        let field=if name=="raw-ident" { "r#type" } else if name=="broken-field" { "this_is_a_very_long_field_name_that_forces_the_struct_box_to_break_across_lines" } else { "value" };
        let setup=if name.starts_with("rest") { "let base=Holder{value:false,second:2u8};" } else if name=="shorthand" { "let value=false;" } else { "" };
        let source=f.root.join(format!("{name}.rs"));
        let text=binary.replace("&& true",&format!("|| {{ {setup} (Holder{{{fields}}}).{field} }}"));
        fs::write(&source,format!("{text}\n{declaration}\n")).unwrap();f.parity(name,&source);
    }
    for (name, suffix, declaration) in [
        ("empty", "|| { let _empty=Empty{}; false }", "struct Empty{}"),
        ("tuple-fields", "|| pair_value(Pair{0:false,1:2u8})", "struct Pair(bool,u8);fn pair_value(p:Pair)->bool{p.0}"),
        ("qualified", "|| (nested::Holder{value:false}).value", "mod nested{pub struct Holder{pub value:bool}}"),
        ("borrowed-field", "|| { let owner=String::from(\"x\"); let h=Text{a:&owner}; h.a.is_empty() }", "struct Text<'a>{a:&'a String}"),
        ("field-drop", "|| { let h=WithGuard{guard:Guard(drops.clone()),value:false}; h.value }", "struct WithGuard{guard:Guard,value:bool}"),
        ("field-unwind", "|| (WithGuard{guard:Guard(drops.clone()),value:{panic!(\"field unwind\")}}).value", "struct WithGuard{guard:Guard,value:bool}"),
        ("field-order-once", "|| { let value=(Holder{first:{let n=drops.get();assert_eq!(n,1);drops.set(n+1);false},second:{let n=drops.get();assert_eq!(n,2);drops.set(n+1);false}}).first; value }", "struct Holder{first:bool,second:bool}"),
    ] {
        let mut text=binary.replace("&& true",suffix);
        if name=="field-order-once" { text=text.replace("assert_eq!(drops.get(), 1);","assert_eq!(drops.get(), 3);"); }
        if name=="field-drop" || name=="field-unwind" { text=text.replace("assert_eq!(drops.get(), 1);","assert_eq!(drops.get(), 2);"); }
        let source=f.root.join(format!("{name}.rs"));fs::write(&source,format!("{text}\n{declaration}\n")).unwrap();f.parity(name,&source);
    }
    let long="this_is_an_original_long_assert_condition_function_name_with_more_than_seventy_eight_bytes";
    let constant=f.root.join("const-true.rs");fs::write(&constant,format!("struct Holder{{value:bool}}const fn {long}()->bool{{true}}const _:()={{assert!({long}() && (Holder{{value:true}}).value);}};fn main(){{}}")).unwrap();f.parity("const-true",&constant);
    f.rejected("const-false",&format!("struct Holder{{value:bool}}const fn {long}()->bool{{true}}const _:()={{assert!({long}() && (Holder{{value:false}}).value);}};fn main(){{}}"));
    f.rejected("field-type",&original.replace("value: true","value: 1u32"));
    f.rejected("unknown-field",&original.replace("Holder { value: true }","Holder { missing: true }"));
    f.rejected("field-move",&format!("{}\nstruct Text{{a:String,b:String}}",binary.replace("&& true","|| { let s=String::from(\"x\"); let h=Text{a:s,b:s}; h.a.is_empty() }")));
    f.rejected("field-borrow",&format!("{}\nstruct Text<'a>{{a:&'a String,b:bool}}",binary.replace("&& true","|| { let mut s=String::from(\"x\"); let h=Text{a:&s,b:{s.clear();false}}; h.a.is_empty() }")));
}

#[test]
#[ignore = "requires original-AST binary formatting and native/oracle runtime controls"]
fn binary_assertions_preserve_operators_nesting_lazy_order_and_payloads() {
    let f=Fixture::new("binary-assert-corpus");
    let original=include_str!("fixtures/native-inference-binary-assert-original.rs");
    for (name, rhs) in [
        ("or", "|| false"),
        ("lazy-and", "&& { drops.set(99); true }"),
        ("nested", "|| false && true || (false || false)"),
        ("arithmetic", "|| { let a=12u32; a + 3 * 2 - 4 / 2 % 2 == 0 }"),
        ("right-associative-source", "|| { let a=12u32; a - (3 - 1) == 0 }"),
        ("bitwise", "|| { let a=12u32; a | 3 ^ 2 & 1 == 0 }"),
        ("shift", "|| { let a=12u32; a << 2 >> 1 == 0 }"),
        ("comparison", "|| { let a=12u32; (a < 3) == (a <= 4) && (a > 5) != (a >= 6) }"),
        ("prefix", "|| { let a=12u32; !a == 0 }"),
        ("nested-parens", "|| (((false || false) && (true == true)))"),
        ("rhs-once", "|| { let before=drops.get(); assert_eq!(before,1); drops.set(before + 1); false }"),
    ] {
        assert_eq!(original.matches("&& true").count(),1);
        let source=f.root.join(format!("{name}.rs"));
        let mut text=original.replace("&& true",rhs);
        if name=="rhs-once" { text=text.replace("assert_eq!(drops.get(), 1);","assert_eq!(drops.get(), 2);"); }
        fs::write(&source,text).unwrap();f.parity(name,&source);
    }
    let lazy_or=f.root.join("lazy-or.rs");
    fs::write(&lazy_or,original.replace("assert!((catch_unwind", "assert!(((catch_unwind")
        .replace("})).is_err() && true));", "})).is_err() == false || { drops.set(99); false }) && false));")).unwrap();
    f.parity("lazy-or",&lazy_or);
    // All runtime operands run through unchanged native checking/generation.
    let long="this_is_an_original_long_assert_condition_function_name_with_more_than_seventy_eight_bytes";
    let constant=f.root.join("const-true.rs");
    fs::write(&constant,format!("const fn {long}()->bool{{true}}const _:()={{assert!(({long}() && true) || false);}};fn main(){{}}")).unwrap();f.parity("const-true",&constant);
    f.rejected("const-false",&format!("const fn {long}()->bool{{false}}const _:()={{assert!(({long}() && true) || false);}};fn main(){{}}"));
    f.rejected("rhs-type",&original.replace("&& true","&& 1u32"));
    f.rejected("rhs-move",&original.replace("&& true","|| { let s=String::from(\"x\"); drop(s); drop(s); false }"));
    f.rejected("rhs-borrow",&original.replace("&& true","|| { let mut xs=vec![1u8]; let live=&xs[..]; xs.push(2); live.is_empty() }"));
}

#[test]
#[ignore = "requires the explicitly built original-token parenthesis ABI"]
fn parenthesized_assertions_preserve_nesting_calls_closures_units_and_payloads() {
    let f = Fixture::new("parenthesized-assert-corpus");
    let original=include_str!("fixtures/native-inference-parenthesized-assert-original.rs");
    let plain=include_str!("fixtures/native-inference-long-assert-payload.rs");
    for (name,text) in [
        ("double", original.replace("assert!((catch_unwind", "assert!(((catch_unwind").replace("})).is_err()));", "})).is_err())));")),
        ("triple", original.replace("assert!((catch_unwind", "assert!((((catch_unwind").replace("})).is_err()));", "})).is_err()))));")),
        ("callee", original.replace("catch_unwind(AssertUnwindSafe", "((catch_unwind))(AssertUnwindSafe")),
        ("closure", original.replace("AssertUnwindSafe(|| {", "AssertUnwindSafe(((|| {").replace("})).is_err()", "})))).is_err()")),
        ("arguments", original.replace("Guard(drops.clone())", "(Guard)(((drops.clone())))")),
        ("unit", original.replace("println!(\"once {{raw braces}} λ\");", "println!(\"once {{raw braces}} λ\"); ((()))")),
        ("macro-statement", original.replace("println!(\"once {{raw braces}} λ\");", "((println!(\"once {{raw braces}} λ\")));")),
        ("block", plain.replace("assert!(catch_unwind", "assert!(({ catch_unwind").replace("})).is_err());", "})).is_err() }));")),
        ("receiver", plain.replace("assert!(catch_unwind", "assert!((catch_unwind").replace("})).is_err());", "}))).is_err());")),
    ] {
        let source=f.root.join(format!("{name}.rs")); fs::write(&source,text).unwrap(); f.parity(name,&source);
    }
    let long="this_is_an_original_long_assert_condition_function_name_with_more_than_seventy_eight_bytes";
    let const_true=f.root.join("const-true.rs");
    fs::write(&const_true,format!("const fn {long}()->bool{{true}}const _:()={{assert!((({long}())));}};fn main(){{println!(\"constant passes\");}}")).unwrap();
    f.parity("const-true",&const_true);
    f.rejected("const-false",&format!("const fn {long}()->bool{{false}}const _:()={{assert!((({long}())));}};fn main(){{}}"));
    f.rejected("move",&original.replace("let _guard = Guard(drops.clone());","let _guard = Guard(drops.clone()); drop(drops); drop(drops);"));
    f.rejected("mutate-borrow",&original.replace("let _guard = Guard(drops.clone());","let _guard = Guard(drops.clone()); let mut data=vec![1u8]; let live=&data[..]; data.push(2); println!(\"{:?}\",live);"));
    let float=f.root.join("parenthesized-float.rs");
    fs::write(&float,format!("use std::panic::{{catch_unwind,AssertUnwindSafe}};fn main(){{let {long}=1.0f64;std::panic::set_hook(Box::new(|p|println!(\"payload {{}}\",p.payload_as_str().unwrap_or(\"?\"))));let result=catch_unwind(AssertUnwindSafe(||{{assert!((((0.)).eq(&({long}))));}}));assert!(result.is_err(),\"must unwind\");}}")).unwrap();
    f.parity("parenthesized-float",&float);
}

#[test]
#[ignore = "requires the explicitly built original-AST assertion text ABI"]
fn long_assertion_text_tracks_width_tokens_literals_and_raw_identifiers() {
    let f = Fixture::new("long-assert-text-corpus");
    for (i, literal) in [
        r#""long text with {{ braces }} and escaped \"quote\" plus enough letters to require a pretty-print break""#,
        r##"r#"long raw text {braces} λλλλλλλλλλλλλλλλλλλλλλλλλλλλλλλλλλλλλλλλλλλλλλλλλλλλλ"#"##,
        r#""αβγδεζηθικλμνξοπρστυφχψωabcdefghijklmnopqrstuvwxyz0123456789""#,
    ].iter().enumerate() {
        let text = format!(r#"
use std::panic::{{catch_unwind, AssertUnwindSafe}};
struct Holder {{ r#type: String }}
fn main() {{
    std::panic::set_hook(Box::new(|p|println!("payload {{}}",p.payload_as_str().unwrap_or("?"))));
    let r#match=Holder{{r#type:String::from("absent")}};
    let result=catch_unwind(AssertUnwindSafe(||{{
        assert!(r#match.r#type.contains(/* comment removed by AST printing */ {literal}));
    }}));
    assert!(result.is_err(),"must fail");
    println!("after {{}}",r#match.r#type);
}}
"#);
        let source = f.root.join(format!("literal-{i}.rs")); fs::write(&source,text).unwrap();
        f.parity(&format!("literal-{i}"), &source);
    }
    let template = include_str!("fixtures/native-inference-long-assert-payload.rs");
    let long="this_is_a_very_long_original_float_variable_name_exceeding_the_pretty_printer_margin";
    let float_source=f.root.join("trailing-dot-float.rs");
    fs::write(&float_source,format!(r#"use std::panic::{{catch_unwind,AssertUnwindSafe}};
fn main(){{std::panic::set_hook(Box::new(|p|println!("payload {{}}",p.payload_as_str().unwrap_or("?"))));
let {long}=1.0f64;let result=catch_unwind(AssertUnwindSafe(||{{assert!(0. .eq(&{long}));}}));assert!(result.is_err(),"must unwind");}}"#)).unwrap();
    f.parity("trailing-dot-float",&float_source);
    for (i,text) in [
        template.replace("println!(\"once {{raw braces}} λ\");", "println!(r#\"raw {{braces}} {}\"#, /* spacing */ 7u8);"),
        template.replace("println!(\"once {{raw braces}} λ\");", "println!(\"{:?}\", [1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 9u8, 10u8, 11u8, 12u8, 13u8, 14u8, 15u8]);"),
        template.replace("fn fail(drops: Rc<Cell<usize>>) {", "fn fail(drops: Rc<Cell<usize>>) { if drops.get() == 0 {").replace("})).is_err());", "})).is_err()); }"),
    ].iter().enumerate() {
        let source=f.root.join(format!("tokens-{i}.rs"));fs::write(&source,text).unwrap();f.parity(&format!("tokens-{i}"),&source);
    }
}

#[test]
#[ignore = "requires the explicitly built original-AST assertion text ABI"]
fn long_assertions_keep_native_borrow_type_const_and_hygiene_refusals() {
    let f = Fixture::new("long-assert-negative");
    let original=include_str!("fixtures/native-inference-long-assert-payload.rs");
    for (name,text) in [
        ("move",original.replace("let _guard = Guard(drops.clone());","let _guard = Guard(drops.clone()); drop(drops); drop(drops);")),
        ("mutate-borrow",original.replace("let _guard = Guard(drops.clone());","let _guard = Guard(drops.clone()); let mut data=vec![1u8]; let live=&data[..]; data.push(2); println!(\"{:?}\",live);")),
        ("not-bool",original.replace("})).is_err());","})));")),
    ] { f.rejected(name,&text); }
    let long="this_is_an_original_long_assert_condition_function_name_with_more_than_seventy_eight_bytes";
    f.rejected("const-false",&format!("const fn {long}()->bool{{false}}const _:()={{assert!({long}());}};fn main(){{}}"));
    let true_source=f.root.join("const-true.rs");fs::write(&true_source,format!("const fn {long}()->bool{{true}}const _:()={{assert!({long}());}};fn main(){{println!(\"constant passes\");}}")).unwrap();f.parity("const-true",&true_source);
    // These valid programs require source identities this extension does not
    // model. They retain L0809, with no artifact or guessed payload.
    for (name,text) in [
        ("struct-field-attribute",format!("struct Holder{{value:bool}}fn main(){{assert!(((Holder{{#[allow(unused)]value:false}}).value.clone().clone().clone().clone().clone().clone().clone().clone().clone().clone()));}}")),
        ("empty-statement",original.replace("let _guard = Guard(drops.clone());","; let _guard = Guard(drops.clone());")),
        ("argument-attribute",format!("fn {long}(x:bool)->bool{{x}}fn main(){{assert!({long}(#[allow(unused)]false));}}")),
        ("hygiene",format!("macro_rules! generated{{($e:expr)=>{{assert!($e)}}}} fn {long}()->bool{{false}}fn main(){{generated!({long}());}}")),
    ] {
        let source=f.root.join(format!("{name}.rs"));fs::write(&source,text).unwrap();let output=f.root.join(name);
        assert!(f.run(&format!("{name}-oracle"),&mut f.oracle(&source,&f.root.join(format!("{name}-oracle-bin")))).status.success());
        let native=f.run(&format!("{name}-native"),&mut f.native(&source,&output));assert_eq!(native.status.code(),Some(1));assert!(String::from_utf8_lossy(&native.stderr).contains("L0809"));assert!(!output.exists());
    }
}

#[test]
#[ignore = "requires the explicitly built place-preserving assertion ABI"]
fn slice_array_places_preserve_full_mixed_source_and_macro_semantics() {
    let f = Fixture::new("slice-array-places");
    for name in ["native-inference-slice-array-place-original", "native-inference-slice-array-place-semantics"] {
        let source = f.source.join("tests/fixtures").join(format!("{name}.rs"));
        let original = fs::read(&source).unwrap();
        let output = f.root.join(name);
        let compiled = f.run(&format!("{name}-build"), &mut f.native(&source, &output));
        assert!(compiled.status.success(), "{}", String::from_utf8_lossy(&compiled.stderr));
        assert!(String::from_utf8_lossy(&compiled.stderr).contains("checked assertion unsizings"));
        assert!(fs::read(&output).unwrap().starts_with(b"\x7fELF"));
        let expected = f.root.join(format!("{name}-oracle"));
        let built = f.run(&format!("{name}-oracle-build"), &mut f.oracle(&source, &expected));
        assert!(built.status.success(), "{}", String::from_utf8_lossy(&built.stderr));
        let actual = f.run(&format!("{name}-run"), &mut Command::new(output));
        let oracle = f.run(&format!("{name}-oracle-run"), &mut Command::new(expected));
        assert!(actual.status.success(), "{}", String::from_utf8_lossy(&actual.stderr));
        assert_eq!(actual.status.code(), oracle.status.code());
        assert_eq!(actual.stdout, oracle.stdout);
        assert_eq!(actual.stderr, oracle.stderr);
        assert_eq!(fs::read(source).unwrap(), original);
    }
}

#[test]
#[ignore = "requires the explicitly built place-preserving assertion ABI"]
fn slice_array_places_preserve_borrows_lifetimes_and_packed_rejection() {
    let f = Fixture::new("slice-array-place-invalid");
    let rejected = |name: &str, text: &str| {
        // Force the same two-round expansion path as the original mixed source;
        // rejection by the unexpanded type mismatch alone is not this proof.
        let text = format!("use std::ffi::OsString;fn proof()->OsString{{let x=\"proof\".into();x}}{}", text.replace("fn main(){", "fn main(){drop(proof());"));
        f.rejected(name, &text);
        let stderr = fs::read_to_string(f.root.join(format!("{name}-native.stderr"))).unwrap();
        assert!(stderr.contains("full native recheck follows"), "{name} did not exercise elaboration: {stderr}");
    };
    rejected("message-place-assignment", r#"fn main(){let x=[1u8];let y=[2u8];let mut a=&x;let s:&[u8]=&x;assert_eq!(s,a,"{}",{a=&y;0});drop(a);}"#);
    rejected("message-reverse-assignment", r#"fn main(){let x=[1u8];let y=[2u8];let mut a=&x;let s:&[u8]=&x;assert_ne!(a,s,"{}",{a=&y;0});drop(a);}"#);
    rejected("message-slice-assignment", r#"fn main(){let x=[1u8];let y=[2u8];let a=&x;let mut s:&[u8]=&x;assert_eq!(s,a,"{}",{s=&y;0});drop(s);}"#);
    rejected("message-width-assignment", r#"fn main(){let x=[1u8];let y=[2u8];let mut a=&x;let s:&[u8]=&x;assert_eq!(s,a,"{value:>width$}",value=1u8,width={a=&y;2usize});drop(a);}"#);
    rejected("indexed-place-assignment", r#"fn main(){let x=[1u8];let y=[1u8];let mut a=[&x];let s:&[u8]=&x;assert_eq!(a[0],{a[0]=&y;s});drop(a);}"#);
    rejected("deref-place-assignment", r#"fn main(){let x=[1u8];let y=[1u8];let mut a=&x;let p=&mut a;let s:&[u8]=&x;assert_eq!(*p,{*p=&y;s});drop(p);}"#);
    rejected("place-expired-owner", r#"fn main(){let a;{let b=[1u8];a=&b;}let s:&[u8]=&[1u8];assert_eq!(s,a);}"#);
    rejected("place-element-mismatch", r#"fn main(){let a=&[1u16];let s:&[u8]=&[1u8];assert_eq!(a,s);}"#);
    rejected("place-wrong-const-length", r#"fn main(){let a:&[u8;2]=&[1u8];let s:&[u8]=&[1u8];assert_eq!(s,a);}"#);
    rejected("packed-place-message", r#"#[repr(packed)]struct P<'a>{pad:u8,a:&'a[u8;1]}fn main(){let p=P{pad:0,a:&[1u8]};let s:&[u8]=&[1u8];assert_eq!(p.a,s,"packed");}"#);
    rejected("packed-tuple-place", r#"#[repr(packed)]struct P<'a>(u8,&'a[u8;1]);fn main(){let p=P(0,&[1u8]);let s:&[u8]=&[1u8];assert_eq!(s,p.1);}"#);
}

#[test]
#[ignore = "requires the explicitly built checked slice/array assertion ABI"]
fn slice_array_assertions_preserve_original_mixed_source_and_runtime_semantics() {
    let f = Fixture::new("slice-array");
    for name in ["native-inference-slice-array-original", "native-inference-slice-array-semantics"] {
        let source = f.source.join("tests/fixtures").join(format!("{name}.rs"));
        let original = fs::read(&source).unwrap();
        let output = f.root.join(name);
        let compiled = f.run(&format!("{name}-build"), &mut f.native(&source, &output));
        assert!(compiled.status.success(), "{}", String::from_utf8_lossy(&compiled.stderr));
        assert!(String::from_utf8_lossy(&compiled.stderr).contains("checked assertion unsizings"));
        assert!(fs::read(&output).unwrap().starts_with(b"\x7fELF"));
        let expected = f.root.join(format!("{name}-oracle"));
        assert!(f.run(&format!("{name}-oracle-build"), &mut f.oracle(&source, &expected)).status.success());
        let actual = f.run(&format!("{name}-run"), &mut Command::new(output));
        let oracle = f.run(&format!("{name}-oracle-run"), &mut Command::new(expected));
        assert!(actual.status.success(), "{}", String::from_utf8_lossy(&actual.stderr));
        assert_eq!(actual.status.code(), oracle.status.code());
        assert_eq!(actual.stdout, oracle.stdout);
        assert_eq!(actual.stderr, oracle.stderr);
        assert_eq!(fs::read(source).unwrap(), original);
    }
}

#[test]
#[ignore = "requires the explicitly built checked slice/array assertion ABI"]
fn slice_array_assertions_preserve_invalid_program_rejection() {
    let f = Fixture::new("slice-array-invalid");
    f.rejected("element-mismatch", r#"fn main(){let s:&[u8]=&[1]; assert_eq!(s,&[1u16]);}"#);
    f.rejected("expired-owner", r#"fn main(){let s:&[u8]; {let a=vec![1u8];s=&a;} assert_eq!(s,&[1u8]);}"#);
    f.rejected("move-live-owner", r#"fn main(){let a=vec![1u8];let s=&a[..];drop(a);assert_eq!(s,&[1u8]);}"#);
    f.rejected("mutate-live-owner", r#"fn main(){let mut a=vec![1u8];let s=&a[..];a.push(2);assert_eq!(s,&[1u8]);}"#);
    f.rejected("alias-args", r#"fn main(){let mut a=vec![1u8];assert_eq!(&a[..],{a.push(2);&[1u8]});}"#);
    f.rejected("lifetime-escape", r#"fn value()->&'static [u8]{let a=[1u8];assert_eq!(&a[..],&[1u8]);&a} fn main(){println!("{:?}",value());}"#);
    f.rejected("no-partial-eq", r#"#[derive(Debug)]struct V;fn main(){assert_eq!(&[V][..],&[V]);}"#);
    f.rejected("no-debug", r#"#[derive(PartialEq)]struct V;fn main(){assert_eq!(&[V][..],&[V]);}"#);
    f.rejected("ambiguous-element", r#"fn main(){assert_eq!(&[][..],&[]);}"#);
    f.rejected("array-length-type", r#"fn main(){let a:&[u8;2]=&[1u8];assert_eq!(&a[..],&[1u8]);}"#);
    f.rejected("reference-place-alias", r#"fn main(){let first=[1u8];let second=[1u8];let mut a=&first;let s:&[u8]=&second;assert_eq!(a,{a=&second;s});drop(a);}"#);
    f.rejected("packed-reference-field", r#"#[repr(packed)]struct P<'a>{pad:u8,a:&'a [u8;1]}fn main(){let a=[1u8];let p=P{pad:0,a:&a};assert_eq!(&[1u8][..],p.a);}"#);
    f.rejected("lookalike-trait", r#"trait PartialEq{fn eq(&self,other:&Self)->bool;}#[derive(Debug)]struct V;impl PartialEq for V{fn eq(&self,_:&Self)->bool{true}}fn main(){assert_eq!(&[V][..],&[V]);}"#);
    f.rejected("packed-tuple-reference-field", r#"use std::ffi::OsString;fn proof()->OsString{let x="proof".into();x}#[repr(packed)]struct P<'a>(u8,&'a [u8;1]);fn main(){drop(proof());let a=[1u8];let p=P(0,&a);assert_eq!(&[1u8][..],p.1);}"#);
    f.rejected("tuple-reference-place-alias", r#"use std::ffi::OsString;fn proof()->OsString{let x="proof".into();x}fn main(){drop(proof());let a=[1u8];let b=[1u8];let mut p=(&a,);let s:&[u8]=&b;assert_eq!(p.0,{p.0=&b;s});drop(p);}"#);
}

struct Fixture {
    root: PathBuf,
    source: PathBuf,
    probe: PathBuf,
    library: PathBuf,
    oracle: PathBuf,
    baseline: PathBuf,
}

impl Fixture {
    fn parity(&self, name: &str, source: &Path) {
        let out=self.root.join(name);let oracle=self.root.join(format!("{name}-oracle"));
        let built=self.run(&format!("{name}-build"),&mut self.native(source,&out));assert!(built.status.success(),"{name}: {}",String::from_utf8_lossy(&built.stderr));
        assert!(fs::read(&out).unwrap().starts_with(b"\x7fELF"));
        assert!(self.run(&format!("{name}-oracle-build"),&mut self.oracle(source,&oracle)).status.success());
        let actual=self.run(&format!("{name}-run"),&mut Command::new(out));let expected=self.run(&format!("{name}-oracle-run"),&mut Command::new(oracle));
        assert!(actual.status.success(),"{name}: {}",String::from_utf8_lossy(&actual.stderr));assert_eq!(actual.status.code(),expected.status.code());assert_eq!(actual.stdout,expected.stdout,"{name}");assert_eq!(actual.stderr,expected.stderr,"{name}");
    }
    fn new(scope: &str) -> Self {
        let root = PathBuf::from(std::env::var_os("LUSTR_PHASE_OUTPUT").expect("LUSTR_PHASE_OUTPUT")).join(scope);
        fs::create_dir_all(root.parent().unwrap()).unwrap();
        fs::create_dir(&root).unwrap();
        Self {
            root,
            source: std::env::current_dir().unwrap(),
            probe: std::env::var_os("LUSTR_PHASE_PROBE").expect("LUSTR_PHASE_PROBE").into(),
            library: std::env::var_os("LUSTR_PHASE_LIBRARY").expect("LUSTR_PHASE_LIBRARY").into(),
            oracle: std::env::var_os("LUSTR_PHASE_ORACLE").expect("LUSTR_PHASE_ORACLE").into(),
            baseline: std::env::var_os("LUSTR_PHASE_BASELINE").expect("LUSTR_PHASE_BASELINE").into(),
        }
    }

    fn run(&self, label: &str, command: &mut Command) -> Output {
        fs::write(self.root.join(format!("{label}.command")), format!("{command:?}\n")).unwrap();
        let result = command.output().unwrap();
        fs::write(self.root.join(format!("{label}.stdout")), &result.stdout).unwrap();
        fs::write(self.root.join(format!("{label}.stderr")), &result.stderr).unwrap();
        fs::write(self.root.join(format!("{label}.status")), format!("{}\n", result.status)).unwrap();
        result
    }

    fn native(&self, source: &Path, output: &Path) -> Command {
        let mut command = Command::new(&self.probe);
        command.arg(&self.library).arg("build").arg(source).arg("-o").arg(output);
        command
    }

    fn oracle(&self, source: &Path, output: &Path) -> Command {
        let mut command = Command::new(&self.oracle);
        command
            .arg(source)
            .args(["--edition=2024", "-Copt-level=0", "-Cdebuginfo=0", "-Cpanic=unwind", "-Coverflow-checks=on", "-Cdebug-assertions=on"])
            .arg("-o")
            .arg(output);
        command
    }

    fn rejected(&self, name: &str, text: &str) {
        let source = self.root.join(format!("{name}.rs"));
        fs::write(&source, text).unwrap();
        let out = self.root.join(name);
        let oracle = self.run(&format!("{name}-oracle"), &mut self.oracle(&source, &out));
        assert_eq!(oracle.status.code(), Some(1), "oracle must reject {name}");
        let candidate = self.run(&format!("{name}-native"), &mut self.native(&source, &out));
        assert_eq!(candidate.status.code(), Some(1), "native must reject {name}: {}", String::from_utf8_lossy(&candidate.stderr));
        assert!(!out.exists(), "rejection published an executable");
    }

    fn cargo_case(&self, name: &str, main: &str, lib: &str, extra: &str, files: &[(&str, &str)], env: &[(&str, std::ffi::OsString)], accepts: bool, oracle_accepts: bool) {
        let root = self.root.join(name);
        fs::create_dir_all(root.join("src")).unwrap();
        let manifest = root.join("Cargo.toml");
        fs::write(&manifest, format!("[package]\nname='env-proof'\nversion='2.3.4'\nedition='2024'\n[workspace]\n[[bin]]\nname='env-bin'\npath='src/main.rs'\n{extra}\n")).unwrap();
        fs::write(root.join("src/main.rs"), main).unwrap();
        if !lib.is_empty() { fs::write(root.join("src/lib.rs"), lib).unwrap(); }
        for (path, text) in files {
            let path = root.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }
        let configure = |c: &mut Command| {
            c.env_remove("CARGO_PKG_VERSION").env_remove("CARGO_CRATE_NAME").env_remove("CARGO_BIN_NAME")
                .env_remove("LUSTR_INFERENCE_ENV");
            for (k, v) in env { c.env(k, v); }
        };
        let cargo = std::env::var_os("LUSTR_PHASE_CARGO").expect("LUSTR_PHASE_CARGO");
        let mut lock = Command::new(&cargo);
        lock.args(["generate-lockfile", "--offline", "--manifest-path"]).arg(&manifest);
        configure(&mut lock);
        assert!(self.run(&format!("{name}-lock"), &mut lock).status.success());
        let mut oracle = Command::new(&cargo);
        let target = root.join("oracle-target");
        oracle.args(["build", "--offline", "--locked", "--jobs", "2", "--bin", "env-bin", "--manifest-path"])
            .arg(&manifest).arg("--config").arg(format!("build.build-dir={:?}", root.join("oracle-build")))
            .env("RUSTC", &self.oracle).env("CARGO_TARGET_DIR", &target);
        configure(&mut oracle);
        let reference = self.run(&format!("{name}-oracle-build"), &mut oracle);
        assert_eq!(reference.status.success(), oracle_accepts, "oracle {name}: {}", String::from_utf8_lossy(&reference.stderr));
        let out = root.join("native");
        let mut native = self.native(&root.join("src/main.rs"), &out);
        native.arg("--manifest-path").arg(&manifest).args(["--bin", "env-bin", "--offline", "--locked"]);
        configure(&mut native);
        let result = self.run(&format!("{name}-native-build"), &mut native);
        assert_eq!(result.status.success(), accepts, "native {name}: {}", String::from_utf8_lossy(&result.stderr));
        if accepts {
            assert!(String::from_utf8_lossy(&result.stderr).contains("constraint-derived conversion sites"));
            assert!(fs::read(&out).unwrap().starts_with(b"\x7fELF"));
            let actual = self.run(&format!("{name}-native-run"), &mut Command::new(out));
            let expected = self.run(&format!("{name}-oracle-run"), &mut Command::new(target.join("debug/env-bin")));
            assert!(actual.status.success());
            assert_eq!(actual.status.code(), expected.status.code());
            assert_eq!(actual.stdout, expected.stdout);
            assert_eq!(actual.stderr, expected.stderr);
        } else {
            assert!(matches!(result.status.code(), Some(1 | 3)), "refusal must remain a compiler failure");
            assert!(!out.exists(), "refused context published an executable");
            assert!(!String::from_utf8_lossy(&result.stderr).contains("constraint-derived conversion sites"), "unsupported context reached elaboration");
        }
        assert_eq!(fs::read_to_string(root.join("src/main.rs")).unwrap(), main);
    }
}

#[test]
#[ignore = "requires the explicitly built native-inference ABI and selected Cargo oracle"]
fn cargo_environment_values_and_unsupported_contexts_are_qualified() {
    use std::os::unix::ffi::OsStringExt;
    let f = Fixture::new("cargo-env");
    let prefix = "use std::ffi::OsString; fn value() -> OsString { let x = \"proof\".into(); x }\n";
    let main = |body: &str| format!("{prefix}fn main() {{ drop(value()); {body} }}\n");
    let shared = main("println!(\"{} / {}\", env!(\"CARGO_PKG_VERSION\"), env_proof::version());");
    let shared_lib = "pub fn version() -> &'static str { env!(\"CARGO_PKG_VERSION\") }";
    f.cargo_case("shared-package", &shared, shared_lib, "", &[], &[], true, true);
    f.cargo_case("conflicting-caller", &shared, shared_lib, "", &[], &[("CARGO_PKG_VERSION", "caller-value".into())], true, true);
    f.cargo_case("different-crate-name", &main("println!(\"{} / {}\", env!(\"CARGO_CRATE_NAME\"), env_proof::name());"),
        "pub fn name() -> &'static str { env!(\"CARGO_CRATE_NAME\") }", "", &[], &[], false, true);
    f.cargo_case("bin-name-absent-in-lib", &main("println!(\"{} / {}\", env!(\"CARGO_BIN_NAME\"), env_proof::name());"),
        "pub fn name() -> &'static str { option_env!(\"CARGO_BIN_NAME\").unwrap_or(\"unset\") }", "", &[], &[], false, true);
    f.cargo_case("ordinary-unset", &main("println!(\"{}\", option_env!(\"LUSTR_INFERENCE_ENV\").unwrap_or(\"unset\"));"), "", "", &[], &[], true, true);
    f.cargo_case("ordinary-set", &main("println!(\"{}\", env!(\"LUSTR_INFERENCE_ENV\"));"), "", "", &[], &[("LUSTR_INFERENCE_ENV", "caller-observed".into())], true, true);
    f.cargo_case("non-unicode", &main("println!(\"{}\", option_env!(\"LUSTR_INFERENCE_ENV\").unwrap_or(\"unset\"));"), "", "", &[],
        &[("LUSTR_INFERENCE_ENV", std::ffi::OsString::from_vec(vec![0xff]))], false, false);
    f.cargo_case("build-script", &shared, shared_lib, "", &[("build.rs", "fn main() {}")], &[], false, true);
    f.cargo_case("cross-package", &main("println!(\"{}\", foreign::version());"), "", "[dependencies]\nforeign={path='foreign'}", &[
        ("foreign/Cargo.toml", "[package]\nname='foreign'\nversion='0.1.0'\nedition='2024'\n"),
        ("foreign/src/lib.rs", "pub fn version() -> &'static str { env!(\"CARGO_PKG_VERSION\") }")], &[], false, true);
    f.cargo_case("position", &main("println!(\"{}\", line!());"), "", "", &[], &[], false, true);
    f.cargo_case("included-file", &main("println!(\"{}\", include_str!(\"message.txt\"));"), "", "", &[("src/message.txt", "observed bytes")], &[], true, true);
}

#[test]
#[ignore = "requires the explicitly built native-inference ABI and matching SDK"]
fn non_unicode_single_file_macro_read_refuses_elaboration() {
    use std::os::unix::ffi::OsStringExt;
    let f = Fixture::new("non-unicode");
    let source = f.root.join("main.rs");
    fs::write(&source, "use std::ffi::OsString; fn value() -> OsString { let x = \"proof\".into(); x } fn main() { drop(value()); println!(\"{}\", option_env!(\"LUSTR_INFERENCE_ENV\").unwrap_or(\"unset\")); }").unwrap();
    let output = f.root.join("native");
    let value = std::ffi::OsString::from_vec(vec![0xff]);
    let oracle = f.run("oracle", f.oracle(&source, &output).env("LUSTR_INFERENCE_ENV", &value));
    assert_eq!(oracle.status.code(), Some(1));
    let native = f.run("native", f.native(&source, &output).env("LUSTR_INFERENCE_ENV", &value));
    assert_eq!(native.status.code(), Some(1));
    assert!(!output.exists());
    assert!(!String::from_utf8_lossy(&native.stderr).contains("constraint-derived conversion sites"));
}

#[test]
#[ignore = "requires the explicitly built native-inference ABI and matching SDK in a bounded sandbox"]
fn real_native_inference_artifacts_and_runtime_match_the_oracle() {
    let f = Fixture::new("valid");
    for name in ["native-inference-extend", "native-inference-into", "native-inference-mixed", "native-inference-pathbuf"] {
        let source = f.source.join("tests/fixtures").join(format!("{name}.rs"));
        let original = fs::read(&source).unwrap();
        let first = f.run(&format!("{name}-pristine"), Command::new(&f.baseline).arg("check").arg(&source));
        assert_eq!(first.status.code(), Some(1));
        let output = f.root.join(name);
        let compiled = f.run(&format!("{name}-build"), &mut f.native(&source, &output));
        assert!(compiled.status.success(), "{}", String::from_utf8_lossy(&compiled.stderr));
        assert!(compiled.stderr.windows(b"constraint-derived conversion sites".len()).any(|w| w == b"constraint-derived conversion sites"));
        assert!(fs::read(&output).unwrap().starts_with(b"\x7fELF"));
        let expected = f.root.join(format!("{name}-oracle"));
        assert!(f.run(&format!("{name}-oracle-build"), &mut f.oracle(&source, &expected)).status.success());
        let native = f.run(&format!("{name}-run"), &mut Command::new(&output));
        let oracle = f.run(&format!("{name}-oracle-run"), &mut Command::new(expected));
        assert!(native.status.success());
        assert_eq!(native.status.code(), oracle.status.code());
        assert_eq!(native.stdout, oracle.stdout);
        assert_eq!(native.stderr, oracle.stderr);
        assert_eq!(fs::read(source).unwrap(), original);
    }
}

#[test]
#[ignore = "requires the explicitly built native-inference ABI and matching SDK"]
fn owned_platform_conversions_match_original_sources_and_mixed_unwind() {
    let f = Fixture::new("owned-platform");
    for name in ["native-inference-owned-path", "native-inference-owned-roundtrip", "native-inference-owned-mixed", "native-inference-owned-same-function"] {
        let source = f.source.join("tests/fixtures").join(format!("{name}.rs"));
        let original = fs::read(&source).unwrap();
        let output = f.root.join(name);
        let compiled = f.run(&format!("{name}-build"), &mut f.native(&source, &output));
        assert!(compiled.status.success(), "{}", String::from_utf8_lossy(&compiled.stderr));
        assert!(String::from_utf8_lossy(&compiled.stderr).contains("constraint-derived conversion sites"));
        assert!(fs::read(&output).unwrap().starts_with(b"\x7fELF"));
        let expected = f.root.join(format!("{name}-oracle"));
        assert!(f.run(&format!("{name}-oracle-build"), &mut f.oracle(&source, &expected)).status.success());
        let native = f.run(&format!("{name}-run"), &mut Command::new(output));
        let oracle = f.run(&format!("{name}-oracle-run"), &mut Command::new(expected));
        assert!(native.status.success());
        assert_eq!(native.status.code(), oracle.status.code());
        assert_eq!(native.stdout, oracle.stdout);
        assert_eq!(native.stderr, oracle.stderr);
        assert_eq!(fs::read(source).unwrap(), original);
    }
}

#[test]
#[ignore = "requires the explicitly built owned PathBuf/OsString native ABI and matching SDK"]
fn owned_platform_conversions_preserve_moves_borrows_mutability_and_identity() {
    let f = Fixture::new("owned-platform-invalid");
    f.rejected("moved-os", r#"use std::ffi::OsString; use std::path::PathBuf;
fn value()->PathBuf {let s=OsString::from("owned");let p=s.into();println!("{s:?}");p} fn main(){drop(value());}"#);
    f.rejected("moved-path", r#"use std::ffi::OsString; use std::path::PathBuf;
fn value()->OsString {let s=PathBuf::from("owned");let p=s.into();println!("{s:?}");p} fn main(){drop(value());}"#);
    f.rejected("borrowed-os", r#"use std::ffi::OsString; use std::path::PathBuf;
fn value()->PathBuf {let s=OsString::from("owned");let borrow=&s;let p=s.into();println!("{borrow:?}");p} fn main(){drop(value());}"#);
    f.rejected("immutable-owned-vector", r#"use std::ffi::OsString; use std::path::PathBuf;
fn value()->Vec<PathBuf> {let p=Vec::new();p.extend([OsString::from("owned").into()]);p} fn main(){drop(value());}"#);
    f.rejected("competing-owned-method", r#"use std::ffi::OsString; use std::path::PathBuf;
trait Other {fn into(self)->PathBuf;} impl Other for OsString {fn into(self)->PathBuf {PathBuf::from(self)}}
fn value()->PathBuf {let s=OsString::from("owned");let p=s.into();p} fn main(){drop(value());}"#);
    f.rejected("custom-owned-extend", r#"use std::ffi::OsString; use std::path::PathBuf;
struct Other; impl From<OsString> for Other {fn from(_:OsString)->Self {Other}}
impl Extend<Other> for Vec<PathBuf> {fn extend<I:IntoIterator<Item=Other>>(&mut self,_:I){}}
fn value()->Vec<PathBuf> {let mut p=Vec::new();p.extend([OsString::from("owned").into()]);p} fn main(){drop(value());}"#);
    f.rejected("shadow-os", r#"use std::path::PathBuf; struct OsString;
fn value()->PathBuf {let s=OsString;let p=s.into();p} fn main(){drop(value());}"#);
    f.rejected("shadow-path", r#"use std::ffi::OsString; struct PathBuf;
fn value()->OsString {let s=PathBuf;let p=s.into();p} fn main(){drop(value());}"#);
    for name in ["competing-owned-method", "custom-owned-extend", "shadow-os", "shadow-path"] {
        let err = fs::read_to_string(f.root.join(format!("{name}-native.stderr"))).unwrap();
        assert!(!err.contains("constraint-derived conversion sites"), "unsupported identity or method reached elaboration: {name}");
    }
}
#[test]
#[ignore = "requires the explicitly built shared Path native inference ABI and matching SDK"]
fn borrowed_path_conversions_match_original_source_aliases_arrays_closures_and_unwind() {
    let f = Fixture::new("borrowed-path");
    for name in ["native-inference-borrowed-path", "native-inference-borrowed-mixed"] {
        let source = f.source.join("tests/fixtures").join(format!("{name}.rs"));
        let original = fs::read(&source).unwrap();
        let output = f.root.join(name);
        let compiled = f.run(&format!("{name}-build"), &mut f.native(&source, &output));
        assert!(compiled.status.success(), "{}", String::from_utf8_lossy(&compiled.stderr));
        assert!(String::from_utf8_lossy(&compiled.stderr).contains("constraint-derived conversion sites"));
        assert!(fs::read(&output).unwrap().starts_with(b"\x7fELF"));
        let expected = f.root.join(format!("{name}-oracle"));
        assert!(f.run(&format!("{name}-oracle-build"), &mut f.oracle(&source, &expected)).status.success());
        let native = f.run(&format!("{name}-run"), &mut Command::new(output));
        let oracle = f.run(&format!("{name}-oracle-run"), &mut Command::new(expected));
        assert!(native.status.success());
        assert_eq!(native.status.code(), oracle.status.code());
        assert_eq!(native.stdout, oracle.stdout);
        assert_eq!(native.stderr, oracle.stderr);
        assert_eq!(fs::read(source).unwrap(), original);
    }
}

#[test]
#[ignore = "requires the explicitly built shared Path native inference ABI and matching SDK"]
fn borrowed_path_conversions_preserve_live_borrows_moves_mutability_and_identity() {
    let f = Fixture::new("borrowed-path-invalid");
    f.rejected("move-borrowed-owner", r#"use std::path::PathBuf;
fn value()->PathBuf {let owner=PathBuf::from("borrowed");let source=owner.as_path();drop(owner);let inferred=source.into();inferred} fn main(){drop(value());}"#);
    f.rejected("mutate-borrowed-owner", r#"use std::path::PathBuf;
fn value()->PathBuf {let mut owner=PathBuf::from("borrowed");let source=owner.as_path();owner.push("changed");let inferred=source.into();inferred} fn main(){drop(value());}"#);
    f.rejected("moved-borrowed-result", r#"use std::path::PathBuf;
fn value()->PathBuf {let owner=PathBuf::from("borrowed");let source=owner.as_path();let inferred=source.into();let checked:PathBuf=inferred;println!("{inferred:?}");checked} fn main(){drop(value());}"#);
    f.rejected("immutable-borrowed-vector", r#"use std::path::PathBuf;
fn value()->Vec<PathBuf> {let owner=PathBuf::from("borrowed");let source=owner.as_path();let result=Vec::new();result.extend([source.into()]);result} fn main(){drop(value());}"#);
    f.rejected("competing-borrowed-method", r#"use std::path::{Path,PathBuf};
trait Other {fn into(self)->PathBuf;} impl Other for &Path {fn into(self)->PathBuf {PathBuf::from(self)}}
fn value()->PathBuf {let owner=PathBuf::from("borrowed");let source=owner.as_path();let inferred=source.into();inferred} fn main(){drop(value());}"#);
    f.rejected("custom-borrowed-extend", r#"use std::path::{Path,PathBuf};
struct Other; impl From<&Path> for Other {fn from(_:&Path)->Self {Other}}
impl Extend<Other> for Vec<PathBuf> {fn extend<I:IntoIterator<Item=Other>>(&mut self,_:I){}}
fn value()->Vec<PathBuf> {let owner=PathBuf::from("borrowed");let source=owner.as_path();let mut result=Vec::new();result.extend([source.into()]);result} fn main(){drop(value());}"#);
    f.rejected("shadow-borrowed-path", r#"use std::path::PathBuf;struct Path;
fn value()->PathBuf {let owner=Path;let source=&owner;let inferred=source.into();inferred} fn main(){drop(value());}"#);
    f.rejected("mutable-path-reference", r#"use std::path::PathBuf;
fn value()->PathBuf {let mut owner=PathBuf::from("borrowed");let source=owner.as_mut_path();let inferred=source.into();inferred} fn main(){drop(value());}"#);
    for name in ["competing-borrowed-method", "custom-borrowed-extend", "shadow-borrowed-path", "mutable-path-reference"] {
        let err = fs::read_to_string(f.root.join(format!("{name}-native.stderr"))).unwrap();
        assert!(!err.contains("constraint-derived conversion sites"), "unsupported identity or method reached elaboration: {name}");
    }
}

#[test]
#[ignore = "requires the explicitly built native-inference ABI and matching SDK"]
fn pathbuf_constraints_keep_ownership_and_ambiguity_rejections() {
    let f = Fixture::new("pathbuf-invalid");
    f.rejected("live-borrow", r#"use std::path::PathBuf;
fn main() { let mut paths=Vec::new(); paths.extend([PathBuf::from("base")]); let first=&paths[0]; paths.extend(["next".into()]); println!("{first:?}"); }"#);
    f.rejected("moved-string", r#"use std::path::PathBuf;
fn path()->PathBuf { let s=String::from("owned"); let p=s.into(); println!("{s}"); p } fn main(){drop(path());}"#);
    f.rejected("immutable-vector", r#"use std::path::PathBuf;
fn paths()->Vec<PathBuf> { let values=Vec::new(); values.extend(["next".into()]); values } fn main(){drop(paths());}"#);
    f.rejected("competing-method", r#"use std::path::PathBuf;
trait Other { fn into(self)->PathBuf; } impl Other for &str { fn into(self)->PathBuf {PathBuf::from(self)} }
fn path()->PathBuf {let p="next".into(); p} fn main(){drop(path());}"#);
    f.rejected("custom-extend", r#"use std::path::PathBuf;
struct Other; impl From<&str> for Other {fn from(_: &str)->Self {Other}}
impl Extend<Other> for Vec<PathBuf> {fn extend<I:IntoIterator<Item=Other>>(&mut self,_:I){}}
fn paths()->Vec<PathBuf> {let mut p=Vec::new();p.extend(["next".into()]);p} fn main(){drop(paths());}"#);
}

#[test]
#[ignore = "requires the explicitly built native-inference ABI and matching SDK in a bounded sandbox"]
fn retained_initial_check_still_requires_the_requested_native_build() {
    let f = Fixture::new("retained-check");
    let source = f.root.join("plain.rs");
    fs::write(&source, "fn main() { println!(\"{}\", 19 + 23); }\n").unwrap();
    let output = f.root.join("plain");
    let mut checking = Command::new(&f.probe);
    checking.arg(&f.library).arg("check").arg(&source).arg("-o").arg(&output);
    let checked = f.run("plain-check", &mut checking);
    assert!(checked.status.success(), "{}", String::from_utf8_lossy(&checked.stderr));
    assert!(!output.exists(), "check unexpectedly produced an executable");
    let built = f.run("plain-build", &mut f.native(&source, &output));
    assert!(built.status.success(), "{}", String::from_utf8_lossy(&built.stderr));
    assert!(!String::from_utf8_lossy(&built.stderr).contains("constraint-derived conversion sites"));
    assert!(fs::read(&output).unwrap().starts_with(b"\x7fELF"));
    let expected = f.root.join("plain-oracle");
    assert!(f.run("plain-oracle-build", &mut f.oracle(&source, &expected)).status.success());
    let native = f.run("plain-run", &mut Command::new(&output));
    let oracle = f.run("plain-oracle-run", &mut Command::new(expected));
    assert_eq!(native.status.code(), oracle.status.code());
    assert_eq!(native.stdout, oracle.stdout);
    assert_eq!(native.stderr, oracle.stderr);
    // This source passes Check. An impossible link destination must still
    // fail at Build, through both unchanged Lust and the retained entry.
    let destination = f.root.join("existing-directory");
    fs::create_dir(&destination).unwrap();
    let baseline = f.run("plain-bad-output-pristine", Command::new(&f.baseline).arg("build").arg(&source).arg("-o").arg(&destination));
    let candidate = f.run("plain-bad-output-retained", &mut f.native(&source, &destination));
    assert!(!baseline.status.success());
    assert_eq!(candidate.status.code(), baseline.status.code());
    assert!(destination.is_dir());
    let foreign_source = f.root.join("foreign-invalid.rs");
    fs::write(&foreign_source, "fn main() { missing(); }\n").unwrap();
    let foreign = "aarch64-unknown-linux-gnu";
    let baseline = f.run("foreign-pristine", Command::new(&f.baseline).arg("build").arg(&foreign_source)
        .arg("--target").arg(foreign).arg("-o").arg(f.root.join("foreign-baseline")));
    let candidate = f.run("foreign-retained", f.native(&foreign_source, &f.root.join("foreign-candidate"))
        .arg("--target").arg(foreign));
    assert_eq!(baseline.status.code(), Some(2));
    assert_eq!(candidate.status.code(), baseline.status.code());
    assert_eq!(candidate.stderr, baseline.stderr);
    assert!(!f.root.join("foreign-candidate").exists());
    let mut checking = Command::new(&f.probe);
    checking.arg(&f.library).arg("check").arg(f.source.join("tests/fixtures/native-inference-mixed.rs"))
        .arg("-o").arg(f.root.join("check-mixed"));
    let checked = f.run("mixed-check", &mut checking);
    assert!(checked.status.success(), "{}", String::from_utf8_lossy(&checked.stderr));
    assert!(String::from_utf8_lossy(&checked.stderr).contains("constraint-derived conversion sites"));
    assert!(!f.root.join("check-mixed").exists());
}

#[test]
#[ignore = "requires the explicitly built native-inference ABI and matching SDK in a bounded sandbox"]
fn invalid_and_ambiguous_programs_remain_rejected() {
    let f = Fixture::new("invalid");
    f.rejected("unknown-target", r#"fn main() { let _x = "unknown".into(); }"#);
    f.rejected("unknown-item", r#"fn main() { let mut x = Vec::new(); x.extend(["unknown".into()]); }"#);
    f.rejected(
        "immutable",
        r#"use std::ffi::OsString;
fn args() -> Vec<OsString> { let args = Vec::new(); args.extend(["one".into(), format!("two").into()]); args }
fn main() { drop(args()); }"#,
    );
    f.rejected("live-borrow", r#"use std::ffi::OsString;
fn main() { let mut args = Vec::new(); args.extend([OsString::from("base")]); let first = &args[0]; args.extend(["one".into(), format!("two").into()]); println!("{first:?}"); }"#);
    f.rejected(
        "moved-string",
        r#"use std::ffi::OsString;
fn args() -> Vec<OsString> { let s = String::from("owned"); let x = s.into(); println!("{s}"); vec![x] }
fn main() { drop(args()); }"#,
    );
    f.rejected(
        "competing-into",
        r#"use std::ffi::OsString;
trait Other { fn into(self) -> OsString; }
impl Other for &str { fn into(self) -> OsString { OsString::from(self) } }
fn value() -> OsString { let x = "one".into(); x }
fn main() { drop(value()); }"#,
    );
    f.rejected(
        "custom-extend",
        r#"use std::ffi::OsString;
struct Other;
impl From<&str> for Other { fn from(_: &str) -> Self { Other } }
impl Extend<Other> for Vec<OsString> { fn extend<I: IntoIterator<Item=Other>>(&mut self, _: I) {} }
fn args() -> Vec<OsString> { let mut x = Vec::new(); x.extend(["one".into()]); x }
fn main() { drop(args()); }"#,
    );
}

#[test]
#[ignore = "requires the explicitly built native-inference ABI and matching SDK in a bounded sandbox"]
fn unchanged_full_coordinator_compiles_natively_and_keeps_cli_behavior() {
    let f = Fixture::new("coordinator");
    let source = f.source.join("crates/lustr/src/main.rs");
    let manifest = f.source.join("crates/lustr/Cargo.toml");
    let output = f.root.join("cargo-lust-native");
    let compiled = f.run(
        "native-build",
        f.native(&source, &output).arg("--manifest-path").arg(&manifest).args(["--bin", "cargo-lust", "--locked", "--offline"]),
    );
    assert!(compiled.status.success(), "{}", String::from_utf8_lossy(&compiled.stderr));
    assert!(fs::read(&output).unwrap().starts_with(b"\x7fELF"));
    let target = f.root.join("oracle-target");
    let cargo = PathBuf::from(std::env::var_os("LUSTR_PHASE_CARGO").expect("LUSTR_PHASE_CARGO"));
    let built = f.run(
        "oracle-build",
        Command::new(cargo)
            .args([
                "build",
                "--config",
                "build.build-dir=\"/build/cargo-build-oracle\"",
                "--locked",
                "--offline",
                "--jobs",
                "2",
                "--manifest-path",
            ])
            .arg(&manifest)
            .args(["--bin", "cargo-lust"])
            .env("RUSTC", &f.oracle)
            .env("CARGO_TARGET_DIR", &target),
    );
    assert!(built.status.success(), "{}", String::from_utf8_lossy(&built.stderr));
    let oracle = target.join("debug/cargo-lust");
    for (i, args) in [
        vec!["--help"],
        vec!["--version"],
        vec!["plan", "--frontend-jobs=3", "rustc", "--offline"],
        vec!["plan", "--bluefe-root", "/package", "build", "--offline"],
        vec!["--frontend-jobs=0", "rustc"],
    ]
    .iter()
    .enumerate()
    {
        let native = f.run(&format!("cli-{i}-native"), Command::new(&output).args(args));
        let reference = f.run(&format!("cli-{i}-oracle"), Command::new(&oracle).args(args));
        assert_eq!(native.status.code(), reference.status.code());
        assert_eq!(native.stdout, reference.stdout);
        assert_eq!(native.stderr, reference.stderr);
    }
}

#[test]
#[ignore = "requires the separately assembled experimental product candidate"]
fn product_entry_uses_native_inference_for_the_unchanged_coordinator_and_mixed_fixture() {
    let f = Fixture::new("product");
    let product = PathBuf::from(std::env::var_os("LUSTR_PHASE_PRODUCT").expect("LUSTR_PHASE_PRODUCT"));
    for (label, source, manifest) in [
        ("coordinator", f.source.join("crates/lustr/src/main.rs"), Some(f.source.join("crates/lustr/Cargo.toml"))),
        ("mixed", f.source.join("tests/fixtures/native-inference-mixed.rs"), None),
    ] {
        let original = fs::read(&source).unwrap();
        let output = f.root.join(label);
        let trace = f.root.join(format!("{label}.trace"));
        let mut command = Command::new(&product);
        command.arg("build").arg(&source).arg("-o").arg(&output)
            .env("LUST_NO_DAEMON", "1").env_remove("LUST_LIBRARY").env("LUSTR_TRACE_PATH", &trace);
        if let Some(manifest) = &manifest { command.arg("--manifest-path").arg(manifest).args(["--bin", "cargo-lust", "--offline", "--locked"]); }
        let result = f.run(&format!("{label}-build"), &mut command);
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
        let trace = fs::read_to_string(trace).unwrap();
        assert!(trace.lines().last().unwrap().contains("\"engine\":\"lustr-native\""), "{trace}");
        assert!(trace.lines().last().unwrap().contains("\"entry\":\"native-inference-worker\""), "{trace}");
        assert!(!trace.contains("full-rust-core"), "unexpected compatibility compilation: {trace}");
        assert!(fs::read(&output).unwrap().starts_with(b"\x7fELF"));
        let oracle = if let Some(manifest)=&manifest {
            let target=f.root.join("oracle-target");
            let cargo=std::env::var_os("LUSTR_PHASE_CARGO").expect("LUSTR_PHASE_CARGO");
            let built=f.run("coordinator-oracle-build",Command::new(cargo).args(["build","--locked","--offline","--jobs","2","--manifest-path"])
                .arg(manifest).args(["--bin","cargo-lust"]).arg("--config").arg(format!("build.build-dir={:?}",f.root.join("oracle-build")))
                .env("RUSTC",&f.oracle).env("CARGO_TARGET_DIR",&target));
            assert!(built.status.success(),"{}",String::from_utf8_lossy(&built.stderr));
            target.join("debug/cargo-lust")
        } else {
            let oracle=f.root.join("mixed-oracle");
            assert!(f.run("mixed-oracle-build",&mut f.oracle(&source,&oracle)).status.success());
            oracle
        };
        let cases=if manifest.is_some() {vec![vec!["--help"],vec!["--version"],vec!["plan","--frontend-jobs=3","rustc","--offline"],
            vec!["plan","--bluefe-root","/package","build","--offline"],vec!["--frontend-jobs=0","rustc"]]} else {vec![vec![]]};
        for (i,args) in cases.iter().enumerate() {
            let actual=f.run(&format!("{label}-{i}-run"),Command::new(&output).args(args));
            let expected=f.run(&format!("{label}-{i}-oracle-run"),Command::new(&oracle).args(args));
            assert_eq!(actual.status.code(),expected.status.code());
            assert_eq!(actual.stdout,expected.stdout);
            assert_eq!(actual.stderr,expected.stderr);
        }
        assert_eq!(fs::read(source).unwrap(), original);
    }
}

#[test]
#[ignore = "requires the separately assembled experimental product candidate"]
fn product_preserves_nondefault_thp_policy_and_full_core_route() {
    use std::os::unix::process::CommandExt;
    unsafe extern "C" { fn prctl(option: i32, ...) -> i32; }
    let f = Fixture::new("product-thp");
    let product = PathBuf::from(std::env::var_os("LUSTR_PHASE_PRODUCT").expect("LUSTR_PHASE_PRODUCT"));
    let source = f.source.join("tests/fixtures/native-inference-into.rs");
    let output = f.root.join("native");
    let trace = f.root.join("trace.jsonl");
    let mut command = Command::new(product);
    command.arg("build").arg(&source).arg("-o").arg(&output).env("LUST_NO_DAEMON", "1")
        .env_remove("LUST_LIBRARY").env("LUSTR_TRACE_PATH", &trace);
    unsafe {
        command.pre_exec(|| {
            if prctl(41, 1u64, 0u64, 0u64, 0u64) != 0 { return Err(std::io::Error::last_os_error()); }
            if prctl(42, 0u64, 0u64, 0u64, 0u64) != 1 { return Err(std::io::Error::other("THP setting did not apply")); }
            Ok(())
        });
    }
    let built = f.run("policy-build", &mut command);
    assert!(built.status.success(), "{}", String::from_utf8_lossy(&built.stderr));
    let trace = fs::read_to_string(trace).unwrap();
    assert!(trace.lines().last().unwrap().contains("full-rust-core"), "{trace}");
    assert!(!trace.contains("native-inference-worker"), "{trace}");
    let expected = f.root.join("oracle");
    assert!(f.run("oracle-build", &mut f.oracle(&source, &expected)).status.success());
    let actual = f.run("policy-run", &mut Command::new(output));
    let oracle = f.run("oracle-run", &mut Command::new(expected));
    assert!(actual.status.success());
    assert_eq!(actual.stdout, oracle.stdout);
    assert_eq!(actual.stderr, oracle.stderr);
}

#[test]
#[ignore = "requires the separately assembled experimental product candidate"]
fn product_does_not_hide_native_worker_failures_or_signals() {
    use std::os::unix::{fs::symlink, process::ExitStatusExt};
    let f = Fixture::new("product-failures");
    let product = PathBuf::from(std::env::var_os("LUSTR_PHASE_PRODUCT").expect("LUSTR_PHASE_PRODUCT"));
    let package = product.parent().unwrap().parent().unwrap();
    let runtime = f.root.join("runtime");
    fs::create_dir_all(runtime.join("bin")).unwrap();
    fs::copy(&product, runtime.join("bin/lustr")).unwrap();
    symlink(package.join("lib"), runtime.join("lib")).unwrap();
    for entry in fs::read_dir(package.join("bin")).unwrap() {
        let entry=entry.unwrap();
        if entry.file_name()!="lustr" && entry.file_name()!="lustr-native-worker" {
            symlink(entry.path(), runtime.join("bin").join(entry.file_name())).unwrap();
        }
    }
    let source=f.source.join("tests/fixtures/native-inference-into.rs");
    for (label, text, code, signal) in [
        ("status3", "fn main() { std::process::exit(3); }", Some(3), None),
        ("signal", "unsafe extern \"C\" { fn raise(n:i32)->i32; } fn main() { unsafe { raise(15); } std::process::exit(3); }", None, Some(15)),
    ] {
        let helper=f.root.join(format!("{label}.rs")); fs::write(&helper,text).unwrap();
        let worker=runtime.join("bin/lustr-native-worker");
        assert!(f.run(&format!("{label}-helper"),&mut f.oracle(&helper,&worker)).status.success());
        let trace=f.root.join(format!("{label}.trace"));
        let output=f.root.join(format!("{label}-artifact"));
        let result=f.run(label,Command::new(runtime.join("bin/lustr")).arg("build").arg(&source).arg("-o").arg(&output)
            .env("LUST_NO_DAEMON","1").env_remove("LUST_LIBRARY").env("LUSTR_TRACE_PATH",&trace));
        assert_eq!(result.status.code(),code);
        assert_eq!(result.status.signal(),signal);
        assert!(!output.exists());
        assert!(!fs::read_to_string(trace).unwrap().contains("full-rust-core"));
    }
}
#[test]
#[ignore = "requires the explicitly built shared OsStr native inference ABI and matching SDK"]
fn borrowed_os_conversions_match_original_source_aliases_arrays_closures_and_unwind() {
    let f = Fixture::new("borrowed-os");
    for name in ["native-inference-borrowed-os", "native-inference-borrowed-os-mixed"] {
        let source = f.source.join("tests/fixtures").join(format!("{name}.rs"));
        let original = fs::read(&source).unwrap();
        let output = f.root.join(name);
        let compiled = f.run(&format!("{name}-build"), &mut f.native(&source, &output));
        assert!(compiled.status.success(), "{}", String::from_utf8_lossy(&compiled.stderr));
        assert!(String::from_utf8_lossy(&compiled.stderr).contains("constraint-derived conversion sites"));
        assert!(fs::read(&output).unwrap().starts_with(b"\x7fELF"));
        let expected = f.root.join(format!("{name}-oracle"));
        assert!(f.run(&format!("{name}-oracle-build"), &mut f.oracle(&source, &expected)).status.success());
        let native = f.run(&format!("{name}-run"), &mut Command::new(output));
        let oracle = f.run(&format!("{name}-oracle-run"), &mut Command::new(expected));
        assert!(native.status.success());
        assert_eq!(native.status.code(), oracle.status.code());
        assert_eq!(native.stdout, oracle.stdout);
        assert_eq!(native.stderr, oracle.stderr);
        assert_eq!(fs::read(source).unwrap(), original);
    }
}

#[test]
#[ignore = "requires the explicitly built shared OsStr native inference ABI and matching SDK"]
fn borrowed_os_conversions_preserve_live_borrows_moves_mutability_and_identity() {
    let f = Fixture::new("borrowed-os-invalid");
    f.rejected("expired-borrowed-owner", r#"use std::ffi::OsString; use std::path::PathBuf;
fn value()->PathBuf {let source;{let owner=OsString::from("borrowed");source=owner.as_os_str();}let inferred=source.into();inferred} fn main(){drop(value());}"#);
    f.rejected("move-borrowed-owner", r#"use std::ffi::OsString; use std::path::PathBuf;
fn value()->PathBuf {let owner=OsString::from("borrowed");let source=owner.as_os_str();drop(owner);let inferred=source.into();inferred} fn main(){drop(value());}"#);
    f.rejected("mutate-borrowed-owner", r#"use std::ffi::OsString; use std::path::PathBuf;
fn value()->PathBuf {let mut owner=OsString::from("borrowed");let source=owner.as_os_str();owner.push("changed");let inferred=source.into();inferred} fn main(){drop(value());}"#);
    f.rejected("moved-borrowed-result", r#"use std::ffi::OsString; use std::path::PathBuf;
fn value()->PathBuf {let owner=OsString::from("borrowed");let source=owner.as_os_str();let inferred=source.into();let checked:PathBuf=inferred;println!("{inferred:?}");checked} fn main(){drop(value());}"#);
    f.rejected("immutable-borrowed-vector", r#"use std::ffi::OsString; use std::path::PathBuf;
fn value()->Vec<PathBuf> {let owner=OsString::from("borrowed");let source=owner.as_os_str();let result=Vec::new();result.extend([source.into()]);result} fn main(){drop(value());}"#);
    f.rejected("competing-borrowed-method", r#"use std::ffi::{OsStr,OsString}; use std::path::PathBuf;
trait Other {fn into(self)->PathBuf;} impl Other for &OsStr {fn into(self)->PathBuf {PathBuf::from(self)}}
fn value()->PathBuf {let owner=OsString::from("borrowed");let source=owner.as_os_str();let inferred=source.into();inferred} fn main(){drop(value());}"#);
    f.rejected("custom-borrowed-extend", r#"use std::ffi::{OsStr,OsString}; use std::path::PathBuf;
struct Other; impl From<&OsStr> for Other {fn from(_:&OsStr)->Self {Other}}
impl Extend<Other> for Vec<PathBuf> {fn extend<I:IntoIterator<Item=Other>>(&mut self,_:I){}}
fn value()->Vec<PathBuf> {let owner=OsString::from("borrowed");let source=owner.as_os_str();let mut result=Vec::new();result.extend([source.into()]);result} fn main(){drop(value());}"#);
    f.rejected("shadow-borrowed-os", r#"use std::ffi::OsString; use std::path::PathBuf;struct OsStr;
fn value()->PathBuf {let owner=OsStr;let source=&owner;let inferred=source.into();inferred} fn main(){drop(value());}"#);
    f.rejected("mutable-os-reference", r#"use std::ffi::OsString; use std::path::PathBuf;
fn value()->PathBuf {let mut owner=OsString::from("borrowed");let source: &mut std::ffi::OsStr=&mut owner;let inferred=source.into();inferred} fn main(){drop(value());}"#);
    for name in ["competing-borrowed-method", "custom-borrowed-extend", "shadow-borrowed-os", "mutable-os-reference"] {
        let err = fs::read_to_string(f.root.join(format!("{name}-native.stderr"))).unwrap();
        assert!(!err.contains("constraint-derived conversion sites"), "unsupported identity or method reached elaboration: {name}");
    }
}
