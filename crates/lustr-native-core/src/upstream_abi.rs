//! Integer ABI adapters emitted by the matching imported LLVM implementation.
//!
//! The outer signature is the admitted native C ABI. The inner call preserves
//! the original Rust instance's ABI attributes. In particular, integer
//! extension is never erased merely because both conventions use a register.
use super::{Error, Value, missing};
use rustc_target::callconv::{ArgAttribute, ArgAttributes, ArgExtension, CastTarget, PassMode};
use std::{collections::BTreeSet, io::Write, path::PathBuf, process::{Command, Stdio}, sync::{Arc, OnceLock}};

fn attributes(mode: &PassMode, value: &Value) -> Result<String, Error> {
    let attributes = match mode {
        PassMode::Direct(attributes) => attributes,
        PassMode::Cast { cast, pad_i32_count: 0 } => {
            let expected = CastTarget::from(rustc_abi::Reg { kind: rustc_abi::RegKind::Integer,
                size: rustc_abi::Size::from_bytes(value.size) }).with_attrs(cast.attrs);
            if expected != **cast { return Err(missing("integer adapter needs the exact native single-integer C aggregate cast")); }
            &cast.attrs
        }
        _ => return Err(missing("integer adapter needs represented direct or single-integer cast modes")),
    };
    // Every admitted field is represented, including absence of pointer facts.
    // Additional attributes require an explicit encoder, never Debug parsing.
    let mut represented = ArgAttributes::new();
    represented.arg_ext = attributes.arg_ext;
    if attributes.regular.contains(ArgAttribute::NoUndef) { represented.regular |= ArgAttribute::NoUndef; }
    if represented != *attributes {
        return Err(missing("integer adapter needs additional upstream argument attributes"));
    }
    let extension = match attributes.arg_ext {
        ArgExtension::None => "", ArgExtension::Zext => "zeroext ", ArgExtension::Sext => "signext ",
    };
    Ok(format!("{extension}{}", if attributes.regular.contains(ArgAttribute::NoUndef) { "noundef " } else { "" }))
}

pub(super) struct Function {
    pub symbol: String,
    original: String,
    ir: String,
}

impl Function {
    pub(super) fn capture(symbol: String, original: &str, inputs: &[Value], output: &Value,
        rust_inputs: &[PassMode], rust_output: &PassMode, c_inputs: &[PassMode], c_output: &PassMode) -> Result<Self, Error> {
        if inputs.len() != rust_inputs.len() || inputs.len() != c_inputs.len() {
            return Err(Error::Failed("integer adapter argument inventory changed".into()));
        }
        let result = format!("i{}", output.size * 8);
        let original_params = inputs.iter().zip(rust_inputs).map(|(value, mode)|
            Ok(format!("i{} {}", value.size * 8, attributes(mode, value)?))).collect::<Result<Vec<_>, Error>>()?.join(", ");
        let wrapper_params = inputs.iter().zip(c_inputs).enumerate().map(|(index, (value, mode))|
            Ok(format!("i{} {}%a{index}", value.size * 8, attributes(mode, value)?))).collect::<Result<Vec<_>, Error>>()?.join(", ");
        let call_params = inputs.iter().zip(rust_inputs).enumerate().map(|(index, (value, mode))|
            Ok(format!("i{} {}%a{index}", value.size * 8, attributes(mode, value)?))).collect::<Result<Vec<_>, Error>>()?.join(", ");
        let rust_return = attributes(rust_output, output)?;
        let c_return = attributes(c_output, output)?;
        let ir = format!("declare {rust_return}{result} @{original}({original_params}) nounwind\n\
            define {c_return}{result} @{symbol}({wrapper_params}) nounwind {{\n\
            entry:\n  %value = call {rust_return}{result} @{original}({call_params})\n  ret {result} %value\n}}\n");
        Ok(Self { symbol, original: original.into(), ir })
    }
}

/// Only the qualification executable constructs exceptional invocations. The
/// ordinary provider uses Normal and the declared SDK path, never PATH lookup.
#[derive(Clone, Copy, Default)]
pub(crate) enum Control { #[default] Normal, MissingTool, FailedTool, MalformedObject, EmptyObject }

pub(super) struct Module {
    tool: PathBuf,
    library: PathBuf,
    identity: Arc<[u8]>,
    ir: Arc<[u8]>,
    definitions: BTreeSet<Vec<u8>>,
    imports: BTreeSet<Vec<u8>>,
    object: OnceLock<Arc<[u8]>>,
    control: Control,
}

fn fingerprint(path: &std::path::Path) -> Result<String, String> {
    let result = Command::new("/usr/bin/sha256sum").arg("--").arg(path).output()
        .map_err(|e| format!("identify imported LLVM tool: {e}"))?;
    if !result.status.success() { return Err(format!("identify imported LLVM tool: {}", String::from_utf8_lossy(&result.stderr))); }
    let hash = result.stdout.get(..64).filter(|v| v.iter().all(|b| b.is_ascii_hexdigit()))
        .ok_or("invalid imported LLVM fingerprint")?;
    Ok(std::str::from_utf8(hash).unwrap().into())
}

fn pin(name: &str) -> Result<&'static str, String> {
    let prefix = format!("{name} = \"");
    include_str!("../../../upstream/bluefe-core.pin").lines().find_map(|line|
        line.strip_prefix(&prefix).and_then(|value| value.strip_suffix('"')))
        .ok_or_else(|| format!("missing declared LLVM tool identity: {name}"))
}

