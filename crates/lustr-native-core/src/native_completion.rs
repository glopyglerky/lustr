//! Completed native facts for the first genuine Rust metadata producer.
//!
//! This is a deliberately closed declaration/body boundary, not a replacement
//! for the full Rust analysis query. It consumes the original native checked
//! program; it neither reparses source nor asks another compiler to accept it.
//! Missing declaration, lint, type, or body representation remains integration
//! work. It never transfers a native-owned function to another body compiler.

use crate::{ast, check::{self, generics::ProjectionReferent}, def, diag, ty};
use std::collections::{BTreeMap, BTreeSet};
use ty::TyIntern;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Error {
    Unintegrated(&'static str),
    Failed(&'static str),
}

pub(crate) struct FamilyDefinition {
    pub name: String,
    pub name_span: diag::Span,
    pub span: diag::Span,
}

pub(crate) struct Family {
    pub receiver: FamilyDefinition,
    pub family: FamilyDefinition,
    pub item: FamilyDefinition,
    pub lifetime: FamilyDefinition,
    pub implementation: Option<FamilyImplementation>,
}

pub(crate) struct FamilyImplementation {
    pub span: diag::Span,
    pub item: FamilyDefinition,
    pub lifetime: FamilyDefinition,
    pub referent: ProjectionReferent,
}

pub(crate) fn complete_family(p: &check::collect::Program, map: &diag::SourceMap, source: &str) -> Result<Family, Error> {
    let unsupported = Error::Unintegrated("projection provider does not have the represented checked declaration inventory");
    let fail = Error::Failed("native projection provider identities do not join checked declarations");
    let items: Vec<_> = p.ast.items.iter().filter(|item| !matches!(item.kind, ast::ItemKind::Fn(_))).collect();
    if !p.imported_crates.is_empty() || !p.ast.stmt_items.is_empty()
        || map.file_count() != 1 || !source.is_ascii() || source.as_bytes().contains(&b'\r')
        || items.len() < 2 || items.len() > 3 || items.iter().any(|item| !item.attrs.is_empty())
        || !matches!(p.ast.inner_attrs.as_slice(), [attr]
            if attr.path.as_slice().first().is_some_and(|segment| p.interner.get(segment.name) == "no_std"))
    {
        return Err(unsupported);
    }
    let [receiver, family, tail @ ..] = items.as_slice() else { return Err(unsupported); };
    let (ast::ItemKind::Struct(receiver_ast), ast::ItemKind::Trait(trait_ast)) = (&receiver.kind, &family.kind) else {
        return Err(unsupported);
    };
    if receiver.vis != ast::Vis::Public || family.vis != ast::Vis::Public
        || !matches!(receiver_ast.fields, ast::VariantFields::Unit)
        || !receiver_ast.generics.params.is_empty() || !receiver_ast.where_clause.preds.is_empty()
        || !receiver_ast.derives.is_empty() || !trait_ast.inner_attrs.is_empty()
        || trait_ast.is_unsafe || trait_ast.is_auto || !trait_ast.generics.params.is_empty()
        || !trait_ast.where_clause.preds.is_empty() || !trait_ast.bounds.is_empty()
        || !trait_ast.fns.is_empty() || !trait_ast.consts.is_empty()
    {
        return Err(unsupported);
    }
    let [item] = trait_ast.types.as_slice() else { return Err(unsupported); };
    if !item.attrs.is_empty() || item.default.is_some() { return Err(unsupported); }
    let named = |name, name_span, span| FamilyDefinition { name: p.interner.get(name).into(), name_span, span };
    let lifetime = |generics: &ast::Generics| match generics.params.as_slice() {
        [ast::GenericParam::Lifetime { lt, bounds, .. }] if bounds.is_empty() => Some(FamilyDefinition {
            name: format!("'{}", p.interner.get(lt.name)), span: lt.span, name_span: lt.span,
        }),
        _ => None,
    };
    let Some(trait_lifetime) = lifetime(&item.generics) else { return Err(unsupported); };
    let Some(&trait_id) = p.gx.trait_names.get(&trait_ast.name) else { return Err(fail); };
    let Some(trait_record) = p.gx.index.trait_record(trait_id) else { return Err(fail); };
    let Some(associated) = trait_record.assoc_by_name(item.name) else { return Err(fail); };
    let Some(declared) = p.gx.trait_assoc_generics.get(&associated) else { return Err(fail); };
    if declared.reference_family_lifetime(&p.tys, p.gx.lang.sized).is_none() { return Err(fail); }
    let implementation = match tail {
        [] if p.gx.index.impls_of_trait(trait_id).is_empty() => None,
        [source_impl] => {
            let ast::ItemKind::Impl(implementation) = &source_impl.kind else { return Err(unsupported); };
            let [value] = implementation.types.as_slice() else { return Err(unsupported); };
            if implementation.is_unsafe || implementation.is_negative || implementation.in_body
                || !implementation.generics.params.is_empty() || !implementation.where_clause.preds.is_empty()
                || !implementation.inner_attrs.is_empty() || !implementation.fns.is_empty() || !implementation.consts.is_empty()
                || !value.attrs.is_empty() || !value.where_clause.preds.is_empty()
            {
                return Err(unsupported);
            }
            let Some(impl_lifetime) = lifetime(&value.generics) else { return Err(unsupported); };
            let [id] = p.gx.index.impls_of_trait(trait_id) else { return Err(fail); };
            let record = p.gx.index.peek(*id);
            let Some(assignment) = p.gx.impl_assoc_generics.get(&(record.def, associated)) else { return Err(fail); };
            let Some(region) = assignment.declaration.single_lifetime() else { return Err(fail); };
            let ty::TyKind::Ref(actual, ty::Mut::Shared, inner) = p.tys.kind(assignment.value) else { return Err(fail); };
            if actual != region { return Err(fail); }
            Some(FamilyImplementation {
                span: implementation.span, item: named(value.name, value.name_span, value.span),
                lifetime: impl_lifetime, referent: ProjectionReferent::from_type(&p.tys, inner).ok_or(fail)?,
            })
        }
        _ => return Err(unsupported),
    };
    Ok(Family {
        receiver: named(receiver_ast.name, receiver_ast.name_span, receiver.span),
        family: named(trait_ast.name, trait_ast.name_span, family.span),
        item: named(item.name, item.name_span, item.span),
        lifetime: trait_lifetime,
        implementation,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Visibility {
    Public,
    Private,
}

#[derive(Clone)]
pub(crate) struct Parameter {
    name: String,
    span: diag::Span,
    ty: ty::IntTy,
}

impl Parameter {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }
    pub(crate) fn span(&self) -> diag::Span {
        self.span
    }
    pub(crate) fn ty(&self) -> ty::IntTy {
        self.ty
    }
}

/// Every field comes from one actual collected/checked native declaration.
/// Original Rust definition indices are deliberately absent: the metadata
/// context must create those identities and retain an explicit mapping.
#[derive(Clone)]
pub(crate) struct Function {
    definition: def::DiskDefId,
    local: def::LocalDefId,
    name: String,
    name_span: diag::Span,
    span: diag::Span,
    visibility: Visibility,
    parameters: Vec<Parameter>,
    result: ty::IntTy,
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
    pub(crate) fn parameters(&self) -> &[Parameter] {
        &self.parameters
    }
    pub(crate) fn result(&self) -> ty::IntTy {
        self.result
    }
    pub(crate) fn calls(&self) -> &BTreeSet<def::DiskDefId> {
        &self.calls
    }
}

/// One complete body accepted by the native checker, lowerer and borrow checker.
/// These records are created only while consuming that body's actual IR. They
/// are an inventory of native work, not a claim that Rust metadata obligations
/// (including declaration lints) have been completed.
#[derive(Clone)]
pub(crate) struct AcceptedBody {
    pub(crate) local: def::LocalDefId,
    pub(crate) definition: def::DiskDefId,
    pub(crate) body: ast::BlockId,
    pub(crate) expr_base: u32,
    pub(crate) expressions: std::sync::Arc<Vec<ExpressionResolution>>,
    pub(crate) pat_base: u32,
    pub(crate) bindings: std::sync::Arc<[Option<u32>]>,
    pub(crate) lowered_calls: Option<BTreeSet<def::DiskDefId>>,
    pub(crate) generated_uses: Option<BTreeSet<def::DiskDefId>>,
}

/// Arena-independent facts observed in the native body's actual checked table.
/// `Other` does not grant a metadata completion proof for the expression.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExpressionResolution {
    None,
    Parameter(usize),
    Local(u32),
    Function(def::DiskDefId),
    ImportedFunction(def::DiskDefId),
    PrimitiveOperands,
    Other,
}

impl AcceptedBody {
    pub(crate) fn local(&self) -> def::LocalDefId { self.local }
    pub(crate) fn definition(&self) -> def::DiskDefId { self.definition }
    pub(crate) fn body(&self) -> ast::BlockId { self.body }
    pub(crate) fn expression(&self, id: ast::ExprId) -> Option<ExpressionResolution> {
        self.expressions.get(id.0.checked_sub(self.expr_base)? as usize).copied()
    }
    fn binding(&self, id: ast::PatId) -> Option<u32> {
        self.bindings.get(id.0.checked_sub(self.pat_base)? as usize).copied().flatten()
    }
    pub(crate) fn lowered_calls(&self) -> Option<&BTreeSet<def::DiskDefId>> { self.lowered_calls.as_ref() }
    /// Actual source-function slots printed by the generator, including this
    /// function's own definition. Backend helper symbols are recorded on the
    /// emitted object separately.
    pub(crate) fn generated_uses(&self) -> Option<&BTreeSet<def::DiskDefId>> { self.generated_uses.as_ref() }
}

pub(crate) struct View<'a> {
    pub program: &'a check::collect::Program,
    pub source: &'a str,
    pub map: &'a diag::SourceMap,
    pub bodies: &'a [AcceptedBody],
    pub identifier: &'a dyn Fn(crate::intern::Sym, diag::Span) -> Option<diag::Span>,
}
impl View<'_> {
    fn program(&self) -> &check::collect::Program { self.program }
    fn source(&self) -> &str { self.source }
    fn source_map(&self) -> &diag::SourceMap { self.map }
    fn accepted_bodies(&self) -> &[AcceptedBody] { self.bodies }
    fn identifier_span(&self, name: crate::intern::Sym, span: diag::Span) -> Option<diag::Span> { (self.identifier)(name, span) }
}

pub(crate) fn complete(prepared: View<'_>, generated: bool, family: Option<&Family>) -> Result<Vec<Function>, Error> {
    let p = prepared.program();
    let syntax = &p.ast;
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
        || !syntax.paren_tys.is_empty()
        || !p.consts.is_empty()
        || !p.statics.is_empty()
        || !p.body_consts.is_empty()
        || !p.body_items.is_empty()
        || !p.body_type_items.is_empty()
        || !p.std_fns.is_empty()
        || !p.shipped_ranges.is_empty()
        || generated && !p.imported_crates.is_empty()
    {
        return Err(Error::Unintegrated("non-function definitions, nested syntax or additional lint obligations"));
    }
    if !(!generated && syntax.inner_attrs.is_empty()
        || syntax.inner_attrs.len() == 1
            && syntax.inner_attrs[0].path.as_slice().first().is_some_and(|segment| p.interner.get(segment.name) == "no_std"))
    {
        return Err(Error::Failed("native root policy differs from its admitted source"));
    }
    let type_items = family.map_or(0, |family| 2 + usize::from(family.implementation.is_some()));
    if syntax.items.len() != p.fns.len() + type_items || p.fns.len() != prepared.accepted_bodies().len() {
        return Err(Error::Unintegrated("the checked functions and type declarations do not cover the complete declaration set"));
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
        if info.params.len() != function.params.len()
            || function.params.len() > 64
            || function.ret.is_none()
        {
            return Err(Error::Unintegrated("unrepresented parameter inventory or inferred return declaration"));
        }
        let result = integer_type(p.tys.kind(info.ret))?;
        let name = p.interner.get(function.name);
        if !snake_case(name) {
            return Err(Error::Unintegrated("function naming lint diagnostics"));
        }
        let mut parameters = Vec::new();
        for (parameter, &checked_type) in function.params.iter().zip(&info.params) {
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
            parameters.push(Parameter { name: name.into(), span, ty: integer_type(p.tys.kind(checked_type))? });
        }
        let block = syntax.block(body.body());
        if !block.inner_attrs.is_empty() || !block.tail_attrs.is_empty() {
            return Err(Error::Unintegrated("statement, block or lint-scope obligations"));
        }
        let tail = block.tail.ok_or(Error::Unintegrated("non-expression function body"))?;
        let mut reads = BodyReads::default();
        for statement in &block.stmts {
            let ast::Stmt::Let { pat, init: Some(init), els: None, attrs, .. } = statement else {
                return Err(Error::Unintegrated("statement or lint-scope obligations"));
            };
            let ast::Pat::Binding { name, mutable: false, by_ref: false } = syntax.pat(*pat) else {
                return Err(Error::Unintegrated("local pattern or mutation obligations"));
            };
            let name = p.interner.get(*name);
            if !attrs.is_empty() || !snake_case(name) {
                return Err(Error::Unintegrated("binding attribute or naming lint obligations"));
            }
            let local = body.binding(*pat).ok_or(Error::Unintegrated("non-primitive or unbound checked local"))?;
            expression(p, body, *init, 0, false, &mut reads)?;
            if reads.locals.insert(local, name.starts_with('_')).is_some() {
                return Err(Error::Failed("two declarations name the same checked local"));
            }
        }
        expression(p, body, tail, 0, false, &mut reads)?;
        if reads.expressions.iter().copied().ne(function.exprs.0..function.exprs.1) {
            return Err(Error::Unintegrated("unrepresented native expressions in the accepted body"));
        }
        if parameters.iter().enumerate().any(|(index, parameter)| !reads.parameters.contains(&index) && !parameter.name.starts_with('_')) {
            return Err(Error::Unintegrated("unused parameter lint diagnostics"));
        }
        if reads.parameters.iter().any(|&index| index >= parameters.len()) {
            return Err(Error::Failed("native parameter resolution names another binding"));
        }
        if reads.locals.values().any(|&used_or_ignored| !used_or_ignored) {
            return Err(Error::Unintegrated("unused local lint diagnostics"));
        }
        let calls = reads.calls;
        if body.lowered_calls() != Some(&calls) {
            return Err(Error::Unintegrated("checked and lowered body call graphs do not have the same represented edges"));
        }
        let mut generated_uses = calls.clone();
        generated_uses.insert(body.definition());
        if generated && body.generated_uses() != Some(&generated_uses) {
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
            result,
            calls,
        });
    }
    if generated {
        require_direct_integer_abi(&functions)?;
    }
    functions.sort_by_key(|function| function.span.lo);
    // All source-visible signatures are primitive. In one root module, actual
    // local resolved call edges can name private functions without crossing a
    // visibility boundary. There are no nominal types to leak in a public API.
    let by_identity: BTreeMap<_, _> = functions.iter().enumerate().map(|(index, function)| (function.definition, index)).collect();
    let imported = |id: &def::DiskDefId| p.imported_crates.values().any(|stable| *stable == id.krate);
    let mut incoming = vec![0usize; functions.len()];
    for function in &functions {
        for call in &function.calls {
            if imported(call) { continue; }
            let target = *by_identity.get(call).ok_or(Error::Unintegrated("external call metadata or emitted dependency"))?;
            incoming[target] += 1;
        }
    }
    let mut ready: Vec<_> = incoming.iter().enumerate().filter_map(|(i, &n)| (n == 0).then_some(i)).collect();
    let mut acyclic = 0;
    while let Some(index) = ready.pop() {
        acyclic += 1;
        for call in &functions[index].calls {
            if imported(call) { continue; }
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
            pending.extend(functions[index].calls.iter().filter_map(|call| by_identity.get(call).copied()));
        }
    }
    if reachable.len() != functions.len() {
        return Err(Error::Unintegrated("unused private definition lint diagnostics"));
    }
    if functions.is_empty() && family.is_none() {
        return Err(Error::Unintegrated("empty exported graph"));
    }
    Ok(functions)
}

