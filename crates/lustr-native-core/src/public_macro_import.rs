//! Host procedural macros are compile-time inputs to native expansion.
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io::{self, BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::{Arc, Mutex},
};

use serde_json::{Value, json};

use crate::{
    diag::{CtxtId, Diag, Span},
    driver::imports::Dependency,
    expand::{Call, ExpandCtx, ProceduralExpansion, ProceduralMacros, Tt, tt_from_tree},
    lexer::{Delim, Tok, Token},
    protocol::ExecutableExtern,
};

use super::{Stamp, hash, stamp};

struct Definition {
    filename: PathBuf,
    source_digest: String,
    start: u32,
    end: u32,
}

impl Definition {
    fn load_source(&self) -> Option<String> {
        if !self.filename.is_absolute() || self.source_digest.is_empty() {
            return None;
        }
        let source = fs::read_to_string(&self.filename).ok()?;
        if self.source_digest.strip_prefix("sha256=")? != hash(source.as_bytes()) {
            return None;
        }
        // Match the pinned Rust source map's BOM removal and CRLF normalization.
        let source = source.strip_prefix('\u{feff}').unwrap_or(&source).replace("\r\n", "\n");
        source.get(self.start as usize..self.end as usize)?;
        Some(source)
    }
}

struct HostArtifact {
    input: ExecutableExtern,
    bytes: Arc<[u8]>,
    artifact_stamp: Stamp,
    receipt: PathBuf,
    receipt_stamp: Stamp,
    receipt_bytes: Arc<[u8]>,
    definitions: BTreeMap<String, Definition>,
}

pub(super) struct HostMacros {
    client: PathBuf,
    server: PathBuf,
    cwd: PathBuf,
    artifacts: BTreeMap<String, HostArtifact>,
    used: Mutex<BTreeSet<String>>,
    worker: Mutex<Option<Worker>>,
}

struct Worker {
    child: Child,
    input: Option<ChildStdin>,
    output: BufReader<ChildStdout>,
}

impl Worker {
    fn start(client: &Path, server: &Path) -> io::Result<Self> {
        let mut child = Command::new(client).arg(server).env_clear()
            .env("RUST_ANALYZER_USE_POSTCARD", "1")
            .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()?;
        let input = child.stdin.take().expect("procedural macro client stdin was piped");
        let output = child.stdout.take().expect("procedural macro client stdout was piped");
        Ok(Self { child, input: Some(input), output: BufReader::new(output) })
    }

    fn expand(&mut self, request: &Value) -> Result<Value, Box<dyn std::error::Error>> {
        let input = self.input.as_mut().expect("procedural macro client is active");
        serde_json::to_writer(&mut *input, request)?;
        input.write_all(b"\n")?;
        let mut reply = String::new();
        if self.output.read_line(&mut reply)? == 0 {
            return Err("procedural macro client exited without a reply".into());
        }
        let reply: Value = serde_json::from_str(&reply)?;
        if let Some(error) = reply["error"].as_str() {
            return Err(error.to_owned().into());
        }
        Ok(reply)
    }

    fn finish(&mut self) -> io::Result<()> {
        drop(self.input.take());
        let status = self.child.wait()?;
        if !status.success() {
            return Err(io::Error::other(format!("procedural macro client terminated with {status}")));
        }
        Ok(())
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}

pub(super) struct MacroRequest<'a>(&'a HostMacros);

impl MacroRequest<'_> {
    pub(super) fn finish(&self) -> io::Result<()> {
        let worker = self.0.worker.lock().unwrap().take();
        worker.map_or(Ok(()), |mut worker| worker.finish())
    }
}

impl Drop for MacroRequest<'_> {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}

impl HostMacros {
    pub(super) fn open(
        inputs: &[ExecutableExtern], installation: &Path, cwd: &Path,
    ) -> Result<Option<Arc<Self>>, String> {
        let mut artifacts = BTreeMap::new();
        for input in inputs.iter().filter(|input| input.is_host_macro()) {
            let artifact = HostArtifact::open(input)?;
            if artifacts.insert(input.metadata.alias.clone(), artifact).is_some() {
                return Err("duplicate host procedural macro alias".into());
            }
        }
        if artifacts.is_empty() {
            return Ok(None);
        }
        let runtime = installation.join("libexec/lustr-macro-expansion");
        let client = runtime.join("bin/lustr-expand-proc-macro");
        let server = runtime.join("bin/rust-analyzer-proc-macro-srv");
        if !client.is_file() || !server.is_file() {
            return Err("host procedural macro expansion is unavailable in this installation".into());
        }
        Ok(Some(Arc::new(Self { client, server, cwd: cwd.into(), artifacts, used: Mutex::default(), worker: Mutex::default() })))
    }

