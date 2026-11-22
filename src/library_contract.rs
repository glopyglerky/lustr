//! The bounded public scalar-library request. Paths for delivered artifacts are
//! per-request inputs; they never join the retained semantic context.
use std::path::{Path, PathBuf};

pub const SDK_RECEIPT: &str = "4e400b263bf9923616dea44ead2d77213703177d3bfaf6becd6a19a4c5a657a7";
pub const TARGET: &str = "x86_64-unknown-linux-gnu";
pub const SDK_RECEIPT_PATH: &str = "lib/lustr-metadata/sdk-receipt.json";

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
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ExternMetadata {
    pub alias: String,
    pub artifact: PathBuf,
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
            && self.check_cfg.iter().all(|s| matches!(s.as_str(), "cfg(docsrs,test)" | "cfg(feature, values())"))
            && self.search_paths.iter().all(|p| p.is_absolute())
            && self.external.as_ref().is_none_or(ExternMetadata::valid)
            && self.sdk_receipt == SDK_RECEIPT
    }
    pub fn archive_name(&self) -> String { format!("lib{}{}.rlib", self.crate_name, self.extra_filename) }
    pub fn metadata_name(&self) -> String { format!("lib{}{}.rmeta", self.crate_name, self.extra_filename) }
    pub fn dep_info_name(&self) -> String { format!("{}{}.d", self.crate_name, self.extra_filename) }
    pub fn outputs(&self, archive: &Path) -> Option<[PathBuf;3]> {
        let parent = output_parent(archive, &self.archive_name())?;
        Some([archive.to_owned(), parent.join(self.metadata_name()), parent.join(self.dep_info_name())])
    }

    pub fn check_outputs(&self, metadata: &Path) -> Option<[PathBuf; 2]> {
        let parent = output_parent(metadata, &self.metadata_name())?;
        Some([parent.join(self.dep_info_name()), metadata.to_owned()])
    }

    pub fn valid_output(&self, path: &Path, check: bool, options: LibraryOptions) -> bool {
        if check {
            options.embed_metadata.is_none() && output_parent(path, &self.metadata_name()).is_some()
        } else {
            self.external.is_none() && output_parent(path, &self.archive_name()).is_some()
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