impl Module {
    pub(super) fn capture(functions: &[Function], control: Control) -> Result<Option<Self>, Error> {
        if functions.is_empty() { return Ok(None); }
        let root = PathBuf::from(env!("LUSTR_IMPORTED_SYSROOT"));
        if pin("source").map_err(Error::Failed)? != env!("LUSTR_IMPORTED_COMMIT") {
            return Err(Error::Failed("LLVM adapter SDK declaration differs from the imported compiler".into()));
        }
        let tool = root.join("lib/rustlib/x86_64-unknown-linux-gnu/bin").join(pin("llc_file").map_err(Error::Failed)?);
        // Hash the actual NEEDED DSO, not the similarly named linker script.
        let library = root.join("lib").join(pin("llvm_file").map_err(Error::Failed)?);
        let mut module = Self { tool, library, identity: Arc::from([]), ir: Arc::from([]),
            definitions: BTreeSet::new(), imports: BTreeSet::new(), object: OnceLock::new(), control };
        module.identity = module.read_identity().map_err(Error::Failed)?.into();
        let mut ir = String::from("target triple = \"x86_64-unknown-linux-gnu\"\n");
        for function in functions {
            if !module.definitions.insert(function.symbol.as_bytes().to_vec())
                || !module.imports.insert(function.original.as_bytes().to_vec()) {
                return Err(Error::Failed("integer adapter definition ownership conflicts".into()));
            }
            ir.push_str(&function.ir);
        }
        module.ir = ir.into_bytes().into();
        Ok(Some(module))
    }

    fn read_identity(&self) -> Result<Vec<u8>, String> {
        let tool = fingerprint(&self.tool)?;
        let library = fingerprint(&self.library)?;
        if tool != pin("llc_sha256")? || library != pin("llvm_sha256")? {
            return Err("LLVM adapter tools do not match the declared SDK identities".into());
        }
        Ok(format!("rust={}\nllc={}\nLLVM={}\n", env!("LUSTR_IMPORTED_COMMIT"),
            tool, library).into_bytes())
    }

    pub(super) fn dependencies(&self) -> [(String, Arc<[u8]>); 2] {
        [("imported-LLVM-adapter-tool".into(), self.identity.clone()), ("imported-LLVM-adapter-IR".into(), self.ir.clone())]
    }

    /// Called only after the installed semantic records and checked native
    /// bodies pass validation. Failure is terminal and publishes no artifact.
    pub(super) fn emit(&self) -> Result<(), String> {
        if self.object.get().is_some() { return Err("integer adapter emission already consumed".into()); }
        if self.read_identity()?.as_slice() != self.identity.as_ref() { return Err("imported LLVM tool generation changed".into()); }
        let path = if matches!(self.control, Control::MissingTool) { self.tool.join("missing-qualification-tool") } else { self.tool.clone() };
        let mut command = Command::new(path);
        command.args(["-O=0", "-mtriple=x86_64-unknown-linux-gnu", "-mcpu=x86-64", "-relocation-model=pic", "-o"])
            .arg(if matches!(self.control, Control::EmptyObject) { "/dev/null" } else { "-" })
            .arg(if matches!(self.control, Control::MalformedObject) { "-filetype=asm" } else { "-filetype=obj" })
            .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
            .env_remove("LD_LIBRARY_PATH").env_remove("LD_PRELOAD");
        if matches!(self.control, Control::FailedTool) { command.arg("--invalid-lustr-qualification-option"); }
        let mut child = command.spawn().map_err(|e| format!("start imported LLVM adapter emission: {e}"))?;
        let written = child.stdin.take().ok_or("missing LLVM adapter input pipe")?.write_all(&self.ir);
        let output = child.wait_with_output().map_err(|e| format!("wait for imported LLVM adapter emission: {e}"))?;
        if !output.status.success() { return Err(format!("imported LLVM adapter emission failed: {}", String::from_utf8_lossy(&output.stderr))); }
        written.map_err(|e| format!("write imported LLVM adapter input: {e}"))?;
        if self.read_identity()?.as_slice() != self.identity.as_ref() { return Err("imported LLVM tool changed during emission".into()); }
        let symbols = crate::unit_object::symbols(&output.stdout).map_err(|e| format!("invalid imported LLVM adapter object: {e}"))?;
        if symbols.defined != self.definitions || symbols.functions != self.definitions || symbols.undefined != self.imports {
            return Err("imported LLVM adapter object disagrees with its admitted symbols".into());
        }
        self.object.set(output.stdout.into()).map_err(|_| "integer adapter emission generation changed".into())
    }

    pub(super) fn object(&self) -> Arc<[u8]> { self.object.get().expect("validated adapter object precedes ownership transfer").clone() }
    pub(super) fn count(&self) -> usize { self.definitions.len() }
}