    pub(super) fn unchanged<'a>(&self, inputs: impl Iterator<Item = &'a ExecutableExtern>) -> bool {
        let mut count = 0;
        let unchanged = inputs.into_iter().all(|input| {
            count += 1;
            self.artifacts.get(&input.metadata.alias).is_some_and(|artifact| {
                artifact.input == *input
                    && stamp(&input.metadata.artifact).is_ok_and(|value| value == artifact.artifact_stamp)
                    && stamp(&artifact.receipt).is_ok_and(|value| value == artifact.receipt_stamp)
            })
        });
        unchanged && count == self.artifacts.len()
    }

    pub(super) fn dependencies(&self) -> impl Iterator<Item = Dependency> + '_ {
        self.artifacts.values().flat_map(|artifact| {
            [
                Dependency {
                    identity: artifact.input.metadata.artifact.to_string_lossy().into_owned(),
                    contents: artifact.bytes.clone(),
                },
                Dependency {
                    identity: artifact.receipt.to_string_lossy().into_owned(),
                    contents: artifact.receipt_bytes.clone(),
                },
            ]
        })
    }

    pub(super) fn begin_request(&self) -> MacroRequest<'_> {
        self.used.lock().unwrap().clear();
        MacroRequest(self)
    }

    pub(super) fn unused(&self) -> Vec<&str> {
        let used = self.used.lock().unwrap();
        self.artifacts.keys().filter(|alias| !used.contains(*alias)).map(String::as_str).collect()
    }
}

impl HostArtifact {
    fn open(input: &ExecutableExtern) -> Result<Self, String> {
        let path = &input.metadata.artifact;
        let artifact_stamp = stamp(path)?;
        let bytes: Arc<[u8]> = fs::read(path).map_err(|error| error.to_string())?.into();
        let mut receipt = path.as_os_str().to_owned();
        receipt.push(".lustr-spans");
        let receipt = PathBuf::from(receipt);
        let receipt_stamp = stamp(&receipt)?;
        let data: Arc<[u8]> = fs::read(&receipt).map_err(|error| error.to_string())?.into();
        if stamp(path)? != artifact_stamp || stamp(&receipt)? != receipt_stamp {
            return Err("procedural macro inputs changed while being opened".into());
        }
        let mut reader = data.strip_prefix(b"lustr-proc-macro-spans-v2\0")
            .ok_or("invalid procedural macro span receipt")?;
        if text(&mut reader)? != format!("sha256={}", hash(&bytes)) {
            return Err("procedural macro span receipt identifies different artifact bytes".into());
        }
        let mut definitions = BTreeMap::new();
        for _ in 0..number(&mut reader)? {
            let name = text(&mut reader)?;
            let filename = PathBuf::from(text(&mut reader)?);
            let start = number(&mut reader)?;
            let end = number(&mut reader)?;
            let source_digest = text(&mut reader)?;
            if start > end
                || definitions.insert(name, Definition { filename, source_digest, start, end }).is_some()
            {
                return Err("invalid procedural macro definition span".into());
            }
        }
        if !reader.is_empty() {
            return Err("trailing bytes in procedural macro span receipt".into());
        }
        Ok(Self { input: input.clone(), bytes, artifact_stamp, receipt, receipt_stamp,
            receipt_bytes: data, definitions })
    }
}

fn number(reader: &mut &[u8]) -> Result<u32, String> {
    let mut bytes = [0; 4];
    reader.read_exact(&mut bytes).map_err(|error| error.to_string())?;
    Ok(u32::from_le_bytes(bytes))
}

