//! The bounded public scalar-library request. Paths for delivered artifacts are
//! per-request inputs; they never join the retained semantic context.
use std::path::{Path, PathBuf};

pub const SDK_RECEIPT: &str = "4e400b263bf9923616dea44ead2d77213703177d3bfaf6becd6a19a4c5a657a7";
pub const TARGET: &str = "x86_64-unknown-linux-gnu";
pub const SDK_RECEIPT_PATH: &str = "lib/lustr-metadata/sdk-receipt.json";

/// A quoted Cargo feature name that can be passed back to Cargo as one
/// unqualified feature, with no decoding or reinterpretation.
pub(crate) fn cargo_feature_name(literal: &str) -> Option<&str> {
    let name = literal.strip_prefix('"')?.strip_suffix('"')?;
    (!name.is_empty() && !name.starts_with('-')
        && !name.chars().any(|c| c.is_whitespace() || c.is_control() || matches!(c, ',' | '/' | '"' | '\\')))
        .then_some(name)
}

fn cargo_check_cfg(value: &str) -> bool {
    if value == "cfg(docsrs,test)" {
        return true;
    }
    let Some(features) = value.strip_prefix("cfg(feature, values(").and_then(|s| s.strip_suffix("))")) else {
        return false;
    };
    features.is_empty() || features.split(',').all(|literal| cargo_feature_name(literal.trim()).is_some())
}

/// This artifact adapter has only the actual Linux GNU x64 host implementation.
/// An omitted rustc --target denotes that compiled host; it is never relabeled.
pub const fn supported_host()->bool {
    cfg!(all(target_arch="x86_64",target_os="linux",target_env="gnu",target_vendor="unknown",
        target_pointer_width="64",target_endian="little"))
}

/// Artifact/lint requests do not change the native semantic session. Only the
/// unstable -Z embedding option is admitted; the stable -C channel is separate
/// upstream and remains unrepresented. None is the actual absent option.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LibraryOptions {
    pub embed_metadata: Option<bool>,
    pub force_warn_unused_crate_dependencies: bool,
    pub debug: DebugOptions,
}

/// The pinned Rust debug levels. Limited and line tables are distinct.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DebugInfo {
    #[default]
    None,
    LineDirectivesOnly,
    LineTablesOnly,
    Limited,
    Full,
}

impl DebugInfo {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "0" | "none" => Self::None,
            "line-directives-only" => Self::LineDirectivesOnly,
            "line-tables-only" => Self::LineTablesOnly,
            "1" | "limited" => Self::Limited,
            "2" | "full" => Self::Full,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SplitDebug {
    #[default]
    Off,
    Packed,
    Unpacked,
}

impl SplitDebug {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "off" => Self::Off,
            "packed" => Self::Packed,
            "unpacked" => Self::Unpacked,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Strip {
    #[default]
    None,
    DebugInfo,
    Symbols,
}

/// Rust's compiler optimization levels, also supported by host Clang.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum OptLevel {
    #[default]
    None,
    Less,
    More,
    Aggressive,
    Size,
    MinSize,
}

impl OptLevel {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "0" => Self::None,
            "1" => Self::Less,
            "2" => Self::More,
            "3" => Self::Aggressive,
            "s" => Self::Size,
            "z" => Self::MinSize,
            _ => return None,
        })
    }

    pub(crate) fn value(self) -> &'static str {
        match self {
            Self::None => "0",
            Self::Less => "1",
            Self::More => "2",
            Self::Aggressive => "3",
            Self::Size => "s",
            Self::MinSize => "z",
        }
    }
}

impl Strip {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "none" => Self::None,
            "debuginfo" => Self::DebugInfo,
            "symbols" => Self::Symbols,
            _ => return None,
        })
    }
}

/// Output policy for this request, independent of its semantic session.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DebugOptions {
    pub info: DebugInfo,
    pub split: SplitDebug,
    pub strip: Strip,
}

impl DebugOptions {
    const LEVELS: [DebugInfo; 5] = [
        DebugInfo::None, DebugInfo::LineDirectivesOnly, DebugInfo::LineTablesOnly,
        DebugInfo::Limited, DebugInfo::Full,
    ];
    const SPLIT: [SplitDebug; 3] = [SplitDebug::Off, SplitDebug::Packed, SplitDebug::Unpacked];
    const STRIP: [Strip; 3] = [Strip::None, Strip::DebugInfo, Strip::Symbols];

    pub fn encode(self) -> [u8; 3] {
        [b'0' + self.info as u8, b'0' + self.split as u8, b'0' + self.strip as u8]
    }

    pub(crate) fn decode(value: &[u8]) -> Option<Self> {
        let [level, split, strip] = *value else { return None };
        Some(Self {
            info: *Self::LEVELS.get(level.checked_sub(b'0')? as usize)?,
            split: *Self::SPLIT.get(split.checked_sub(b'0')? as usize)?,
            strip: *Self::STRIP.get(strip.checked_sub(b'0')? as usize)?,
        })
    }

    fn supported(self, check: bool) -> bool {
        check || (self.info == DebugInfo::None && self.split == SplitDebug::Off && self.strip != Strip::Symbols)
    }

