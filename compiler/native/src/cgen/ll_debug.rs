// Rust source debug records for the LLVM emitter. The native checker and
// lowerer supply bindings and types; LLVM resolves instruction and frame
// locations to the addresses in the emitted object.
use super::{FnGen, LocalDecl, Span, Ty, TyKind};
use crate::ir::{Local, Terminator};
use crate::ty::{FloatTy, Mut, TyIntern};
use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;

pub(super) struct DebugFn {
    entries: Vec<String>,
    files: BTreeMap<String, usize>,
    types: HashMap<Ty, (usize, String)>,
    locations: BTreeMap<(usize, u32, u32), usize>,
    scope_files: BTreeMap<(usize, usize), usize>,
    subprogram: usize,
    active: Vec<(Local, usize)>,
    bindings: BTreeMap<Local, (usize, u32, u32, usize, String)>,
}

impl DebugFn {
    pub(super) fn new(function: &FnGen<'_, '_>, linkage: &str) -> Result<Self, String> {
        let mut debug = Self {
            entries: vec![String::new()],
            files: BTreeMap::new(),
            types: HashMap::new(),
            locations: BTreeMap::new(),
            scope_files: BTreeMap::new(),
            subprogram: 0,
            active: Vec::new(),
            bindings: BTreeMap::new(),
        };
        let def = function.p.fn_def(function.body.fn_id);
        let (file, line, _) = debug.source(function, def.name_span).ok_or("Rust debug function has no source location")?;
        debug.entries[0] = format!(
            "distinct !DICompileUnit(language: DW_LANG_Rust, file: !%{file}, producer: \"Lustr\", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug)"
        );
        let name = function.p.display_path(function.p.fn_info(function.body.fn_id).name);
        let mut types = Vec::new();
        for local in &function.body.locals[..=function.body.arg_count] {
            let ty = if matches!(function.body.tys.kind(local.ty), TyKind::Unit | TyKind::Never) {
                "null".into()
            } else {
                format!("!%{}", debug.ty(function, local.ty)?)
            };
            types.push(ty);
        }
        let signature = debug.push(format!("!DISubroutineType(types: !{{{}}})", types.join(", ")));
        debug.subprogram = debug.push(format!(
            "distinct !DISubprogram(name: {}, linkageName: {}, scope: !%{file}, file: !%{file}, line: {line}, type: !%{signature}, scopeLine: {line}, spFlags: DISPFlagDefinition, unit: !%0)",
            quote(&name), quote(linkage)
        ));
        for (i, local) in function.body.locals.iter().enumerate().skip(1) {
            let LocalDecl { name: Some(name), binding_span: Some(span), ty, .. } = local else { continue };
            let (file, line, column) = debug.source(function, *span).ok_or("Rust debug binding has no source location")?;
            let ty = debug.ty(function, *ty)?;
            debug.bindings.insert(i as Local, (file, line, column, ty, quote(name)));
        }
        // The scalar path models initialized lexical bindings by their actual
        // stores and StorageDead markers. A branching body with user locals
        // needs a control-flow join of those states before it can use this path.
        if debug.bindings.keys().any(|&i| i as usize > function.body.arg_count)
            && function.body.blocks.iter().any(|b| matches!(b.term, Terminator::If(..) | Terminator::Switch(..)))
        {
            return Err("Rust debug locations for local bindings in branching bodies are not represented".into());
        }
        Ok(debug)
    }

    fn push(&mut self, entry: String) -> usize {
        let index = self.entries.len();
        self.entries.push(entry);
        index
    }

    fn ty(&mut self, function: &FnGen<'_, '_>, ty: Ty) -> Result<usize, String> {
        if let Some(&(id, _)) = self.types.get(&ty) {
            return Ok(id);
        }
        let (name, entry) = match function.body.tys.kind(ty) {
            TyKind::Int(int) => {
                let encoding = if int.signed() { "DW_ATE_signed" } else { "DW_ATE_unsigned" };
                basic_type(int.name(), int.bits(), encoding)
            }
            TyKind::Bool => basic_type("bool", 8, "DW_ATE_boolean"),
            TyKind::Char => basic_type("char", 32, "DW_ATE_UTF"),
            TyKind::Float(float) => basic_type(float.name(), if float == FloatTy::F32 { 32 } else { 64 }, "DW_ATE_float"),
            TyKind::Ref(_, m, base) | TyKind::RawPtr(m, base) => {
                let target = self.ty(function, base)?;
                let prefix = match (function.body.tys.kind(ty), m) {
                    (TyKind::Ref(..), Mut::Shared) => "&",
                    (TyKind::Ref(..), Mut::Mut) => "&mut ",
                    (_, Mut::Shared) => "*const ",
                    (_, Mut::Mut) => "*mut ",
                };
                let name = format!("{prefix}{}", self.types[&base].1);
                let entry = format!("!DIDerivedType(tag: DW_TAG_pointer_type, name: {}, baseType: !%{target}, size: 64)", quote(&name));
                (name, entry)
            }
            TyKind::Unit => basic_type("()", 0, "DW_ATE_unsigned"),
            _ => return Err("Rust debug type is not represented by the native scalar emitter".into()),
        };
        let id = self.push(entry);
        self.types.insert(ty, (id, name));
        Ok(id)
    }