fn text(reader: &mut &[u8]) -> Result<String, String> {
    let size = number(reader)? as usize;
    let (value, rest) = reader.split_at_checked(size).ok_or("truncated procedural macro span receipt")?;
    *reader = rest;
    std::str::from_utf8(value).map(str::to_owned).map_err(|error| error.to_string())
}

impl ProceduralMacros for HostMacros {
    fn expand(&self, call: &Call, ctx: &ExpandCtx<'_>, strings: &mut Vec<String>)
        -> Option<Result<ProceduralExpansion, Diag>>
    {
        let [alias, name] = call.segs.as_slice() else { return None; };
        let artifact = self.artifacts.get(&alias.text)?;
        let definition = artifact.definitions.get(&name.text)?;
        let expanded = self.expand_macro(call, ctx, strings, artifact, definition);
        if expanded.is_ok() {
            self.used.lock().unwrap().insert(alias.text.clone());
        }
        Some(expanded.map_err(|error| {
            Diag::new("L0809", call.span, format!("cannot expand procedural macro: {error}"), "")
        }))
    }
}

impl HostMacros {
    fn expand_macro(
        &self, call: &Call, ctx: &ExpandCtx<'_>, strings: &mut Vec<String>,
        artifact: &HostArtifact, definition: &Definition,
    ) -> Result<ProceduralExpansion, Box<dyn std::error::Error>> {
        if call.span.ctxt != CtxtId::ROOT {
            return Err("source expansion from a non-root context is unrepresented".into());
        }
        let group = call.group_span;
        let bounds = group.lo as usize..group.hi as usize;
        let (open, close) = match call.delim {
            Delim::Paren => ('(', ')'),
            Delim::Bracket => ('[', ']'),
            Delim::Brace => ('{', '}'),
        };
        if group.ctxt != CtxtId::ROOT || group.file != call.span.file
            || ctx.source_file != Some(group.file)
            || group.lo < call.span.lo || group.hi > call.span.hi
            || ctx.src.get(bounds).is_none_or(|text| !text.starts_with(open) || !text.ends_with(close))
        {
            return Err("unrepresented macro argument source".into());
        }
        let mut environment: BTreeMap<String, String> = match &ctx.client.env {
            Some(environment) => environment.iter().cloned().collect(),
            None => std::env::vars_os().filter_map(|(name, value)| {
                Some((name.into_string().ok()?, value.into_string().ok()?))
            }).collect(),
        };
        environment.extend(ctx.cargo_env.iter().cloned());
        let request = json!({
            "dylib": artifact.input.metadata.artifact,
            "macro_name": call.name_text(), "working_directory": self.cwd,
            "definition": {"start": definition.start, "end": definition.end},
            "provider_file": definition.filename,
            "provider_source": definition.load_source(),
            "consumer_file": ctx.client.shown(&ctx.file_path),
            "consumer_local_file": ctx.real_path,
            "consumer_source": ctx.src,
            "environment": environment.into_iter().collect::<Vec<_>>(),
            "call": {"start": call.span.lo, "end": call.span.hi},
            "arguments": {"start": group.lo + 1, "end": group.hi - 1},
        });
        let report = {
            let mut worker = self.worker.lock().unwrap();
            if worker.is_none() {
                *worker = Some(Worker::start(&self.client, &self.server)?);
            }
            worker.as_mut().expect("procedural macro client started").expand(&request)?
        };
        let returned = report["tokens"].as_array().ok_or("missing procedural macro tokens")?;
        let tokens = returned_tokens(returned, call, ctx, strings)?;
        let environment = serde_json::from_value(report["environment"].clone())?;
        let paths: Vec<PathBuf> = serde_json::from_value(report["paths"].clone())?;
        let paths = paths.into_iter().map(|path| self.cwd.join(path)).collect();
        Ok(ProceduralExpansion { tokens, environment, paths })
    }
}

fn returned_span(value: &Value, call: &Call) -> Result<Span, Box<dyn std::error::Error>> {
    let offset = |field| -> Result<u32, Box<dyn std::error::Error>> {
        Ok(value[field].as_u64().ok_or("invalid procedural macro span")?.try_into()?)
    };
    let lo = offset("start")?;
    let hi = offset("end")?;
    if lo > hi || lo < call.span.lo || hi > call.span.hi {
        return Err("procedural macro token span is outside its call".into());
    }
    Ok(Span::new(call.span.file, lo, hi))
}

fn returned_tokens(
    mut values: &[Value], call: &Call, ctx: &ExpandCtx<'_>, strings: &mut Vec<String>,
) -> Result<Vec<Tt>, Box<dyn std::error::Error>> {
    let mut tokens: Vec<Token> = Vec::with_capacity(values.len());
    while let Some((value, rest)) = values.split_first() {
        let span = returned_span(&value["range"], call)?;
        let text = value["text"].as_str().ok_or("missing procedural macro token text")?;
        let (token, tail) = match value["kind"].as_str() {
            Some("punctuation") => returned_punctuation(values, call, ctx)?,
            Some("delimiter") => {
                let tok = match text {
                    "(" => Tok::LParen, ")" => Tok::RParen,
                    "[" => Tok::LBracket, "]" => Tok::RBracket,
                    "{" => Tok::LBrace, "}" => Tok::RBrace,
                    _ => return Err("invalid procedural macro delimiter".into()),
                };
                (Token { tok, span }, rest)
            }
            Some("literal") => {
                let lexed = crate::lexer::lex(text, call.span.file, 0, ctx.interner)
                    .map_err(|error| format!("{error:?}"))?;
                let [token, end] = lexed.tokens.as_slice() else {
                    return Err("procedural macro returned an unrepresented literal spelling".into());
                };
                let mut tok = token.tok;
                if end.tok != Tok::Eof || !matches!(tok, Tok::Int(..) | Tok::Str(..) | Tok::Char(..)
                    | Tok::Float(..) | Tok::ByteStr(..) | Tok::ByteChar(..))
                {
                    return Err("procedural macro returned an unrepresented literal kind".into());
                }
                tok.shift_strings(strings.len() as u32);
                strings.extend(lexed.strings);
                (Token { tok, span }, rest)
            }
            _ => return Err("invalid procedural macro token kind".into()),
        };
        if tokens.last().is_some_and(|previous| previous.tok == Tok::Gt && previous.span.hi == token.span.lo)
            && matches!(token.tok, Tok::Gt | Tok::Ge)
        {
            return Err("procedural macro punctuation spacing conflicts with its source ranges".into());
        }
        tokens.push(token);
        values = tail;
    }
    let (trees, balanced) = crate::lexer::build_trees(&tokens);
    if !balanced {
        return Err("unbalanced procedural macro output".into());
    }
    Ok(trees.iter().map(tt_from_tree).collect())
}

fn returned_punctuation<'a>(
    values: &'a [Value], call: &Call, ctx: &ExpandCtx<'_>,
) -> Result<(Token, &'a [Value]), Box<dyn std::error::Error>> {
    let mut bytes = [0; 3]; // Longest native punctuators are three ASCII bytes.
    let mut span = returned_span(&values[0]["range"], call)?;
    let mut best = None;
    for (index, value) in values.iter().take(bytes.len()).enumerate() {
        if value["kind"] != "punctuation" { break; }
        let Some([byte]) = value["text"].as_str().map(str::as_bytes) else {
            return Err("invalid procedural macro punctuation".into());
        };
        if !byte.is_ascii_punctuation() || b"()[]{}".contains(byte) {
            return Err("invalid procedural macro punctuator".into());
        }
        bytes[index] = *byte;
        let text = std::str::from_utf8(&bytes[..=index])?;
        span = span.to(returned_span(&value["range"], call)?);
        let tok = match text {
            ">>" => Some(Tok::Shr),
            ">>=" => Some(Tok::ShrEq),
            "<-" => return Err("unrepresented left-arrow procedural macro punctuator".into()),
            _ => crate::lexer::lex(text, call.span.file, 0, ctx.interner).ok().and_then(|lexed| {
                match lexed.tokens.as_slice() {
                    [token, end] if end.tok == Tok::Eof && token.span.hi as usize == text.len() => Some(token.tok),
                    _ => None,
                }
            }),
        };
        if let Some(tok) = tok {
            best = Some((Token { tok, span }, &values[index + 1..]));
        }
        if value["joint"].as_bool() != Some(true) { break; }
    }
    best.ok_or_else(|| "unrepresented procedural macro punctuation".into())
}
