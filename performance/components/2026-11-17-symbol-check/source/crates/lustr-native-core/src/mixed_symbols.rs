//! Symbol closure for the mixed scalar emitter's Linux x86_64 TinyCC objects.
//! This deliberately accepts a bounded relocatable format, not arbitrary ELF.
//! No loader, compiler metadata, symbol tool, or unchecked byte access is used.
use std::{fs, io::Read, path::Path};

const LIMIT: usize = 64 * 1024 * 1024;

fn require(condition: bool, message: &str) -> Result<(), String> {
    if condition { Ok(()) } else { Err(message.into()) }
}

fn part(bytes: &[u8], offset: usize, size: usize) -> Result<&[u8], String> {
    bytes.get(offset..offset.checked_add(size).ok_or("ELF range overflow")?).ok_or_else(|| "truncated ELF object".into())
}

fn number(bytes: &[u8], offset: usize, size: usize) -> Result<usize, String> {
    let mut word = [0u8; 8];
    word[..size].copy_from_slice(part(bytes, offset, size)?);
    u64::from_le_bytes(word).try_into().map_err(|_| "ELF value exceeds host address space".into())
}

fn name(strings: &[u8], offset: usize) -> Result<&[u8], String> {
    let tail = strings.get(offset..).ok_or("invalid ELF string offset")?;
    let end = tail.iter().take(4097).position(|b| *b == 0).ok_or("unterminated or oversized ELF string")?;
    Ok(&tail[..end])
}

#[derive(Clone, Copy)]
struct Section {
    kind: usize,
    flags: usize,
    offset: usize,
    size: usize,
    link: usize,
    info: usize,
    entry: usize,
}

/// Inspect the actual object before its existing rename and durability steps.
/// A growing or substituted input cannot turn the size limit into an unbounded read.
pub(super) fn check(path: &Path, expected: &str) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    require(metadata.is_file() && metadata.len() <= LIMIT as u64, "expected a bounded regular native object")?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    fs::File::open(path).map_err(|e| e.to_string())?.take(LIMIT as u64 + 1).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    check_bytes(&bytes, expected.as_bytes())
}