fn snake_case(name: &str) -> bool {
    !name.is_empty() && name.bytes().enumerate().all(|(n, c)| c == b'_' || c.is_ascii_lowercase() || (n != 0 && c.is_ascii_digit()))
}

fn integer_type(kind: ty::TyKind) -> Result<ty::IntTy, Error> {
    match kind {
        ty::TyKind::Int(ty) => Ok(ty),
        _ => Err(Error::Unintegrated("non-integer signature")),
    }
}

pub(crate) fn has_direct_integer_abi(kind: ty::IntTy) -> bool {
    matches!(kind.bits(), 32 | 64)
}

pub(crate) fn require_direct_integer_abi(functions: &[Function]) -> Result<(), Error> {
    if functions.iter().any(|function| {
        !has_direct_integer_abi(function.result)
            || function.parameters.iter().any(|param| !has_direct_integer_abi(param.ty))
    }) {
        return Err(Error::Unintegrated("unrepresented linked integer ABI"));
    }
    Ok(())
}

#[derive(Default)]
struct BodyReads {
    expressions: BTreeSet<u32>,
    parameters: BTreeSet<usize>,
    locals: BTreeMap<u32, bool>,
    calls: BTreeSet<def::DiskDefId>,
}

fn expression(
    p: &check::collect::Program,
    body: &AcceptedBody,
    id: ast::ExprId,
    depth: usize,
    bitwise_operand: bool,
    reads: &mut BodyReads,
) -> Result<(), Error> {
    use ExpressionResolution as Resolution;
    if depth > 128 {
        return Err(Error::Unintegrated("metadata expression nesting bound"));
    }
    // rustc's UnusedParens checks statement values and call arguments.
    // Bitwise operands are outside that check, and the native AST already
    // preserves the grouping that determines their evaluation order.
    if p.ast.is_parenthesized(id) && !bitwise_operand {
        return Err(Error::Unintegrated("parentheses lint diagnostics"));
    }
    if !reads.expressions.insert(id.0) {
        return Err(Error::Failed("native expression ownership is duplicated"));
    }
    let actual = body.expression(id).ok_or(Error::Failed("accepted expression has no native checked result"))?;
    match p.ast.exprs.get(id.0 as usize).ok_or(Error::Failed("accepted expression is absent"))? {
        ast::Expr::Int(value, _) if *value <= u64::MAX as u128 => {}
        ast::Expr::Unary { op: ast::UnOp::Neg, expr }
            if matches!(p.ast.exprs.get(expr.0 as usize), Some(ast::Expr::Int(_, _))) =>
        {
            expression(p, body, *expr, depth + 1, false, reads)?;
        }
        ast::Expr::Path(path) if path.len() == 1 && !path[0].has_args() => match actual {
            Resolution::Parameter(parameter) => {
                reads.parameters.insert(parameter);
            }
            Resolution::Local(local) => {
                *reads.locals.get_mut(&local).ok_or(Error::Unintegrated("local value resolution outside the completed bindings"))? = true;
            }
            _ => return Err(Error::Unintegrated("unrepresented value resolution")),
        },
        ast::Expr::Binary { op: ast::BinOp::BitAnd | ast::BinOp::BitOr | ast::BinOp::BitXor, lhs, rhs }
            if actual == Resolution::PrimitiveOperands =>
        {
            expression(p, body, *lhs, depth + 1, true, reads)?;
            expression(p, body, *rhs, depth + 1, true, reads)?;
        }
        ast::Expr::Call { callee, args } => {
            if !reads.expressions.insert(callee.0) {
                return Err(Error::Failed("native callee expression ownership is duplicated"));
            }
            if p.ast.is_parenthesized(*callee)
                || !matches!(p.ast.exprs.get(callee.0 as usize), Some(ast::Expr::Path(path)) if path.len() == 1 && !path[0].has_args())
            {
                return Err(Error::Unintegrated("indirect, generic or qualified call metadata"));
            }
            let function = match (body.expression(*callee), actual) {
                (Some(Resolution::Function(function)), _) => function,
                // The native checker's direct imported call records the
                // binding on the call node, with the original callee path.
                (_, Resolution::ImportedFunction(function)) => function,
                _ => return Err(Error::Unintegrated("call lacks an actual native function resolution")),
            };
            reads.calls.insert(function);
            for argument in args {
                expression(p, body, *argument, depth + 1, false, reads)?;
            }
        }
        _ => return Err(Error::Unintegrated("body operation or applicable lint completion")),
    }
    Ok(())
}
