//! Completed native facts for the first genuine Rust metadata producer.
//!
//! This is a deliberately closed declaration/body boundary, not a replacement
//! for the full Rust analysis query. It consumes the original native checked
//! program; it neither reparses source nor asks another compiler to accept it.
//! Missing declaration, lint, type, or body representation remains integration
//! work. It never transfers a native-owned function to another body compiler.

use crate::{ast, check, def, diag, native_object, ty};
use std::collections::{BTreeMap, BTreeSet};
use ty::TyIntern;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Error {
    Unintegrated(&'static str),
    Failed(&'static str),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Visibility {
    Public,
    Private,
}

pub(crate) struct Parameter {
    name: String,
    span: diag::Span,
}

impl Parameter {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }
    pub(crate) fn span(&self) -> diag::Span {
        self.span
    }
}

/// Every field comes from one actual collected/checked native declaration.
/// Original Rust definition indices are deliberately absent: the metadata
/// context must create those identities and retain an explicit mapping.
pub(crate) struct Function {
    definition: def::DiskDefId,
    local: def::LocalDefId,
    name: String,
    name_span: diag::Span,
    span: diag::Span,
    visibility: Visibility,
    parameters: Vec<Parameter>,
    calls: BTreeSet<def::DiskDefId>,
}

impl Function {
    pub(crate) fn definition(&self) -> def::DiskDefId {
        self.definition
    }
    pub(crate) fn local(&self) -> def::LocalDefId {
        self.local
    }
    pub(crate) fn name(&self) -> &str {
        &self.name
    }
    pub(crate) fn name_span(&self) -> diag::Span {
        self.name_span
    }
    pub(crate) fn span(&self) -> diag::Span {
        self.span
    }
    pub(crate) fn visibility(&self) -> Visibility {
        self.visibility
    }
    /// The current completed signature is `(u64, ...) -> u64`; no erased
    /// generic, lifetime, predicate, ABI or aggregate facts are implied.
    pub(crate) fn parameters(&self) -> &[Parameter] {
        &self.parameters
    }
    pub(crate) fn calls(&self) -> &BTreeSet<def::DiskDefId> {
        &self.calls
    }
}

/// This token is not constructible from a success flag or an emitter's exit
/// status. It owns the actual native program and a complete joined inventory.
/// All bodies have already passed native checking, lowering and borrowck.
pub(crate) struct CompletedUnit {
    prepared: native_object::PreparedUnit,
    functions: Vec<Function>,
}

impl CompletedUnit {
    pub(crate) fn prepared(&self) -> &native_object::PreparedUnit {
        &self.prepared
    }
    pub(crate) fn functions(&self) -> &[Function] {
        &self.functions
    }
    pub(crate) fn root_span(&self) -> diag::Span {
        diag::Span::new(diag::FileId(0), 0, self.prepared.source().len() as u32)
    }
    pub(crate) fn emit(self) -> Result<EmittedUnit, String> {
        let Self { prepared, functions } = self;
        let object = native_object::emit(prepared)?;
        let declared: BTreeSet<_> = functions.iter().map(Function::definition).collect();
        let emitted: BTreeSet<_> = object.functions().iter().map(native_object::EmittedFunction::definition).collect();
        if declared != emitted || object.functions().len() != functions.len() {
            return Err("native metadata declaration and emitted-instance inventories differ".into());
        }
        Ok(EmittedUnit { object, functions })
    }
}

pub(crate) struct EmittedUnit {
    object: native_object::EmittedUnit,
    functions: Vec<Function>,
}

impl EmittedUnit {
    pub(crate) fn native(&self) -> &native_object::EmittedUnit {
        &self.object
    }
    pub(crate) fn functions(&self) -> &[Function] {
        &self.functions
    }
}

/// Complete the applicable obligations of a flat scalar library. The native
/// parser/collector owns syntax, duplicates, signature well-formedness and
/// declaration binding. Native check/lower/borrowck owns every body. This pass
/// joins those actual facts and refuses obligations it cannot discharge.
pub(crate) fn complete(prepared: native_object::PreparedUnit) -> Result<CompletedUnit, Error> {
    let p = prepared.program();
    let syntax = &p.ast;
    if prepared.working_directory().is_none() {
        return Err(Error::Failed("native request working directory was not retained"));
    }
    if !prepared.source().is_ascii() {
        return Err(Error::Unintegrated("Unicode source lint and identifier records"));
    }
    if prepared.source().as_bytes().contains(&b'\r') {
        return Err(Error::Unintegrated("normalized source byte/span mapping"));
    }
    if prepared.source_map().file_count() != 1 {
        return Err(Error::Unintegrated("multiple source-file metadata generations"));
    }
    if !syntax.stmt_items.is_empty()
        || !syntax.anon_consts.is_empty()
        || !syntax.paren_exprs.is_empty()
        || !syntax.paren_tys.is_empty()
        || !p.consts.is_empty()
        || !p.statics.is_empty()
        || !p.body_consts.is_empty()
        || !p.body_items.is_empty()
        || !p.body_type_items.is_empty()
        || !p.std_fns.is_empty()
        || !p.shipped_ranges.is_empty()
        || !p.imported_crates.is_empty()
    {
        return Err(Error::Unintegrated("non-function definitions, nested syntax or additional lint obligations"));
    }
    if syntax.inner_attrs.len() != 1
        || !syntax.inner_attrs[0].path.as_slice().first().is_some_and(|segment| p.interner.get(segment.name) == "no_std")
    {
        return Err(Error::Failed("native root policy differs from its admitted source"));
    }
    if syntax.items.len() != p.fns.len() || p.fns.len() != prepared.accepted_bodies().len() {
        return Err(Error::Unintegrated("the complete declaration set is not a flat function set"));
    }
    let mut functions = Vec::new();
    let mut identities = BTreeSet::new();
    let mut covered_items = BTreeSet::new();
    for body in prepared.accepted_bodies() {
        let id = body.local();
        let Some(info) = p.fns.get(id.0 as usize) else {
            return Err(Error::Failed("accepted body has no collected declaration"));
        };
        if p.disk_id(id) != body.definition() || !identities.insert(body.definition()) {
            return Err(Error::Failed("accepted body identity is missing or duplicated"));
        }
        let check::collect::FnLoc::Item(item_index) = info.loc else {
            return Err(Error::Unintegrated("associated or nested definition metadata"));
        };
        let Some(item) = syntax.items.get(item_index as usize) else {
            return Err(Error::Failed("native function item is absent"));
        };
        let ast::ItemKind::Fn(function) = &item.kind else {
            return Err(Error::Failed("native function identity names another definition kind"));
        };
        if !covered_items.insert(item_index) || function.body != Some(body.body()) || function.name != info.name || function.vis != item.vis
        {
            return Err(Error::Failed("native body/declaration joining is not one-to-one"));
        }
        if !item.attrs.is_empty()
            || !function.attrs.is_empty()
            || function.is_test
            || function.qual != ast::FnQual::default()
            || function.self_param.is_some()
            || function.variadic
            || !function.generics.params.is_empty()
            || !function.where_clause.preds.is_empty()
            || info.is_const
            || p.gx.fn_class.get(id.0 as usize) != Some(&check::generics::FnClass::Plain)
        {
            return Err(Error::Unintegrated("function attributes, qualifiers, generics or predicates"));
        }
        let visibility = match function.vis {
            ast::Vis::Public => Visibility::Public,
            ast::Vis::Private => Visibility::Private,
            ast::Vis::Restricted { .. } => return Err(Error::Unintegrated("restricted visibility metadata")),
        };
        if !info.params.iter().chain([&info.ret]).all(|&t| p.tys.kind(t) == ty::TyKind::Int(ty::IntTy::U64))
            || info.params.len() != function.params.len()
            || function.params.len() > 64
            || function.ret.is_none()
        {
            return Err(Error::Unintegrated("non-u64 function signature or inferred return declaration"));
        }
        let name = p.interner.get(function.name);
        if !snake_case(name) {
            return Err(Error::Unintegrated("function naming lint diagnostics"));
        }
        let mut parameters = Vec::new();
        for parameter in &function.params {
            if parameter.mutable || parameter.pat.is_some() || !parameter.attrs.is_empty() {
                return Err(Error::Unintegrated("parameter pattern, attribute or mutation lint obligations"));
            }
            let name = p.interner.get(parameter.name);
            if !snake_case(name) {
                return Err(Error::Unintegrated("parameter naming lint diagnostics"));
            }
            let span = prepared
                .identifier_span(parameter.name, parameter.span)
                .ok_or(Error::Failed("parameter name is not bound to its original token"))?;
            parameters.push(Parameter { name: name.into(), span });
        }
        let block = syntax.block(body.body());
        if !block.stmts.is_empty() || !block.inner_attrs.is_empty() || !block.tail_attrs.is_empty() {
            return Err(Error::Unintegrated("statement, block or lint-scope obligations"));
        }
        let tail = block.tail.ok_or(Error::Unintegrated("non-expression function body"))?;
        let mut expressions = BTreeSet::new();
        let mut used_parameters = BTreeSet::new();
        let mut calls = BTreeSet::new();
        expression(p, body, tail, 0, &mut expressions, &mut used_parameters, &mut calls)?;
        if expressions.iter().copied().ne(function.exprs.0..function.exprs.1) {
            return Err(Error::Unintegrated("unrepresented native expressions in the accepted body"));
        }
        if parameters.iter().enumerate().any(|(index, parameter)| !used_parameters.contains(&index) && !parameter.name.starts_with('_')) {
            return Err(Error::Unintegrated("unused parameter lint diagnostics"));
        }
        if used_parameters.iter().any(|&index| index >= parameters.len()) {
            return Err(Error::Failed("native parameter resolution names another binding"));
        }
        if body.lowered_calls() != Some(&calls) {
            return Err(Error::Unintegrated("checked and lowered body call graphs do not have the same represented edges"));
        }
        let mut generated_uses = calls.clone();
        generated_uses.insert(body.definition());
        if body.generated_uses() != Some(&generated_uses) {
            return Err(Error::Unintegrated("generated source-function graph has unrepresented instance edges"));
        }
        functions.push(Function {
            definition: body.definition(),
            local: id,
            name: name.into(),
            name_span: function.name_span,
            span: item.span,
            visibility,
            parameters,
            calls,
        });
    }
    functions.sort_by_key(|function| function.span.lo);
    // All source-visible signatures are primitive. In one root module, actual
    // local resolved call edges can name private functions without crossing a
    // visibility boundary. There are no nominal types to leak in a public API.
    let by_identity: BTreeMap<_, _> = functions.iter().enumerate().map(|(index, function)| (function.definition, index)).collect();
    let mut incoming = vec![0usize; functions.len()];
    for function in &functions {
        for call in &function.calls {
            let target = *by_identity.get(call).ok_or(Error::Unintegrated("external call metadata or emitted dependency"))?;
            incoming[target] += 1;
        }
    }
    let mut ready: Vec<_> = incoming.iter().enumerate().filter_map(|(i, &n)| (n == 0).then_some(i)).collect();
    let mut acyclic = 0;
    while let Some(index) = ready.pop() {
        acyclic += 1;
        for call in &functions[index].calls {
            let target = by_identity[call];
            incoming[target] -= 1;
            if incoming[target] == 0 {
                ready.push(target);
            }
        }
    }
    if acyclic != functions.len() {
        return Err(Error::Unintegrated("recursive body liveness and recursion lint obligations"));
    }
    let mut reachable = BTreeSet::new();
    let mut pending: Vec<_> =
        functions.iter().enumerate().filter_map(|(i, function)| (function.visibility == Visibility::Public).then_some(i)).collect();
    while let Some(index) = pending.pop() {
        if reachable.insert(index) {
            pending.extend(functions[index].calls.iter().map(|call| by_identity[call]));
        }
    }
    if reachable.len() != functions.len() || functions.is_empty() {
        return Err(Error::Unintegrated("unused private definition lint diagnostics or empty exported graph"));
    }
    Ok(CompletedUnit { prepared, functions })
}

fn snake_case(name: &str) -> bool {
    !name.is_empty() && name.bytes().enumerate().all(|(n, c)| c == b'_' || c.is_ascii_lowercase() || (n != 0 && c.is_ascii_digit()))
}

fn expression(
    p: &check::collect::Program,
    body: &native_object::AcceptedBody,
    id: ast::ExprId,
    depth: usize,
    seen: &mut BTreeSet<u32>,
    parameters: &mut BTreeSet<usize>,
    calls: &mut BTreeSet<def::DiskDefId>,
) -> Result<(), Error> {
    use native_object::ExpressionResolution as Resolution;
    if depth > 128 {
        return Err(Error::Unintegrated("metadata expression nesting bound"));
    }
    if !seen.insert(id.0) {
        return Err(Error::Failed("native expression ownership is duplicated"));
    }
    let actual = body.expression(id).ok_or(Error::Failed("accepted expression has no native checked result"))?;
    match p.ast.exprs.get(id.0 as usize).ok_or(Error::Failed("accepted expression is absent"))? {
        ast::Expr::Int(value, None | Some(crate::lexer::IntSuffix::U64)) if *value <= u64::MAX as u128 => {}
        ast::Expr::Path(path) if path.len() == 1 && !path[0].has_args() => match actual {
            Resolution::Parameter(parameter) => {
                parameters.insert(parameter);
            }
            _ => return Err(Error::Unintegrated("non-parameter value resolution")),
        },
        ast::Expr::Binary { op: ast::BinOp::BitAnd | ast::BinOp::BitOr | ast::BinOp::BitXor, lhs, rhs }
            if actual == Resolution::PrimitiveOperands =>
        {
            expression(p, body, *lhs, depth + 1, seen, parameters, calls)?;
            expression(p, body, *rhs, depth + 1, seen, parameters, calls)?;
        }
        ast::Expr::Call { callee, args } => {
            if !seen.insert(callee.0) {
                return Err(Error::Failed("native callee expression ownership is duplicated"));
            }
            if !matches!(p.ast.exprs.get(callee.0 as usize), Some(ast::Expr::Path(path)) if path.len() == 1 && !path[0].has_args()) {
                return Err(Error::Unintegrated("indirect, generic or qualified call metadata"));
            }
            let Some(Resolution::Function(function)) = body.expression(*callee) else {
                return Err(Error::Unintegrated("call lacks an actual local native function resolution"));
            };
            calls.insert(function);
            for argument in args {
                expression(p, body, *argument, depth + 1, seen, parameters, calls)?;
            }
        }
        _ => return Err(Error::Unintegrated("body operation or applicable lint completion")),
    }
    Ok(())
}