    pub fn native_rust(self) -> bool {
        self == (Self { info: DebugInfo::Full, split: SplitDebug::Off, strip: Strip::None })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ExternMetadata {
    pub alias: String,
    pub artifact: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ExecutableExtern {
    pub metadata: ExternMetadata,
    pub archive: Option<PathBuf>,
}

impl ExecutableExtern {
    fn valid(&self) -> bool {
        self.metadata.valid() && self.archive.as_ref().is_none_or(|archive| {
            archive.is_absolute() && archive.extension().is_some_and(|ext| ext == "rlib")
                && archive.file_stem() == self.metadata.artifact.file_stem()
                && archive.to_str().is_some_and(|s| !s.contains(['\0', '\n', '\r']))
        })
    }
}

impl ExternMetadata {
    pub fn parse(argument: &str) -> Option<Self> {
        let (alias, artifact) = argument.split_once('=')?;
        let value = Self { alias: alias.into(), artifact: std::path::absolute(artifact).ok()? };
        value.valid().then_some(value)
    }

    fn valid(&self) -> bool {
        let alias = self.alias.as_str();
        identifier(alias) && !matches!(alias, "core" | "alloc" | "std" | "compiler_builtins")
            && self.artifact.is_absolute() && self.artifact.extension().is_some_and(|ext| ext == "rmeta")
            && self.artifact.to_str().is_some_and(|s| !s.contains(['\0', '\n', '\r']))
    }
}

fn identifier(s: &str) -> bool {
    !s.is_empty() && s.bytes().enumerate().all(|(i, c)|
        c == b'_' || c.is_ascii_alphabetic() || i != 0 && c.is_ascii_digit())
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LibraryKey {
    pub crate_name: String,
    pub metadata: String,
    pub extra_filename: String,
    pub source_path: PathBuf,
    pub working_directory: PathBuf,
    pub check_cfg: Vec<String>,
    pub search_paths: Vec<PathBuf>,
    pub external: Option<ExternMetadata>,
    pub executable_dependencies: Option<Vec<ExecutableExtern>>,
    pub optimization: OptLevel,
    /// Cargo supplies cfg(test), and the crate supplies its own main.
    pub custom_test: bool,
    pub sdk_receipt: String,
}
impl LibraryKey {
    pub fn valid(&self) -> bool {
        supported_host() && identifier(&self.crate_name)
            && !self.metadata.is_empty() && self.metadata.len() <= 128
            && self.metadata.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
            && (self.extra_filename.is_empty() || self.extra_filename.starts_with('-'))
            && self.extra_filename.len() <= 128
            && self.extra_filename.bytes().all(|c| c.is_ascii_alphanumeric() || b"_-".contains(&c))
            && self.source_path.is_absolute() && self.working_directory.is_absolute()
            && self.source_path.to_str().is_some_and(|s| !s.contains(['\0','\n','\r']))
            && self.working_directory.to_str().is_some_and(|s| !s.contains(['\0','\n','\r']))
            && self.check_cfg.len() <= 2
            && self.check_cfg.iter().all(|s| cargo_check_cfg(s))
            && self.search_paths.iter().all(|p| p.is_absolute())
            && self.external.as_ref().is_none_or(ExternMetadata::valid)
            && self.executable_dependencies.as_ref().is_none_or(|dependencies| {
                self.external.is_none() && dependencies.iter().all(ExecutableExtern::valid)
                    && dependencies.windows(2).all(|pair| pair[0].metadata.alias < pair[1].metadata.alias)
            })
            && (!self.custom_test || self.is_executable())
            && self.sdk_receipt == SDK_RECEIPT
    }
    pub fn archive_name(&self) -> String { format!("lib{}{}.rlib", self.crate_name, self.extra_filename) }
    pub fn metadata_name(&self) -> String { format!("lib{}{}.rmeta", self.crate_name, self.extra_filename) }
    pub fn dep_info_name(&self) -> String { format!("{}{}.d", self.crate_name, self.extra_filename) }
    pub fn binary_name(&self) -> String { format!("{}{}", self.crate_name, self.extra_filename) }
    pub fn is_executable(&self) -> bool { self.executable_dependencies.is_some() }
    pub fn target_kind(&self) -> &'static str { if self.is_executable() { "bin" } else { "lib" } }

    pub fn matches_target_name(&self, target: &str) -> bool {
        if self.is_executable() {
            target.bytes().map(|byte| if byte == b'-' { b'_' } else { byte }).eq(self.crate_name.bytes())
        } else {
            target == self.crate_name
        }
    }

    pub fn executable_outputs(&self, output: &Path, check: bool) -> Option<[PathBuf; 2]> {
        let name = if check { self.metadata_name() } else { self.binary_name() };
        let parent = output_parent(output, &name)?;
        Some([parent.join(self.dep_info_name()), output.to_owned()])
    }
    pub fn outputs(&self, archive: &Path) -> Option<[PathBuf;3]> {
        let parent = output_parent(archive, &self.archive_name())?;
        Some([archive.to_owned(), parent.join(self.metadata_name()), parent.join(self.dep_info_name())])
    }

    pub fn check_outputs(&self, metadata: &Path) -> Option<[PathBuf; 2]> {
        let parent = output_parent(metadata, &self.metadata_name())?;
        Some([parent.join(self.dep_info_name()), metadata.to_owned()])
    }

    pub fn valid_output(&self, path: &Path, check: bool, options: LibraryOptions) -> bool {
        let native_debug = options.debug.native_rust();
        if !(options.debug.supported(check) || native_debug) {
            return false;
        }
        if let Some(dependencies) = &self.executable_dependencies {
            options.embed_metadata.is_none() && self.executable_outputs(path, check).is_some()
                && dependencies.iter().all(|dependency| dependency.archive.is_none() == check)
        } else if check {
            options.embed_metadata.is_none() && output_parent(path, &self.metadata_name()).is_some()
        } else {
            output_parent(path, &self.archive_name()).is_some()
        }
    }
}

fn output_parent<'a>(output: &'a Path, name: &str) -> Option<&'a Path> {
    if !output.is_absolute() || output.to_str()?.contains(['\0', '\n', '\r'])
        || output.file_name()?.to_str()? != name
    {
        return None;
    }
    output.parent()
}