    fn source(&mut self, function: &FnGen<'_, '_>, span: Span) -> Option<(usize, u32, u32)> {
        let span = function.map.call_site(span);
        if span.lo == span.hi || span.file.0 as usize >= function.map.file_count() || function.map.text(span.file).is_none() {
            return None;
        }
        let path = function.map.path(span.file);
        if path.is_empty() {
            return None;
        }
        let (line, column) = function.map.line_col(span);
        let file = if let Some(&id) = self.files.get(path) {
            id
        } else {
            let id = self.push(format!("!DIFile(filename: {}, directory: \"\")", quote(path)));
            self.files.insert(path.into(), id);
            id
        };
        Some((file, line, column))
    }

    fn scope(&self) -> usize {
        self.active.last().map_or(self.subprogram, |&(_, scope)| scope)
    }

    fn location(&mut self, scope: usize, line: u32, column: u32) -> usize {
        let key = (scope, line, column);
        if let Some(&id) = self.locations.get(&key) {
            return id;
        }
        let id = self.push(format!("!DILocation(line: {line}, column: {column}, scope: !%{scope})"));
        self.locations.insert(key, id);
        id
    }

    pub(super) fn mark(&mut self, function: &FnGen<'_, '_>, body: &mut String, start: usize, span: Span) {
        if start == body.len() {
            return;
        }
        let Some((file, line, column)) = self.source(function, span) else {
            return;
        };
        let parent = self.scope();
        let scope = if let Some(&id) = self.scope_files.get(&(parent, file)) {
            id
        } else {
            let id = self.push(format!("!DILexicalBlockFile(scope: !%{parent}, file: !%{file}, discriminator: 0)"));
            self.scope_files.insert((parent, file), id);
            id
        };
        let location = self.location(scope, line, column);
        let code = body.split_off(start);
        let mut switch = false;
        for line in code.lines() {
            body.push_str(line);
            // The emitter's switch cases are the only instruction it splits
            // across lines. Attach metadata after the closing bracket.
            switch |= line.starts_with("  switch ") && !line.ends_with(']');
            if switch && line.trim_end().ends_with(']') {
                switch = false;
            }
            if !switch && (line.starts_with("  ") || line.trim() == "]") {
                let _ = write!(body, ", !dbg !%{location}");
            }
            body.push('\n');
        }
    }

    pub(super) fn declare(&mut self, body: &mut String, local: Local, argument: bool) {
        let Some((file, line, column, ty, name)) = self.bindings.get(&local).cloned() else {
            return;
        };
        if self.active.iter().any(|&(l, _)| l == local) {
            return;
        }
        let mut scope = self.subprogram;
        let arg = if argument {
            format!(", arg: {local}")
        } else {
            let parent = self.scope();
            scope = self.push(format!("distinct !DILexicalBlock(scope: !%{parent}, file: !%{file}, line: {line}, column: {column})"));
            self.active.push((local, scope));
            String::new()
        };
        let node = self.push(format!("!DILocalVariable(name: {name}{arg}, scope: !%{scope}, file: !%{file}, line: {line}, type: !%{ty})"));
        let location = self.location(scope, line, column);
        let _ = writeln!(
            body,
            "  call void @llvm.dbg.declare(metadata ptr %l{local}, metadata !%{node}, metadata !DIExpression()), !dbg !%{location}"
        );
    }

    pub(super) fn dead(&mut self, local: Local) -> Result<(), String> {
        let Some(index) = self.active.iter().position(|&(l, _)| l == local) else {
            return Ok(());
        };
        if index + 1 != self.active.len() {
            return Err("Rust debug locations for overlapping lexical bindings are not represented".into());
        }
        self.active.pop();
        Ok(())
    }

    pub(super) fn has_binding(&self, local: Local) -> bool {
        self.bindings.contains_key(&local)
    }

    pub(super) fn attribute(&self) -> String {
        format!(" !dbg !%{}", self.subprogram)
    }

    pub(super) fn finish(self, output: &mut String) {
        for (index, entry) in self.entries.into_iter().enumerate() {
            let _ = writeln!(output, "!%{index} = {entry}");
        }
    }
}

fn basic_type(name: &str, size: u32, encoding: &str) -> (String, String) {
    (name.to_owned(), format!("!DIBasicType(name: {}, size: {size}, encoding: {encoding})", quote(name)))
}

fn quote(text: &str) -> String {
    let mut output = String::from("\"");
    for byte in text.bytes() {
        if (b' '..=b'~').contains(&byte) && !matches!(byte, b'"' | b'\\' | b'%') {
            output.push(byte as char);
        } else {
            let _ = write!(output, "\\{byte:02X}");
        }
    }
    output.push('"');
    output
}

/// Assign module-local metadata indices after unit grouping. Debug builds
/// keep definitions in their own units, with no cross-unit inline copies.
pub(super) fn append(output: &mut String, text: &str, next: &mut usize, units: &mut Vec<usize>) {
    let base = *next;
    units.push(base);
    let mut rest = text;
    while let Some((head, token)) = rest.split_once("!%") {
        output.push_str(head);
        let end = token.bytes().take_while(u8::is_ascii_digit).count();
        let local = token[..end].parse::<usize>().expect("generated LLVM metadata index");
        let index = base + local;
        let _ = write!(output, "!{index}");
        *next = (*next).max(index + 1);
        rest = &token[end..];
    }
    output.push_str(rest);
}
