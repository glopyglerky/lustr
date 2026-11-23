use paths::AbsPathBuf;
use proc_macro_api::bidirectional_protocol::msg::{SubRequest, SubResponse};
use proc_macro_api::bidirectional_protocol::reject_subrequests;
use proc_macro_api::{ExpansionInputs, MacroDylib, ProcMacroClient, ProcMacroKind};
use serde::{Deserialize, Serialize};
use span::{
    Edition, EditionedFileId, FileId, ROOT_ERASED_FILE_AST_ID, Span, SpanAnchor, SyntaxContext,
    TextRange,
};
use std::cell::RefCell;
use std::error::Error;
use std::io;
use tt::{Delimiter, Leaf, LitKind, TopSubtreeBuilder};

#[derive(Deserialize, Serialize)]
struct Range {
    start: u32,
    end: u32,
}

#[derive(Deserialize, Serialize)]
struct Token {
    text: String,
    range: Range,
}

#[derive(Deserialize)]
struct Request {
    server: String,
    dylib: String,
    macro_name: String,
    working_directory: String,
    definition: Range,
    provider_source: Option<String>,
    call: Range,
    consumer_source: String,
    tokens: Vec<Token>,
}

fn snapshot_slice<'a>(source: &'a str, range: &Range) -> Option<&'a str> {
    source.get(range.start as usize..range.end as usize)
}

fn root_span(file_id: EditionedFileId, range: &Range) -> Span {
    Span {
        range: TextRange::new(range.start.into(), range.end.into()),
        anchor: SpanAnchor { file_id, ast_id: ROOT_ERASED_FILE_AST_ID },
        ctx: SyntaxContext::root(Edition::Edition2024),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let request: Request = serde_json::from_reader(io::stdin().lock())?;
    let server = AbsPathBuf::try_from(request.server.as_str()).map_err(|_| "server path is not absolute")?;
    let dylib = AbsPathBuf::try_from(request.dylib.as_str()).map_err(|_| "artifact path is not absolute")?;
    let version = semver::Version::parse("1.102.0-nightly")?;
    let client =
        ProcMacroClient::spawn(&server, std::iter::empty::<(&str, &Option<String>)>(), Some(&version), 1)?;
    let macros = client.load_dylib(MacroDylib::new(dylib)).map_err(|error| error.to_string())?;
    let expansion = macros.iter().find(|candidate| {
        candidate.name() == request.macro_name && candidate.kind() == ProcMacroKind::Bang
    }).ok_or("unresolved function-like procedural macro")?;
    let provider = EditionedFileId::new(FileId::from_raw(1), Edition::Edition2024);
    let consumer = EditionedFileId::new(FileId::from_raw(2), Edition::Edition2024);
    if request.definition.start > request.definition.end
        || snapshot_slice(&request.consumer_source, &request.call).is_none()
    {
        return Err("invalid procedural macro source range".into());
    }
    let def_site = root_span(provider, &request.definition);
    let call_site = root_span(consumer, &request.call);
    let input = request.tokens.iter().map(|token| {
        if snapshot_slice(&request.consumer_source, &token.range) != Some(&token.text)
            || token.range.start < request.call.start || token.range.end > request.call.end
        {
            return Err("procedural macro input token is outside the call snapshot");
        }
        let literal = tt::token_to_literal(&token.text, root_span(consumer, &token.range));
        if literal.kind != LitKind::Integer || !literal.suffix().is_empty() {
            return Err("unrepresented procedural macro input token");
        }
        Ok(Leaf::Literal(literal))
    }).collect::<Result<Vec<_>, _>>()?;
    let mut tree = TopSubtreeBuilder::new(Delimiter::invisible_spanned(call_site));
    for leaf in input {
        tree.push(leaf);
    }
    let input = tree.build();
    let callback = |request_kind| match request_kind {
        SubRequest::SourceText { file_id, ast_id, start, end }
            if ast_id == ROOT_ERASED_FILE_AST_ID.into_raw() =>
        {
            let source = if file_id == consumer.as_u32() {
                Some(request.consumer_source.as_str())
            } else if file_id == provider.as_u32() {
                request.provider_source.as_deref()
            } else {
                return reject_subrequests(request_kind);
            };
            let text = source.and_then(|source| snapshot_slice(source, &Range { start, end })).map(str::to_owned);
            Ok(SubResponse::SourceTextResult { text })
        }
        other => reject_subrequests(other),
    };
    let observed = RefCell::<Option<ExpansionInputs>>::new(None);
    let note = |inputs| assert!(observed.replace(Some(inputs)).is_none());
    let expanded = expansion.expand_with_inputs(
        input.view(), None, Vec::new(), def_site, call_site, call_site,
        request.working_directory.clone(), &callback, &note,
    ).map_err(|error| error.to_string())?
        .map_err(|error| format!("procedural macro panicked: {error}"))?;
    let tokens = expanded.view().strip_invisible().iter_flat_tokens().map(|token| {
        let tt::TokenTree::Leaf(Leaf::Literal(literal)) = token else {
            return Err("unrepresented returned procedural macro token");
        };
        if literal.kind != LitKind::Integer || !literal.suffix().is_empty()
            || literal.span.anchor.file_id != consumer
            || literal.span.anchor.ast_id != ROOT_ERASED_FILE_AST_ID
            || literal.span.ctx != SyntaxContext::root(Edition::Edition2024)
        {
            return Err("unrepresented returned value or span");
        }
        let range = Range { start: literal.span.range.start().into(), end: literal.span.range.end().into() };
        if snapshot_slice(&request.consumer_source, &range).is_none()
            || range.start < request.call.start || range.end > request.call.end
        {
            return Err("returned span is outside the consumer call");
        }
        Ok(Token { text: literal.text().to_owned(), range })
    }).collect::<Result<Vec<_>, _>>()?;
    let observed = observed.into_inner().ok_or("missing procedural macro dependency report")?;
    serde_json::to_writer(io::stdout().lock(), &serde_json::json!({
        "tokens": tokens, "environment": observed.environment, "paths": observed.paths,
    }))?;
    Ok(())
}
