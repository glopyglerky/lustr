// Copyright the Rust Project Developers. Licensed under MIT OR Apache-2.0.
// Selected original x86_64 Linux branches of rustc_codegen_ssa::back::metadata
// at 5b7b37aefcb5b2fba3c2392bc9065b9e4602d494. See source-slices.json.
// The caller's completed request fixes ELF, x86_64, little endian, Linux and
// no extra target flags. Other target branches are not silently approximated.
use object::write::{Object, StandardSegment};
use object::{Architecture, BinaryFormat, Endianness, FileFlags, SectionFlags, SectionKind, elf};

pub(crate) fn wrapper(section_name: &[u8], data: &[u8]) -> Result<Vec<u8>, String> {
    let mut file = Object::new(BinaryFormat::Elf, Architecture::X86_64, Endianness::Little);
    // Original add_gnu_property_note, specialized only after target binding.
    let note = file.add_section(file.segment_name(StandardSegment::Data).to_vec(), b".note.gnu.property".to_vec(), SectionKind::Note);
    let mut note_data = Vec::new();
    for value in [4u32, 16, elf::NT_GNU_PROPERTY_TYPE_0] {
        note_data.extend_from_slice(&value.to_le_bytes());
    }
    note_data.extend_from_slice(b"GNU\0");
    for value in [elf::GNU_PROPERTY_X86_FEATURE_1_AND, 4, 3, 0] {
        note_data.extend_from_slice(&value.to_le_bytes());
    }
    file.append_section_data(note, &note_data, 8);
    // elf_os_abi(Linux) and elf_e_flags(X86_64) are both zero upstream.
    file.flags = FileFlags::Elf { os_abi: elf::ELFOSABI_NONE, abi_version: 0, e_flags: 0 };

    let section = file.add_section(file.segment_name(StandardSegment::Debug).to_vec(), section_name.to_vec(), SectionKind::Debug);
    file.section_mut(section).flags = SectionFlags::Elf { sh_flags: elf::SHF_EXCLUDE as u64 };
    file.append_section_data(section, data, 1);
    file.write().map_err(|error| format!("original metadata wrapper: {error}"))
}
