// Cargo test descriptors and native entry thunks. The installed Rust libtest
// adapter owns scheduling, filtering, reporting and result classification.
use super::{Program, SourceMap, c_string_lit, names};
use crate::ast::{Ast, AttrInput, Expr};
use crate::check::collect::{FnLoc, has_native_termination};
use crate::def::LocalDefId;
use crate::lexer::{Tok, TokenTree};
use crate::ty::UNIT;
use std::fmt::Write;
use std::path::Path;

pub(crate) struct NativeTest {
    pub id: LocalDefId,
    name: String,
    file: String,
    line: u32,
    column: u32,
    ignored: Option<Option<String>>,
    panic: Option<Option<String>>,
    reports_status: bool,
}

pub(crate) fn collect(program: &Program, map: &SourceMap, cwd: &Path) -> Result<Vec<NativeTest>, String> {
    let mut tests = Vec::new();
    for (index, function) in program.fns.iter().enumerate() {
        let def = function.def(&program.ast);
        if !def.is_test {
            continue;
        }
        if !matches!(function.loc, FnLoc::Item(_))
            || !function.params.is_empty()
            || !has_native_termination(program, function.ret)
            || !def.generics.params.is_empty()
        {
            return Err(format!("native Cargo harness does not represent the signature of {}", program.display_path(function.name)));
        }
        let span = map.call_site(def.name_span);
        let (line, column) = map.line_col(span);
        let original_file = Path::new(map.path(span.file));
        let file = original_file.strip_prefix(cwd).unwrap_or(original_file).to_string_lossy().into_owned();
        let mut ignored = None;
        let mut panic = None;
        for attr in def.attrs.iter().filter(|attr| attr.path.len() == 1) {
            match program.name(attr.path[0].name) {
                "ignore" if ignored.is_none() => ignored = Some(attr_string(&attr.input, &program.ast)),
                "should_panic" if panic.is_none() => panic = Some(attr_string(&attr.input, &program.ast)),
                _ => {}
            }
        }
        let name = program.display_path(function.name);
        let reports_status = function.ret != UNIT;
        if reports_status && panic.is_some() {
            return Err(format!("test {name} must return () to use #[should_panic]"));
        }
        if [&ignored, &panic].iter().any(|text| text.as_ref().and_then(|value| value.as_ref()).is_some_and(|value| value.contains('\0'))) {
            return Err(format!("native Cargo test attribute in {name} contains a NUL byte"));
        }
        tests.push(NativeTest { id: LocalDefId(index as u32), name, file, line, column, ignored, panic, reports_status });
    }
    Ok(tests)
}

pub(crate) fn attr_string(input: &AttrInput, ast: &Ast) -> Option<String> {
    fn tree_string(trees: &[TokenTree], ast: &Ast) -> Option<String> {
        trees.iter().find_map(|tree| match tree {
            TokenTree::Token(token) => match token.tok {
                Tok::Str(i, _) => ast.strings.get(i as usize).cloned(),
                _ => None,
            },
            TokenTree::Delim(_, children, _) => tree_string(children, ast),
        })
    }
    match input {
        AttrInput::None => None,
        AttrInput::Eq(expr) => match ast.expr(*expr) {
            Expr::Str(i) => ast.strings.get(*i as usize).cloned(),
            _ => None,
        },
        AttrInput::List(_, trees) => tree_string(trees, ast),
    }
}

pub(super) fn main_declarations(out: &mut String, tests: &[NativeTest], slots: &[u32]) {
    out.push_str(
        "typedef void (*lust_test_report)(void*, const uint8_t*, size_t);\n\
        int lust_test_invoke(void (*body)(void*), void*, lust_test_report);\n\
        typedef struct {\n\
          const char *name, *file, *ignore_reason, *expected;\n\
          uint32_t line, column, flags;\n\
          int (*run)(void*, lust_test_report);\n\
        } lust_test_descriptor;\n\
        int lustr_test_main(int argc, char **argv, size_t count, const lust_test_descriptor*);\n",
    );
    for (i, test) in tests.iter().enumerate() {
        let slot = slots[test.id.0 as usize];
        if test.reports_status {
            let symbol = termination_entry(slot);
            let _ = writeln!(out, "int {symbol}(void);\nstatic void lust_test_body_{i}(void *result) {{ *(int*)result = {symbol}(); }}");
        } else {
            let symbol = names::body(slot);
            let _ =
                writeln!(out, "lust_Unit {symbol}(void);\nstatic void lust_test_body_{i}(void *unused) {{ (void)unused; {symbol}(); }}");
        }
        let _ = writeln!(
            out,
            "static int lust_test_run_{i}(void *ctx, lust_test_report report) {{\n\
               return lust_test_invoke(lust_test_body_{i}, ctx, report);\n\
             }}"
        );
    }
    let _ = writeln!(out, "static const lust_test_descriptor lust_tests[{}] = {{", tests.len().max(1));
    for (i, test) in tests.iter().enumerate() {
        let opt_string = |value: &Option<Option<String>>| {
            value.as_ref().and_then(|text| text.as_ref()).map_or_else(|| "0".into(), |text| c_string_lit(text.as_bytes()))
        };
        let flags = u8::from(test.ignored.is_some()) | (u8::from(test.panic.is_some()) << 1);
        let _ = writeln!(
            out,
            "  {{ {}, {}, {}, {}, {}, {}, {flags}, lust_test_run_{i} }},",
            c_string_lit(test.name.as_bytes()),
            c_string_lit(test.file.as_bytes()),
            opt_string(&test.ignored),
            opt_string(&test.panic),
            test.line,
            test.column
        );
    }
    out.push_str("};\n");
}

pub(super) fn termination_entry(slot: u32) -> String {
    format!("lust_test_termination_{}", names::fn_tag(slot))
}

/// Joins the original native catch to the Rust libtest adapter. Copy the
/// message while its native payload is alive, then drop it on the native side.
pub(crate) const CATCH_ADAPTER: &str = "
int lust_test_invoke(void (*body)(void*), void *ctx,
                      void (*report)(void*, const uint8_t*, size_t)) {
    int status = 0;
    if (!lust_unw_try(body, &status)) return status ? 256 + status : 0;
    lust_exc exc;
    lust_unw_take_any(&exc);
    lust_unw_get()->count--;
    int text = exc.kind != LUST_EXC_BOX;
    if (!text) {
        const void *const *vt = exc.box.vt;
        uint64_t lo = (uint64_t)(uintptr_t)vt[5], hi = (uint64_t)(uintptr_t)vt[6];
        text = (lo == lust_any_ids[0] && hi == lust_any_ids[1])
            || (lo == lust_any_ids[2] && hi == lust_any_ids[3]);
    }
    if (text) {
        lust_str message = lust_exc_text(&exc);
        report(ctx, message.ptr, message.len);
    }
    lust_exc_drop(&exc);
    return text ? 1 : 2;
}
";
