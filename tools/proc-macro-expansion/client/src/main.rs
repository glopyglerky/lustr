use paths::AbsPathBuf;
use proc_macro_api::bidirectional_protocol::msg::{SubRequest, SubResponse};
use proc_macro_api::bidirectional_protocol::reject_subrequests;
use proc_macro_api::{ExpansionInputs, MacroDylib, ProcMacroClient, ProcMacroKind};
use salsa::plumbing::AsId;
use serde::{Deserialize, Serialize};
use span::{
    Edition, EditionedFileId, FileId, MacroCallId, ROOT_ERASED_FILE_AST_ID, Span, SpanAnchor,
    SyntaxContext, TextRange, Transparency,
};
use std::cell::RefCell;
use std::error::Error;
use std::io::{self, BufRead, Write};
use tt::{DelimSpan, Delimiter, DelimiterKind, Leaf, LitKind, Spacing, TokenTree, TopSubtree, TtElement, TtIter};

#[derive(Deserialize, Serialize)]
struct Range {
    start: u32,
    end: u32,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Token {
    Literal { text: String, range: Range },
    Identifier { text: String, range: Range },
    Punctuation { text: char, joint: bool, range: Range },
    Delimiter { text: char, range: Range },
}

#[derive(Deserialize)]
struct Request {
    dylib: String,
    macro_name: String,
    working_directory: String,
    definition: Range,
    provider_file: String,
    provider_source: Option<String>,
    call: Range,
    consumer_file: String,
    consumer_local_file: String,
    consumer_source: String,
    environment: Vec<(String, String)>,
    arguments: Range,
}

#[salsa::interned]
struct MacroInvocation {
    consumer: String,
    dylib: String,
    name: String,
    start: u32,
    end: u32,
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

fn arguments(request: &Request, call: Span) -> Result<TopSubtree, &'static str> {
    let source = snapshot_slice(&request.consumer_source, &request.arguments)
        .ok_or("invalid procedural macro argument range")?;
    let tree = syntax_bridge::parse_to_token_tree(Edition::Edition2024, call.anchor, call.ctx, source)
        .ok_or("cannot tokenize procedural macro arguments")?;
    let base = span::TextSize::from(request.arguments.start);
    let shift = |span: &mut Span| {
        span.range = TextRange::new(span.range.start() + base, span.range.end() + base);
    };
    let tokens = tree.view().as_token_trees().iter_flat_tokens().map(|mut tree| {
        match &mut tree {
            TokenTree::Leaf(Leaf::Ident(token)) => shift(&mut token.span),
            TokenTree::Leaf(Leaf::Literal(token)) => shift(&mut token.span),
            TokenTree::Leaf(Leaf::Punct(token)) => shift(&mut token.span),
            TokenTree::Subtree(group) => {
                shift(&mut group.delimiter.open);
                shift(&mut group.delimiter.close);
            }
        }
        tree
    }).collect();
    let mut input = TopSubtree::from_serialized(tokens);
    if input.top_subtree().delimiter.kind == DelimiterKind::Invisible {
        input.set_top_subtree_delimiter_span(DelimSpan::from_single(call));
    } else {
        // The upstream converter omits the invisible root when the source is
        // one group. Preserve that group's own opening and closing spans.
        input = TopSubtree::from_token_trees(Delimiter::invisible_spanned(call), input.as_token_trees());
    }
    Ok(input)
}

fn output_range(request: &Request, consumer: EditionedFileId, call_context: SyntaxContext, span: Span) -> Result<Range, &'static str> {
    if span.anchor.file_id != consumer
        || span.anchor.ast_id != ROOT_ERASED_FILE_AST_ID
        || !(span.ctx == call_context || span.ctx == SyntaxContext::root(Edition::Edition2024))
    {
        return Err("unrepresented returned span");
    }
    let range = Range { start: span.range.start().into(), end: span.range.end().into() };
    if snapshot_slice(&request.consumer_source, &range).is_none()
        || range.start < request.call.start || range.end > request.call.end
    {
        return Err("returned span is outside the consumer call");
    }
    Ok(range)
}

fn output_tokens(
    tokens: TtIter<'_>, request: &Request, consumer: EditionedFileId, call_context: SyntaxContext,
) -> Result<Vec<Token>, &'static str> {
    let range = |span| output_range(request, consumer, call_context, span);
    let mut pending = vec![(tokens, None)];
    let mut output = Vec::new();
    while let Some((tokens, _)) = pending.last_mut() {
        let Some(tree) = tokens.next() else {
            if let Some((_, Some(close))) = pending.pop() {
                output.push(close);
            }
            continue;
        };
        let token = match tree {
            TtElement::Leaf(Leaf::Literal(literal)) if !matches!(literal.kind, LitKind::Err(_)) => {
                Token::Literal { text: literal.to_string(), range: range(literal.span)? }
            }
            TtElement::Leaf(Leaf::Ident(identifier)) => {
                Token::Identifier { text: identifier.to_string(), range: range(identifier.span)? }
            }
            TtElement::Leaf(Leaf::Punct(token)) => {
                Token::Punctuation { text: token.char, joint: token.spacing == Spacing::Joint, range: range(token.span)? }
            }
            TtElement::Subtree(group, inner) => {
                let (open, close) = match group.delimiter.kind {
                    DelimiterKind::Parenthesis => ('(', ')'),
                    DelimiterKind::Bracket => ('[', ']'),
                    DelimiterKind::Brace => ('{', '}'),
                    DelimiterKind::Invisible => return Err("unrepresented invisible procedural macro group"),
                };
                pending.push((inner, Some(Token::Delimiter { text: close, range: range(group.delimiter.close)? })));
                Token::Delimiter { text: open, range: range(group.delimiter.open)? }
            }
            _ => return Err("unrepresented returned procedural macro token"),
        };
        output.push(token);
    }
    Ok(output)
}

