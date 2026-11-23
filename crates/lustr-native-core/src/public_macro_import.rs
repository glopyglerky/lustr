//! Host procedural macros are compile-time inputs to native expansion.
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
};

use serde_json::{Value, json};

use crate::{
    diag::{CtxtId, Diag, Span},
    driver::imports::Dependency,
    expand::{Call, ExpandCtx, ProceduralExpansion, ProceduralMacros, Tt},
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
        Ok(Some(Arc::new(Self { client, server, cwd: cwd.into(), artifacts, used: Mutex::default() })))
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

    pub(super) fn clear_uses(&self) {
        self.used.lock().unwrap().clear();
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
            "server": self.server, "dylib": artifact.input.metadata.artifact,
            "macro_name": call.name_text(), "working_directory": self.cwd,
            "definition": {"start": definition.start, "end": definition.end},
            "provider_source": definition.load_source(),
            "consumer_source": ctx.src,
            "environment": environment.into_iter().collect::<Vec<_>>(),
            "call": {"start": call.span.lo, "end": call.span.hi},
            "arguments": {"start": group.lo + 1, "end": group.hi - 1},
        });
        let mut child = Command::new(&self.client).env_clear().env("RUST_ANALYZER_USE_POSTCARD", "1")
            .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
        let sent = serde_json::to_writer(child.stdin.take().unwrap(), &request);
        let reply = child.wait_with_output()?;
        sent?;
        if !reply.status.success() {
            return Err(String::from_utf8_lossy(&reply.stderr).into_owned().into());
        }
        let report: Value = serde_json::from_slice(&reply.stdout)?;
        let returned = report["tokens"].as_array().ok_or("missing procedural macro tokens")?;
        let tokens = returned.iter().map(|value| returned_token(value, call, ctx, strings))
            .collect::<Result<Vec<_>, _>>()?;
        let environment = serde_json::from_value(report["environment"].clone())?;
        let paths: Vec<PathBuf> = serde_json::from_value(report["paths"].clone())?;
        let paths = paths.into_iter().map(|path| self.cwd.join(path)).collect();
        Ok(ProceduralExpansion { tokens, environment, paths })
    }
}

fn returned_token(
    value: &Value, call: &Call, ctx: &ExpandCtx<'_>, strings: &mut Vec<String>,
) -> Result<Tt, Box<dyn std::error::Error>> {
    let text = value["text"].as_str().ok_or("missing procedural macro token text")?;
    let offset = |field| -> Result<u32, Box<dyn std::error::Error>> {
        Ok(value["range"][field].as_u64().ok_or("invalid procedural macro span")?.try_into()?)
    };
    let lo = offset("start")?;
    let hi = offset("end")?;
    if lo > hi || lo < call.span.lo || hi > call.span.hi {
        return Err("procedural macro token span is outside its call".into());
    }
    let lexed = crate::lexer::lex(text, call.span.file, 0, ctx.interner)
        .map_err(|error| format!("{error:?}"))?;
    let [token, end] = lexed.tokens.as_slice() else {
        return Err("procedural macro returned an unrepresented token spelling".into());
    };
    let punctuation = value["punctuation"].as_bool().ok_or("missing procedural macro token kind")?;
    let mut token_kind = token.tok;
    if punctuation {
        if text.len() != 1 || !text.as_bytes()[0].is_ascii_punctuation() {
            return Err("unrepresented procedural macro punctuation".into());
        }
    } else if !matches!(token_kind, Tok::Int(..)) {
        return Err("procedural macro returned an unrepresented literal kind".into());
    }
    if end.tok != Tok::Eof {
        return Err("procedural macro returned trailing tokens".into());
    }
    token_kind.shift_strings(strings.len() as u32);
    strings.extend(lexed.strings);
    let span = Span::new(call.span.file, lo, hi);
    Ok(Tt::Tok(Token { tok: token_kind, span }))
}
