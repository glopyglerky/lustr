// Rust source debug records for the LLVM emitter. The native checker and
// lowerer supply bindings and types; LLVM resolves instruction and frame
// locations to the addresses in the emitted object.
use super::{CTy, FnGen, Layout, LocalDecl, Span, VariantShape, c_layout, cty, ty_name};
use crate::check::collect::Variant;
use crate::ir::{Body, Local, StmtKind, Terminator};
use crate::ty::fmt::{FmtTy, fmt_kind};
use crate::ty::layout::{self, EnumTag, Repr, ReprKind, niche_tag_value};
use crate::ty::{FloatTy, IntTy, Mut, Ty, TyIntern, TyKind, USIZE, int};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fmt::Write;

struct DebugVariant {
    name: String,
    discriminant: i128,
    fields: Vec<(String, Ty)>,
}

impl DebugVariant {
    fn builtin(name: &str, discriminant: i128, payload: Option<Ty>) -> Self {
        Self {
            name: name.into(),
            discriminant,
            fields: payload.into_iter().map(|ty| ("__0".into(), ty)).collect(),
        }
    }
}

pub(super) struct DebugFn {
    entries: Vec<String>,
    files: BTreeMap<String, usize>,
    types: HashMap<Ty, (usize, String)>,
    locations: BTreeMap<(usize, u32, u32), usize>,
    scope_files: BTreeMap<(usize, usize), usize>,
    subprogram: usize,
    active: Vec<(Local, usize)>,
    bindings: BTreeMap<Local, (usize, u32, u32, usize, String)>,
    scopes: BTreeMap<(Local, usize), (usize, usize)>,
    entry_locals: Vec<Option<Vec<Local>>>,
}