fn expand(client: &ProcMacroClient, database: &dyn salsa::Database, mut request: Request) -> Result<serde_json::Value, Box<dyn Error>> {
    let dylib = AbsPathBuf::try_from(request.dylib.as_str()).map_err(|_| "artifact path is not absolute")?;
    let macros = client.load_dylib(MacroDylib::new(dylib)).map_err(|error| error.to_string())?;
    let expansion = macros.iter().find(|candidate| {
        candidate.name() == request.macro_name && candidate.kind() == ProcMacroKind::Bang
    }).ok_or("unresolved function-like procedural macro")?;
    let provider_file = FileId::from_raw(1);
    let consumer_file = FileId::from_raw(2);
    let provider = EditionedFileId::new(provider_file, Edition::Edition2024);
    let consumer = EditionedFileId::new(consumer_file, Edition::Edition2024);
    if request.definition.start > request.definition.end
        || snapshot_slice(&request.consumer_source, &request.call).is_none()
    {
        return Err("invalid procedural macro source range".into());
    }
    let invocation = MacroInvocation::new(database, request.consumer_local_file.clone(), request.dylib.clone(),
        request.macro_name.clone(), request.call.start, request.call.end);
    let mark = MacroCallId(invocation.as_id());
    let edition = Edition::Edition2024;
    let root = SyntaxContext::root(edition);
    let site = |transparency| SyntaxContext::new(database, mark, transparency, edition, root,
        (transparency != Transparency::Opaque).then_some(root),
        (transparency == Transparency::Transparent).then_some(root));
    let call = root_span(consumer, &request.call);
    let input = arguments(&request, call)?;
    let call_site = Span { ctx: site(Transparency::Transparent), ..call };
    let mixed_site = Span { ctx: site(Transparency::SemiOpaque), ..call };
    let def_site = Span { ctx: site(Transparency::Opaque), ..root_span(provider, &request.definition) };
    let environment = std::mem::take(&mut request.environment);
    let unsupported = RefCell::new(None::<String>);
    let reject = |request_kind| reject_subrequests(request_kind).inspect_err(|error| {
        unsupported.borrow_mut().get_or_insert_with(|| error.to_string());
    });
    let callback = |request_kind| match request_kind {
        SubRequest::SourceText { file_id, ast_id, start, end }
            if ast_id == ROOT_ERASED_FILE_AST_ID.into_raw() =>
        {
            let source = if file_id == consumer.as_u32() {
                Some(request.consumer_source.as_str())
            } else if file_id == provider.as_u32() {
                request.provider_source.as_deref()
            } else {
                return reject(request_kind);
            };
            let text = source.and_then(|source| snapshot_slice(source, &Range { start, end })).map(str::to_owned);
            Ok(SubResponse::SourceTextResult { text })
        }
        SubRequest::FilePath { file_id } if file_id == consumer_file.index() => {
            Ok(SubResponse::FilePathResult { name: request.consumer_file.clone() })
        }
        SubRequest::FilePath { file_id } if file_id == provider_file.index() => {
            Ok(SubResponse::FilePathResult { name: request.provider_file.clone() })
        }
        SubRequest::LocalFilePath { file_id } if file_id == consumer_file.index() => {
            Ok(SubResponse::LocalFilePathResult { name: Some(request.consumer_local_file.clone()) })
        }
        SubRequest::LocalFilePath { file_id } if file_id == provider_file.index() => {
            Ok(SubResponse::LocalFilePathResult { name: None })
        }
        other => reject(other),
    };
    let observed = RefCell::<Option<ExpansionInputs>>::new(None);
    let note = |inputs| assert!(observed.replace(Some(inputs)).is_none());
    let expanded = expansion.expand_with_inputs(
        input.view(), None, environment, def_site, call_site, mixed_site,
        request.working_directory.clone(), &callback, &note,
    );
    if let Some(error) = unsupported.take() {
        return Err(error.into());
    }
    let expanded = expanded.map_err(|error| error.to_string())?
        .map_err(|error| format!("procedural macro expansion failed: {error}"))?;
    let tokens = output_tokens(expanded.view().strip_invisible().iter(), &request, consumer, call_site.ctx)?;
    let observed = observed.into_inner().ok_or("missing procedural macro dependency report")?;
    Ok(serde_json::json!({
        "tokens": tokens, "environment": observed.environment, "paths": observed.paths,
    }))
}

fn main() -> Result<(), Box<dyn Error>> {
    let arg = std::env::args().nth(1).ok_or("missing procedural macro server path")?;
    let server = AbsPathBuf::try_from(arg.as_str()).map_err(|_| "server path is not absolute")?;
    let version = semver::Version::parse("1.102.0-nightly")?;
    // Keep the host dylibs in one process for this compiler invocation.
    // Each expansion still receives the actual caller's environment.
    let client = ProcMacroClient::spawn(
        &server, [("RUST_ANALYZER_USE_POSTCARD", &None::<String>)], Some(&version), 1,
    );
    let database = salsa::DatabaseImpl::new();
    let mut output = io::BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let line = line?;
        let result = match &client {
            Ok(client) => serde_json::from_str(&line).map_err(Into::into)
                .and_then(|request| expand(client, &database, request)),
            Err(error) => Err(error.to_string().into()),
        };
        let reply = result.unwrap_or_else(|error| serde_json::json!({"error": error.to_string()}));
        serde_json::to_writer(&mut output, &reply)?;
        output.write_all(b"\n")?;
        output.flush()?;
    }
    Ok(())
}
