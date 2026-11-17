//! Rename one validated scalar ELF export without a symbol-tool subprocess.
//! Code, relocation records and symbol indexes are unchanged. The new string
//! table is appended, then the complete object is validated before publication.
#![forbid(unsafe_code)]

#[path = "../../lustr-native-core/src/mixed_symbols.rs"]
mod symbols;

use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const LIMIT: usize = 64 * 1024 * 1024;

fn number(bytes: &[u8], at: usize, width: usize) -> Result<usize, String> {
    let end = at.checked_add(width).ok_or("ELF offset overflow")?;
    let value = bytes.get(at..end).ok_or("truncated ELF number")?;
    let mut word = [0u8; 8];
    word.get_mut(..width).ok_or("invalid ELF number width")?.copy_from_slice(value);
    u64::from_le_bytes(word).try_into().map_err(|_| "ELF value exceeds host address space".into())
}

fn replace(bytes: &mut [u8], at: usize, value: &[u8]) -> Result<(), String> {
    let end = at.checked_add(value.len()).ok_or("ELF write offset overflow")?;
    bytes.get_mut(at..end).ok_or("truncated ELF field")?.copy_from_slice(value);
    Ok(())
}

fn renamed(mut bytes: Vec<u8>, expected: &str, replacement: &str) -> Result<Vec<u8>, String> {
    if replacement.is_empty() || replacement.as_bytes().contains(&0) || replacement.len() > LIMIT {
        return Err("invalid replacement symbol".into());
    }
    if expected == replacement {
        symbols::check_renamed_bytes(&bytes, replacement.as_bytes())?;
        return Ok(bytes);
    }
    // read_checked has already proved every range, count and symbol contract.
    // Keep checked reads here as well; no unsafe or unchecked indexing is used.
    let base = number(&bytes, 40, 8)?;
    let count = number(&bytes, 60, 2)?;
    let table = (0..count)
        .map(|index| base + index * 64)
        .find(|at| number(&bytes, at + 4, 4) == Ok(2))
        .ok_or("missing validated ELF symbol table")?;
    let symbols_at = number(&bytes, table + 24, 8)?;
    let symbol_count = number(&bytes, table + 32, 8)? / 24;
    let strings_header = base + number(&bytes, table + 40, 4)? * 64;
    let strings_at = number(&bytes, strings_header + 24, 8)?;
    let strings_size = number(&bytes, strings_header + 32, 8)?;
    let strings_end = strings_at.checked_add(strings_size).ok_or("ELF string table overflow")?;
    let strings = bytes.get(strings_at..strings_end).ok_or("truncated ELF string table")?;
    let mut selected = None;
    for index in 1..symbol_count {
        let at = symbols_at + index * 24;
        let start = number(&bytes, at, 4)?;
        let tail = strings.get(start..).ok_or("invalid ELF symbol name offset")?;
        let end = tail.iter().position(|byte| *byte == 0).ok_or("unterminated ELF symbol name")?;
        let name = &tail[..end];
        if name == replacement.as_bytes() {
            return Err("replacement symbol already exists".into());
        }
        if name == expected.as_bytes() && selected.replace(at).is_some() {
            return Err("duplicate expected symbol".into());
        }
    }
    let selected = selected.ok_or("missing expected symbol")?;
    let new_size = strings_size.checked_add(replacement.len()).and_then(|n| n.checked_add(1)).ok_or("ELF string table size overflow")?;
    let alignment = number(&bytes, strings_header + 48, 8)?.max(1);
    let new_offset = bytes.len().checked_add(alignment - 1).ok_or("renamed ELF alignment overflow")? & !(alignment - 1);
    let total = new_offset.checked_add(new_size).ok_or("renamed ELF size overflow")?;
    if total > LIMIT {
        return Err("renamed ELF exceeds scalar object limit".into());
    }
    let name_offset: u32 = strings_size.try_into().map_err(|_| "ELF symbol name offset exceeds format")?;
    // Preserve all existing strings and their offsets. Moving their table
    // changes no symbol index, relocation, section contents or instruction.
    bytes.resize(new_offset, 0);
    bytes.extend_from_within(strings_at..strings_end);
    bytes.extend_from_slice(replacement.as_bytes());
    bytes.push(0);
    replace(&mut bytes, strings_header + 24, &(new_offset as u64).to_le_bytes())?;
    replace(&mut bytes, strings_header + 32, &(new_size as u64).to_le_bytes())?;
    replace(&mut bytes, selected, &name_offset.to_le_bytes())?;
    symbols::check_renamed_bytes(&bytes, replacement.as_bytes())?;
    Ok(bytes)
}

struct Temporary(PathBuf);
impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

/// Publish complete bytes under a new name without replacing any destination.
/// The sibling temporary has its own inode, never a link to the input object.
/// The caller retains its existing object/directory durability barriers.
fn publish(path: &Path, bytes: &[u8]) -> io::Result<()> {
    static SERIAL: AtomicU64 = AtomicU64::new(0);
    let parent = path.parent().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "object output needs a parent"))?;
    for _ in 0..128 {
        let temporary = parent.join(format!(".lustr-symbol-{}-{}.tmp", std::process::id(), SERIAL.fetch_add(1, Ordering::Relaxed)));
        let mut file = match OpenOptions::new().write(true).create_new(true).open(&temporary) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        };
        let temporary = Temporary(temporary);
        file.write_all(bytes)?;
        drop(file);
        // hard_link publishes a complete file atomically and fails when path
        // already exists, including dangling symlinks. Unlike rename, it can
        // never overwrite an existing compiler output. Drop removes the temp.
        return fs::hard_link(&temporary.0, path);
    }
    Err(io::Error::new(io::ErrorKind::AlreadyExists, "no unused symbol temporary name"))
}

pub(crate) fn rename(input: &Path, expected: &str, replacement: &str, output: &Path) -> Result<(), String> {
    let bytes = symbols::read_checked(input, expected)?;
    let bytes = renamed(bytes, expected, replacement)?;
    publish(output, &bytes).map_err(|error| error.to_string())
}