impl DebugFn {
    pub(super) fn new(function: &FnGen<'_, '_>, linkage: Option<&str>, optimized: bool) -> Result<Self, String> {
        let mut debug = Self {
            entries: vec![String::new()],
            files: BTreeMap::new(),
            types: HashMap::new(),
            locations: BTreeMap::new(),
            scope_files: BTreeMap::new(),
            subprogram: 0,
            active: Vec::new(),
            bindings: BTreeMap::new(),
            scopes: BTreeMap::new(),
            entry_locals: Vec::new(),
        };
        let def = function.p.fn_def(function.body.fn_id);
        let (file, line, _) = debug.source(function, def.name_span).ok_or("Rust debug function has no source location")?;
        debug.entries[0] = format!(
            "distinct !DICompileUnit(language: DW_LANG_Rust, file: !%{file}, producer: \"Lustr\", isOptimized: {optimized}, runtimeVersion: 0, emissionKind: FullDebug)"
        );
        let name = function.p.display_path(function.p.fn_info(function.body.fn_id).name);
        let mut types = Vec::new();
        let closure = function.p.gx.closure_fns.contains_key(&function.body.fn_id);
        for (i, local) in function.body.locals[..=function.body.arg_count].iter().enumerate() {
            if closure && i == 1 {
                continue;
            }
            let ty = if matches!(function.body.tys.kind(local.ty), TyKind::Unit | TyKind::Never) {
                "null".into()
            } else {
                format!("!%{}", debug.ty(function, local.ty)?)
            };
            types.push(ty);
        }
        let signature = debug.push(format!("!DISubroutineType(types: !{{{}}})", types.join(", ")));
        let linkage = linkage.map_or_else(String::new, |name| format!(", linkageName: {}", quote(name)));
        let optimization_flag = if optimized { " | DISPFlagOptimized" } else { "" };
        debug.subprogram = debug.push(format!(
            "distinct !DISubprogram(name: {}{linkage}, scope: !%{file}, file: !%{file}, line: {line}, type: !%{signature}, scopeLine: {line}, spFlags: DISPFlagDefinition{optimization_flag}, unit: !%0)",
            quote(&name)
        ));
        for (i, local) in function.body.locals.iter().enumerate().skip(1) {
            let LocalDecl { name: Some(name), binding_span: Some(span), ty, .. } = local else { continue };
            let (file, line, column) = debug.source(function, *span).ok_or("Rust debug binding has no source location")?;
            let ty = debug.ty(function, *ty)?;
            debug.bindings.insert(i as Local, (file, line, column, ty, quote(name)));
        }
        debug.entry_locals = incoming_locals(function.body, |local| local as usize > function.body.arg_count && debug.has_binding(local));
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
                let prefix = match (function.body.tys.kind(ty), m) {
                    (TyKind::Ref(..), Mut::Shared) => "&",
                    (TyKind::Ref(..), Mut::Mut) => "&mut ",
                    (_, Mut::Shared) => "*const ",
                    (_, Mut::Mut) => "*mut ",
                };
                let element = match function.body.tys.kind(base) {
                    TyKind::Str => Some((int(IntTy::U8), true)),
                    TyKind::Slice(element) => Some((element, false)),
                    _ => None,
                };
                if let Some((element, string)) = element {
                    return self.slice(function, ty, prefix, element, string);
                }
                if matches!(function.body.tys.kind(base), TyKind::Dyn(..)) {
                    return self.trait_pointer(function, ty);
                }
                let name = ty_name(function.p, &function.body.tys, ty);
                let id = self.reserve_type(ty, &name);
                let target = self.ty(function, base)?;
                let size = debug_layout(function, ty)?.size * 8;
                self.entries[id] =
                    format!("!DIDerivedType(tag: DW_TAG_pointer_type, name: {}, baseType: !%{target}, size: {size})", quote(&name));
                return Ok(id);
            }
            TyKind::Unit => basic_type("()", 0, "DW_ATE_unsigned"),
            TyKind::Never => basic_type("!", 0, "DW_ATE_unsigned"),
            TyKind::FnPtr(signature) => {
                let signature = function.body.tys.sig(signature).value;
                let shape = debug_layout(function, ty)?;
                let name = ty_name(function.p, &function.body.tys, ty);
                let id = self.reserve_type(ty, &name);
                let mut types = Vec::new();
                let result = match function.body.tys.kind(signature.output) {
                    TyKind::Unit | TyKind::Never => "null".to_owned(),
                    _ => format!("!%{}", self.ty(function, signature.output)?),
                };
                types.push(result);
                for &input in function.body.tys.tys_of(signature.inputs).iter() {
                    types.push(format!("!%{}", self.ty(function, input)?));
                }
                let signature = self.push(format!("!DISubroutineType(types: !{{{}}})", types.join(", ")));
                self.entries[id] = format!(
                    "!DIDerivedType(tag: DW_TAG_pointer_type, name: {}, baseType: !%{signature}, size: {})",
                    quote(&name),
                    shape.size * 8
                );
                return Ok(id);
            }
            TyKind::Adt(..) if let CTy::Adt(adt) = cty(&function.body.tys, ty) => {
                let definition = function.p.adt(adt);
                if definition.is_enum {
                    let variants = definition.variants.iter().map(|variant| DebugVariant {
                        name: function.p.name(variant.name).into(),
                        discriminant: variant.disc,
                        fields: variant_fields(function, variant),
                    }).collect::<Vec<_>>();
                    return self.variants(function, ty, &variants);
                }
                let name = function.p.display_path(definition.name);
                let fields = variant_fields(function, &definition.variants[0]);
                return self.aggregate(function, ty, name, &fields, debug_layout(function, ty)?);
            }
            TyKind::Tuple(elements) => {
                let elements = function.body.tys.tys_of(elements);
                let mut names = Vec::new();
                let mut fields = Vec::new();
                for (i, &element) in elements.iter().enumerate() {
                    self.ty(function, element)?;
                    names.push(self.types[&element].1.clone());
                    fields.push((format!("__{i}"), element));
                }
                let comma = if elements.len() == 1 { "," } else { "" };
                return self.aggregate(function, ty, format!("({}{comma})", names.join(", ")), &fields, debug_layout(function, ty)?);
            }
            TyKind::Array(element, _) if let CTy::Array(_, count) = cty(&function.body.tys, ty) => {
                let base = self.ty(function, element)?;
                let shape = debug_layout(function, ty)?;
                let name = format!("[{}; {count}]", self.types[&element].1);
                let count = self.push(format!("!DISubrange(count: {count})"));
                let elements = self.push(format!("!{{!%{count}}}"));
                let entry = format!(
                    "!DICompositeType(tag: DW_TAG_array_type, baseType: !%{base}, size: {}, align: {}, elements: !%{elements})",
                    shape.size * 8,
                    shape.align * 8
                );
                (name, entry)
            }
            TyKind::Adt(..) if let Some((ok, error)) = function.body.tys.result_args(ty) => {
                return self.variants(function, ty, &[
                    DebugVariant::builtin("Ok", 0, Some(ok)),
                    DebugVariant::builtin("Err", 1, Some(error)),
                ]);
            }
            TyKind::Adt(..) if let Some(value) = function.body.tys.option_elem(ty) => {
                return self.variants(function, ty, &[
                    DebugVariant::builtin("None", 0, None),
                    DebugVariant::builtin("Some", 1, Some(value)),
                ]);
            }
            TyKind::Adt(..) if matches!(cty(&function.body.tys, ty), CTy::BoxDyn) => return self.trait_pointer(function, ty),
            TyKind::Adt(..) if function.body.tys.is_exit_code(ty) => {
                let shape = debug_layout(function, ty)?;
                let word = debug_layout(function, int(IntTy::U8))?;
                if shape.size != word.size || shape.align != word.align {
                    return Err("Rust ExitCode does not have the native status-byte layout".into());
                }
                let id = self.reserve_type(ty, "ExitCode");
                let byte = self.ty(function, int(IntTy::U8))?;
                let code = self.field(id, "__0", byte, word.size, word.align, 0);
                self.structure(id, "ExitCode", &shape, &[code]);
                return Ok(id);
            }
            TyKind::Adt(..) if matches!(cty(&function.body.tys, ty), CTy::String | CTy::Vec(_)) => {
                let element = function.body.tys.vec_elem(ty).unwrap_or_else(|| int(IntTy::U8));
                return self.buffer(function, ty, element);
            }
            TyKind::Adt(..) if fmt_kind(&function.body.tys, ty) == Some(FmtTy::Formatter) => {
                return self.formatter(function, ty);
            }
            TyKind::Dyn(..) => return self.trait_pointer(function, ty),
            kind => {
                return Err(format!(
                    "Rust debug type {} ({kind:?}) in {} is not represented",
                    ty_name(function.p, &function.body.tys, ty),
                    function.p.display_path(function.p.fn_info(function.body.fn_id).name)
                ));
            }
        };
        let id = self.push(entry);
        self.types.insert(ty, (id, name));
        Ok(id)
    }

    fn reserve_type(&mut self, ty: Ty, name: &str) -> usize {
        let id = self.push(String::new());
        self.types.insert(ty, (id, name.to_owned()));
        id
    }

    fn structure(&mut self, id: usize, name: &str, shape: &Layout, members: &[usize]) {
        let elements = self.push(format!("!{{{}}}", members.iter().map(|field| format!("!%{field}")).collect::<Vec<_>>().join(", ")));
        self.entries[id] = format!(
            "distinct !DICompositeType(tag: DW_TAG_structure_type, name: {}, size: {}, align: {}, elements: !%{elements})",
            quote(name),
            shape.size * 8,
            shape.align * 8
        );
    }

    fn field(&mut self, scope: usize, name: &str, ty: usize, size: u64, align: u64, offset: u64) -> usize {
        self.push(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: {}, scope: !%{scope}, baseType: !%{ty}, size: {}, align: {}, offset: {})",
            quote(name),
            size * 8,
            align * 8,
            offset * 8
        ))
    }

    fn aggregate(
        &mut self,
        function: &FnGen<'_, '_>,
        ty: Ty,
        name: String,
        fields: &[(String, Ty)],
        shape: Layout,
    ) -> Result<usize, String> {
        if fields.len() != shape.field_offsets.len() {
            return Err(format!("Rust debug layout for {name} has a mismatched field list"));
        }
        let id = self.reserve_type(ty, &name);
        let mut members = Vec::new();
        for ((name, ty), &offset) in fields.iter().zip(&shape.field_offsets) {
            let base = self.ty(function, *ty)?;
            let field = debug_layout(function, *ty)?;
            members.push(self.field(id, name, base, field.size, field.align.min(shape.align), offset));
        }
        self.structure(id, &name, &shape, &members);
        Ok(id)
    }

    fn slice(&mut self, function: &FnGen<'_, '_>, ty: Ty, prefix: &str, element: Ty, string: bool) -> Result<usize, String> {
        let base = self.ty(function, element)?;
        let width = self.ty(function, USIZE)?;
        let element_name = &self.types[&element].1;
        let element_type = if string { "str".into() } else { format!("[{element_name}]") };
        let name = format!("{prefix}{element_type}");
        let pointer = self.push(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, name: {}, baseType: !%{base}, size: 64)",
            quote(&format!("*const {element_name}"))
        ));
        let shape = debug_layout(function, ty)?;
        let word = debug_layout(function, USIZE)?;
        if shape.size != 2 * word.size || shape.align != word.align {
            return Err(format!("Rust debug layout for {name} is not a two-word slice reference"));
        }
        let id = self.reserve_type(ty, &name);
        let data = self.field(id, "data_ptr", pointer, word.size, word.align, 0);
        let length = self.field(id, "length", width, word.size, word.align, word.size);
        self.structure(id, &name, &shape, &[data, length]);
        Ok(id)
    }

    fn trait_pointer(&mut self, function: &FnGen<'_, '_>, ty: Ty) -> Result<usize, String> {
        let name = ty_name(function.p, &function.body.tys, ty);
        let shape = debug_layout(function, ty)?;
        let word = debug_layout(function, USIZE)?;
        if shape.size != 2 * word.size || shape.align != word.align {
            return Err(format!("Rust debug layout for {name} is not a two-word trait pointer"));
        }
        let id = self.reserve_type(ty, &name);
        let pointer = self.push(format!("!DIDerivedType(tag: DW_TAG_pointer_type, baseType: null, size: {})", word.size * 8));
        let data = self.field(id, "pointer", pointer, word.size, word.align, 0);
        let metadata = self.field(id, "vtable", pointer, word.size, word.align, word.size);
        self.structure(id, &name, &shape, &[data, metadata]);
        Ok(id)
    }

    fn buffer(&mut self, function: &FnGen<'_, '_>, ty: Ty, element: Ty) -> Result<usize, String> {
        let name = ty_name(function.p, &function.body.tys, ty);
        let shape = debug_layout(function, ty)?;
        let word = debug_layout(function, USIZE)?;
        let [ptr_at, len_at, cap_at] = shape.field_offsets[..] else {
            return Err(format!("Rust debug layout for {name} does not have the native buffer fields"));
        };
        let id = self.reserve_type(ty, &name);
        let base = self.ty(function, element)?;
        let usize_type = self.ty(function, USIZE)?;
        let pointer = self.push(format!("!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !%{base}, size: {})", word.size * 8));
        let ptr = self.field(id, "ptr", pointer, word.size, word.align, ptr_at);
        let len = self.field(id, "len", usize_type, word.size, word.align, len_at);
        let cap = self.field(id, "cap", usize_type, word.size, word.align, cap_at);
        self.structure(id, &name, &shape, &[ptr, len, cap]);
        Ok(id)
    }

    fn formatter(&mut self, function: &FnGen<'_, '_>, ty: Ty) -> Result<usize, String> {
        let name = "std::fmt::Formatter";
        // These private fields describe the native Formatter in runtime/fmt.h.
        let pointer = layout::ptr_layout();
        let fields = [
            ("fill", IntTy::U32),
            ("align", IntTy::U8),
            ("flags", IntTy::U8),
            ("width", IntTy::U16),
            ("prec", IntTy::U16),
        ];
        let layouts: Vec<_> = std::iter::once(pointer.clone())
            .chain(fields.iter().map(|&(_, ty)| layout::int_layout(ty)))
            .collect();
        let shape = layout::struct_layout(&layouts, Repr { kind: ReprKind::C, ..Repr::default() })
            .map_err(|error| format!("Rust debug layout for {name}: {error:?}"))?;
        let native = debug_layout(function, ty)?;
        if (shape.size, shape.align) != (native.size, native.align) {
            return Err(format!("Rust debug layout for {name} differs from the native runtime"));
        }
        let id = self.reserve_type(ty, name);
        let sink = self.push(format!("!DIDerivedType(tag: DW_TAG_pointer_type, baseType: null, size: {})", pointer.size * 8));
        let mut members = vec![self.field(id, "buf", sink, pointer.size, pointer.align, shape.field_offsets[0])];
        for ((name, ty), (field, &offset)) in fields.into_iter().zip(layouts[1..].iter().zip(&shape.field_offsets[1..])) {
            let base = self.ty(function, int(ty))?;
            members.push(self.field(id, name, base, field.size, field.align, offset));
        }
        self.structure(id, name, &shape, &members);
        Ok(id)
    }

    fn variants(&mut self, function: &FnGen<'_, '_>, ty: Ty, variants: &[DebugVariant]) -> Result<usize, String> {
        let name = ty_name(function.p, &function.body.tys, ty);
        let layout = debug_layout(function, ty)?;
        let id = self.reserve_type(ty, &name);
        if layout.uninhabited {
            self.structure(id, &name, &layout, &[]);
            return Ok(id);
        }
        let kind = layout.enum_tag.ok_or("native enum debug layout has no tag form")?;
        if layout.variant_offsets.len() != variants.len() {
            return Err(format!("native enum debug layout for {name} has a mismatched variant list"));
        }
        let part = self.push(String::new());
        let tag = match kind {
            EnumTag::Explicit { size, signed } => Some((0, size, signed)),
            EnumTag::NicheFilled { niche, .. } => Some((niche.offset, niche.scalar_size, false)),
            EnumTag::None(_) => None,
        };
        let discriminator = if let Some((offset, size, signed)) = tag {
            let integer = IntTy::ALL
                .iter()
                .copied()
                .find(|t| u64::from(t.bits()) == size * 8 && t.signed() == signed)
                .ok_or("native debug enum uses an unrepresented tag width")?;
            let base = self.ty(function, int(integer))?;
            let entry = self.push(format!(
                "!DIDerivedType(tag: DW_TAG_member, scope: !%{id}, baseType: !%{base}, size: {}, offset: {}, flags: DIFlagArtificial)",
                size * 8,
                offset * 8
            ));
            format!(", discriminator: !%{entry}")
        } else {
            String::new()
        };
        let mut members = Vec::new();
        for (index, variant) in variants.iter().enumerate() {
            if matches!(kind, EnumTag::None(single) if single != index) {
                continue;
            }
            let offsets = &layout.variant_offsets[index];
            if offsets.len() != variant.fields.len() {
                return Err(format!("native enum debug layout for {name}::{} has a mismatched field list", variant.name));
            }
            let variant_id = self.push(String::new());
            let mut payload = Vec::new();
            for ((field, ty), &offset) in variant.fields.iter().zip(offsets) {
                let member = self.ty(function, *ty)?;
                let shape = debug_layout(function, *ty)?;
                payload.push(self.field(variant_id, field, member, shape.size, shape.align.min(layout.align), offset));
            }
            self.structure(variant_id, &variant.name, &layout, &payload);
            let value = match kind {
                EnumTag::Explicit { size, signed } => {
                    // LLVM's constant has the same bit width as the stored tag.
                    let repr = if signed { variant.discriminant.to_string() } else { (variant.discriminant as u128).to_string() };
                    format!(", extraData: i{} {repr}", size * 8)
                }
                EnumTag::NicheFilled { variant: data, niche, first, last, start } if index != data && (first..=last).contains(&index) => {
                    format!(", extraData: i{} {}", niche.scalar_size * 8, niche_tag_value(niche, first, start, index))
                }
                _ => String::new(),
            };
            members.push(self.push(format!(
                "!DIDerivedType(tag: DW_TAG_member, name: {}, scope: !%{part}, baseType: !%{variant_id}, size: {}, align: {}{value})",
                quote(&variant.name),
                layout.size * 8,
                layout.align * 8
            )));
        }
        let list = self.push(format!("!{{{}}}", members.iter().map(|member| format!("!%{member}")).collect::<Vec<_>>().join(", ")));
        self.entries[part] = format!(
            "!DICompositeType(tag: DW_TAG_variant_part, scope: !%{id}, size: {}, align: {}, elements: !%{list}{discriminator})",
            layout.size * 8,
            layout.align * 8
        );
        self.structure(id, &name, &layout, &[part]);
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

    fn binding(&mut self, local: Local, argument: bool) -> Option<(usize, usize)> {
        let (file, line, column, ty, name) = self.bindings.get(&local).cloned()?;
        if self.active.iter().any(|&(l, _)| l == local) {
            return None;
        }
        let parent = if argument { self.subprogram } else { self.scope() };
        let (scope, node) = if let Some(&nodes) = self.scopes.get(&(local, parent)) {
            nodes
        } else {
            let mut scope = parent;
            let arg = if argument {
                format!(", arg: {local}")
            } else {
                scope = self.push(format!("distinct !DILexicalBlock(scope: !%{parent}, file: !%{file}, line: {line}, column: {column})"));
                String::new()
            };
            let node =
                self.push(format!("!DILocalVariable(name: {name}{arg}, scope: !%{scope}, file: !%{file}, line: {line}, type: !%{ty})"));
            self.scopes.insert((local, parent), (scope, node));
            (scope, node)
        };
        if !argument {
            self.active.push((local, scope));
        }
        Some((node, self.location(scope, line, column)))
    }

    pub(super) fn declare(&mut self, body: &mut String, local: Local, argument: bool) {
        let Some((node, location)) = self.binding(local, argument) else { return };
        let _ = writeln!(
            body,
            "  call void @llvm.dbg.declare(metadata ptr %l{local}, metadata !%{node}, metadata !DIExpression()), !dbg !%{location}"
        );
    }

    pub(super) fn scalar_argument(&mut self, entry: &mut String, local: Local, ty: &str, align: u64) {
        let _ = writeln!(entry, "  %l{local} = alloca {ty}, align {align}");
        let binding = self.binding(local, true);
        if let Some((node, location)) = binding {
            let _ = writeln!(
                entry,
                "  call void @llvm.dbg.value(metadata {ty} %p{local}, metadata !%{node}, metadata !DIExpression()), !dbg !%{location}"
            );
        }
        let _ = writeln!(entry, "  store {ty} %p{local}, ptr %l{local}");
        if let Some((node, location)) = binding {
            // Before the spill the incoming SSA value owns the location.
            // Afterward the variable can be read and changed in its stack slot.
            let _ = writeln!(
                entry,
                "  call void @llvm.dbg.value(metadata ptr %l{local}, metadata !%{node}, metadata !DIExpression(DW_OP_deref)), !dbg !%{location}"
            );
        }
    }

    pub(super) fn dead(&mut self, body: &mut String, local: Local) {
        let Some(index) = self.active.iter().position(|&(l, _)| l == local) else {
            return;
        };
        let inner = self.active.split_off(index);
        for (alive, _) in inner.into_iter().skip(1) {
            self.declare(body, alive, false);
        }
    }

    pub(super) fn enter_block(&mut self, body: &mut String, block: usize, cleanup: bool) {
        self.active.clear();
        if !cleanup && let Some(locals) = self.entry_locals[block].clone() {
            for local in locals {
                self.declare(body, local, false);
            }
        }
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

/// Local storage known initialized on every normal predecessor. A call's
/// destination becomes live on its normal edge, never on its unwind edge.
fn incoming_locals(body: &Body, tracked: impl Fn(Local) -> bool) -> Vec<Option<Vec<Local>>> {
    let mut incoming = vec![None; body.blocks.len()];
    incoming[0] = Some(Vec::new());
    let mut queue = VecDeque::from([0]);
    while let Some(block_id) = queue.pop_front() {
        let block = &body.blocks[block_id];
        let mut locals = incoming[block_id].clone().expect("reachable block");
        let initialize = |locals: &mut Vec<Local>, local| {
            if tracked(local)
                && let Err(index) = locals.binary_search(&local)
            {
                locals.insert(index, local);
            }
        };
        for stmt in &block.stmts {
            match &stmt.kind {
                StmtKind::Assign(place, _) if place.proj.is_empty() => initialize(&mut locals, place.local),
                StmtKind::StorageDead(local) => {
                    if let Ok(index) = locals.binary_search(local) {
                        locals.remove(index);
                    }
                }
                _ => {}
            }
        }
        let successors = match &block.term {
            Terminator::Goto(next) => vec![*next],
            Terminator::If(_, yes, no) => vec![*yes, *no],
            Terminator::Switch(_, cases, other) => cases.iter().map(|(_, next)| *next).chain([*other]).collect(),
            Terminator::Call { dest, next, .. } => {
                if dest.proj.is_empty() {
                    initialize(&mut locals, dest.local);
                }
                vec![*next]
            }
            _ => Vec::new(),
        };
        for next in successors {
            let next = next.0 as usize;
            let changed = if let Some(old) = &mut incoming[next] {
                let before = old.len();
                old.retain(|local| locals.binary_search(local).is_ok());
                old.len() != before
            } else {
                incoming[next] = Some(locals.clone());
                true
            };
            if changed {
                queue.push_back(next);
            }
        }
    }
    incoming
}

fn debug_layout(function: &FnGen<'_, '_>, ty: Ty) -> Result<Layout, String> {
    c_layout(function.p, &cty(&function.body.tys, ty)).ok_or_else(|| "Rust debug type has no native layout".into())
}

fn variant_fields(function: &FnGen<'_, '_>, variant: &Variant) -> Vec<(String, Ty)> {
    variant.fields.iter().enumerate().map(|(i, field)| {
        let name = if variant.shape == VariantShape::Named { function.p.name(field.name).to_owned() } else { format!("__{i}") };
        (name, field.ty)
    }).collect()
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