fn check_bytes(bytes: &[u8], expected: &[u8]) -> Result<(), String> {
    require(
        (64..=LIMIT).contains(&bytes.len()) && !expected.is_empty() && expected.len() <= 256 && !expected.contains(&0),
        "invalid native object or expected symbol size",
    )?;
    require(
        part(bytes, 0, 7)? == b"\x7fELF\x02\x01\x01"
            && matches!(bytes[7], 0 | 3)
            && part(bytes, 8, 8)? == [0; 8]
            && number(bytes, 16, 2)? == 1
            && number(bytes, 18, 2)? == 62
            && number(bytes, 20, 4)? == 1
            && number(bytes, 24, 8)? == 0
            && number(bytes, 32, 8)? == 0
            && number(bytes, 48, 4)? == 0
            && number(bytes, 52, 2)? == 64
            && number(bytes, 54, 2)? == 0
            && number(bytes, 56, 2)? == 0
            && number(bytes, 58, 2)? == 64,
        "expected Linux x86_64 little-endian ELF64 relocatable object",
    )?;
    let base = number(bytes, 40, 8)?;
    let count = number(bytes, 60, 2)?;
    let names_index = number(bytes, 62, 2)?;
    require(
        base >= 64 && base % 8 == 0 && (2..=4096).contains(&count) && (1..count).contains(&names_index),
        "unsupported ELF section table",
    )?;
    part(bytes, base, count * 64)?;
    require(part(bytes, base, 64)? == [0; 64], "invalid ELF null section")?;
    let mut sections = Vec::with_capacity(count);
    let mut ranges = vec![(0, 64), (base, base + count * 64)];
    let mut symbols_index = None;
    for index in 0..count {
        let at = base + index * 64;
        let section = Section {
            kind: number(bytes, at + 4, 4)?,
            flags: number(bytes, at + 8, 8)?,
            offset: number(bytes, at + 24, 8)?,
            size: number(bytes, at + 32, 8)?,
            link: number(bytes, at + 40, 4)?,
            info: number(bytes, at + 44, 4)?,
            entry: number(bytes, at + 56, 8)?,
        };
        let alignment = number(bytes, at + 48, 8)?;
        require(index == 0 || matches!(section.kind, 1 | 2 | 3 | 4 | 8), "unsupported ELF section type")?;
        require(
            number(bytes, at + 16, 8)? == 0
                && section.flags & !0x37 == 0
                && (alignment == 0 || alignment.is_power_of_two() && alignment <= 4096),
            "unsupported ELF section attributes",
        )?;
        require(section.offset <= bytes.len() && section.size <= LIMIT, "ELF section exceeds object bounds")?;
        if section.kind != 8 && section.size != 0 {
            part(bytes, section.offset, section.size)?;
            ranges.push((section.offset, section.offset + section.size));
        }
        if section.kind == 2 {
            require(symbols_index.replace(index).is_none(), "expected exactly one ELF symbol table")?;
        }
        sections.push(section);
    }
    ranges.sort_unstable();
    require(ranges.windows(2).all(|pair| pair[0].1 <= pair[1].0), "overlapping ELF tables or sections")?;
    let section_names = sections[names_index];
    require(section_names.kind == 3, "ELF section names need a string table")?;
    let section_names = part(bytes, section_names.offset, section_names.size)?;
    require(section_names.first() == Some(&0) && section_names.last() == Some(&0), "invalid ELF section-name table")?;
    for index in 0..count {
        name(section_names, number(bytes, base + index * 64, 4)?)?;
    }
    let symbols_index = symbols_index.ok_or("missing ELF symbol table")?;
    let table = sections[symbols_index];
    require(table.entry == 24 && table.size >= 48 && table.size % 24 == 0 && (1..count).contains(&table.link), "invalid ELF symbol table")?;
    let symbol_count = table.size / 24;
    require(symbol_count <= 65536, "ELF symbol count exceeds scalar object limit")?;
    require((1..=symbol_count).contains(&table.info), "invalid ELF local-symbol boundary")?;
    let strings = sections[table.link];
    require(strings.kind == 3, "ELF symbols need a string table")?;
    let strings = part(bytes, strings.offset, strings.size)?;
    require(strings.first() == Some(&0) && strings.last() == Some(&0), "invalid ELF symbol string table")?;
    require(part(bytes, table.offset, 24)? == [0; 24], "invalid ELF null symbol")?;
    let mut found = false;
    let mut relocatable = vec![false; symbol_count];
    for index in 1..symbol_count {
        let at = table.offset + index * 24;
        let symbol_name = name(strings, number(bytes, at, 4)?)?;
        let info = bytes[at + 4];
        let binding = info >> 4;
        let kind = info & 15;
        let visibility = bytes[at + 5];
        let section = number(bytes, at + 6, 2)?;
        let value = number(bytes, at + 8, 8)?;
        let size = number(bytes, at + 16, 8)?;
        // Inspect undefined entries before the binding/type tests. In particular,
        // a malformed local or zero-name import must never disappear from closure.
        if section == 0 {
            return Err(format!("native object requires unrepresented symbols: {}", String::from_utf8_lossy(symbol_name)));
        }
        require(
            matches!(binding, 0..=2) && (binding == 0) == (index < table.info) && visibility <= 3,
            "unsupported ELF symbol binding or visibility",
        )?;
        require(!symbol_name.is_empty() || kind == 3 && binding == 0, "unnamed ELF symbol")?;
        if kind == 4 {
            require(binding == 0 && section == 0xfff1 && value == 0 && size == 0, "invalid ELF file symbol")?;
            continue;
        }
        // Common and absolute non-file definitions, TLS, IFUNC and extended
        // section indexes are outside this scalar object contract.
        require(matches!(kind, 0..=3) && (1..count).contains(&section), "unsupported ELF symbol type or section")?;
        let target = sections[section];
        require(matches!(target.kind, 1 | 8) && value <= target.size && size <= target.size - value, "ELF symbol is outside its section")?;
        if kind == 2 {
            require(target.kind == 1 && target.flags & 4 != 0 && size != 0, "ELF function lacks executable bytes")?;
        }
        if kind == 3 {
            require(binding == 0 && value == 0 && size == 0, "invalid ELF section symbol")?;
        }
        relocatable[index] = true;
        if symbol_name == expected {
            require(!found && binding == 1 && kind == 2 && visibility == 0, "invalid or duplicate native export")?;
            found = true;
        }
    }
    require(found, "native object is missing its expected function export")?;
    for relocation in sections.iter().filter(|section| section.kind == 4) {
        require(
            relocation.link == symbols_index
                && (1..count).contains(&relocation.info)
                && relocation.entry == 24
                && relocation.size % 24 == 0,
            "invalid ELF relocation table",
        )?;
        let target = sections[relocation.info];
        require(target.kind == 1, "invalid ELF relocation target")?;
        for at in (relocation.offset..relocation.offset + relocation.size).step_by(24) {
            let offset = number(bytes, at, 8)?;
            let kind = number(bytes, at + 8, 4)?;
            let symbol = number(bytes, at + 12, 4)?;
            let width = match kind {
                1 => 8,
                2 | 4 | 10 | 11 => 4,
                _ => return Err("unsupported ELF relocation kind".into()),
            };
            require(
                (1..symbol_count).contains(&symbol) && relocatable[symbol] && offset <= target.size && width <= target.size - offset,
                "ELF relocation is outside its symbol table or target",
            )?;
        }
    }
    Ok(())
}
