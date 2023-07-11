use std::env;
use std::fs;
use std::io::{Write, Seek, BufWriter};
use std::os::unix::prelude::FileExt;

use object::{Object, ObjectSection};
use object::read::*;

const LE_STUB: &[u8; 563] = include_bytes!("lestub");
const LE_HEADER_OFF: isize = 0x80;

fn print_section_relocations(section: &object::Section, obj_file: &object::File) {
    for (ind, rel) in section.relocations() {
        print!("ind: {:x}\t", ind);
        match rel.kind() {
            object::RelocationKind::Absolute => {
                print!("Absolute\tsize: {}\ttarget: ", rel.size());
                match rel.target() {
                    object::RelocationTarget::Symbol(s) => {
                        let sym = obj_file.symbol_by_index(s).unwrap();
                        let sym_sec_idx = sym.section_index();
                        let sym_sec = if let Some(idx) = sym_sec_idx {
                            Some(obj_file.section_by_index(idx).unwrap())
                        } else {
                            None
                        };
                        let sym_sec_name = if let Some(sec) = &sym_sec {
                            sec.name().unwrap()
                        } else {
                            "None"
                        };
                        let implicit_addend = if rel.has_implicit_addend() {
                            let mut arr: [u8; 4] = [0; 4];
                            arr.copy_from_slice(&section.data().unwrap()[ind as usize..ind as usize+4]);
                            u32::from_le_bytes(arr)
                        } else {
                            0
                        };
                        println!("{} (0x{:04x}) (in {})\taddend: {}\timplicit addend: {}", sym.name().unwrap(), sym.address(), sym_sec_name, rel.addend(), implicit_addend);
                    },
                    object::RelocationTarget::Section(sec) => println!("{}", obj_file.section_by_index(sec).unwrap().name().unwrap()),
                    object::RelocationTarget::Absolute => println!("Absolute"),
                    _ => println!("Err"),
                };
            },
            object::RelocationKind::PltRelative |
            object::RelocationKind::Relative => {
                print!("Relative\tsize: {}\ttarget: ", rel.size());
                match rel.target() {
                    object::RelocationTarget::Symbol(s) => {
                        let sym = obj_file.symbol_by_index(s).unwrap();
                        let sym_sec_idx = sym.section_index();
                        let sym_sec = if let Some(idx) = sym_sec_idx {
                            Some(obj_file.section_by_index(idx).unwrap())
                        } else {
                            None
                        };
                        let sym_sec_name = if let Some(sec) = &sym_sec {
                            sec.name().unwrap()
                        } else {
                            "None"
                        };
                        let implicit_addend = if rel.has_implicit_addend() {
                            let mut arr: [u8; 4] = [0; 4];
                            arr.copy_from_slice(&section.data().unwrap()[ind as usize..ind as usize+4]);
                            u32::from_le_bytes(arr)
                        } else {
                            0
                        };
                        println!("{} (0x{:04x}) (in {})\taddend: {}\timplicit addend: {}", sym.name().unwrap(), sym.address(), sym_sec_name, rel.addend(), implicit_addend);
                    },
                    object::RelocationTarget::Section(sec) => println!("{}", obj_file.section_by_index(sec).unwrap().name().unwrap()),
                    object::RelocationTarget::Absolute => println!("Absolute"),
                    _ => println!("Err"),
                };
            },
            _ => {
                println!("Unsupported Relocation Type");
            },
        }
    }
}

struct LEHeader {
    num_text_pages: u32,
    num_data_pages: u32,
    last_page_bytes: u32,
    fixup_page_offsets: Vec<u32>,
    fixup_records: Vec<u8>
}

fn write_le_header(new_header: &LEHeader, le_stub: &mut std::fs::File) -> std::result::Result<u32, Box<dyn std::error::Error>> {
    let le_header_offset = 0x80;
    let object_table_offset = 0xC4 + le_header_offset;
    let page_table_offset = 0xF4 + le_header_offset;

    // Number of memory pages 14h
    le_stub.write_at(&(new_header.num_text_pages + new_header.num_data_pages).to_le_bytes(), le_header_offset + 0x14)?;
    // Object Table 1 page map index
    le_stub.write_at(&(1 as u32).to_le_bytes(), object_table_offset + 0xC)?;
    // Object Table 1 page map entries
    le_stub.write_at(&new_header.num_text_pages.to_le_bytes(), object_table_offset + 0x10)?;
    // Object Table 2 page map index
    le_stub.write_at(&(new_header.num_text_pages + 1).to_le_bytes(), object_table_offset + 0x18 + 0xC)?;
    // Object Table 2 page map entries
    le_stub.write_at(&new_header.num_data_pages.to_le_bytes(), object_table_offset + 0x18 + 0x10)?;

    // Page Table
    le_stub.seek(std::io::SeekFrom::Start(page_table_offset))?;
    for p_idx in 1..(new_header.num_text_pages + new_header.num_data_pages + 1) {
        le_stub.write(&((p_idx as u32) << 8).to_be_bytes())?;
    }
    // Resident Name Table
    let name_table_offset = le_stub.stream_position()?;
    le_stub.write(b"\x05ELFLE\0\0")?;
    // Entry Table
    le_stub.write(&[0u8, 0u8])?;

    // Fixup page table
    let fixup_page_table_offset = le_stub.stream_position()?;
    for offset in new_header.fixup_page_offsets.iter() {
        le_stub.write(&offset.to_le_bytes())?;
    }
    // Fixup records table
    let fixup_record_table_offset = le_stub.stream_position()?;
    le_stub.write(&new_header.fixup_records)?;

    // Fixup Section length 30h
    le_stub.write_at(
        &(new_header.fixup_page_offsets.len() as u32 * 4 + new_header.fixup_records.len() as u32).to_le_bytes(),
        le_header_offset + 0x30
    )?;
    // Resource table offset, Resource table entries, Resident name table offset 50h 54h 58h
    le_stub.write_at(&(name_table_offset as u32).to_le_bytes(), le_header_offset + 0x50)?;
    le_stub.write_at(&[0u8, 0u8, 0u8, 0u8], le_header_offset + 0x54)?;
    le_stub.write_at(&(name_table_offset as u32).to_le_bytes(), le_header_offset + 0x58)?;
    // Entry table offset 5C
    le_stub.write_at(&(name_table_offset as u32 + 8).to_le_bytes(), le_header_offset + 0x5C)?;
    // Fixup page table offset 68h
    le_stub.write_at(&(fixup_page_table_offset as u32 - le_header_offset as u32).to_le_bytes(), le_header_offset + 0x68)?;
    // Fixup record table offset 6Ch
    le_stub.write_at(&(fixup_record_table_offset as u32 - le_header_offset as u32).to_le_bytes(), le_header_offset + 0x6C)?;
    // Data pages offset 80h
    //let data_pages_offset = 0x1000 +
    //    ((new_header.fixup_records.len() + new_header.fixup_page_offsets.len() * 4 + 0x68 + le_header_offset as usize) / 0x1000) * 0x1000;
    //le_stub.write_at(&(data_pages_offset as u32).to_le_bytes(), le_header_offset + 0x80)?;

    //let data_pages_offset = ((le_stub.stream_position()? >> 12) + 1) << 12;
    let data_pages_offset = le_stub.stream_position()?;
    le_stub.write_at(&(data_pages_offset as u32).to_le_bytes(), le_header_offset + 0x80)?;

    Ok(data_pages_offset as u32)
}

fn output_le_relocations(obj_file: &object::File, le_header: &mut LEHeader, verbose: bool) {
    let mut current_page = 0;
    let mut reloc_idx = 0;
    // start with 0
    le_header.fixup_page_offsets.push(0x00000000);
    let current_section = obj_file.section_by_name(".text").unwrap();
    let mut text_relocations: Vec<(u64, Relocation)> = current_section.relocations().collect();
    text_relocations.sort_by(|(loca, _), (locb, _)| loca.cmp(locb));
    if verbose { println!("\t[LE Text Relocations]"); }
    for (loc, rel) in text_relocations {
        match rel.target() {
            object::RelocationTarget::Symbol(s) => {
                let sym = obj_file.symbol_by_index(s).unwrap();
                let sec_index = obj_file.section_by_index(sym.section_index().unwrap()).unwrap();
                let sec = sec_index.name().unwrap();
                // Type
                le_header.fixup_records.push(match rel.kind() {
                    object::RelocationKind::Absolute => 0x07,
                    object::RelocationKind::Relative => 0x08,
                    object::RelocationKind::PltRelative => 0x08,
                    _ => panic!()
                });
                // Flags
                let dwordoffset = sym.address() >= 0x10000;
                if dwordoffset {
                    // 32-bit Target Offset Flag
                    le_header.fixup_records.push(0x10);
                } else {
                    le_header.fixup_records.push(0);
                }
                let page = loc / 0x1000;
                let src_in_page = loc % 0x1000;
                // Source Offset in Page
                le_header.fixup_records.extend_from_slice(&(src_in_page as u16).to_le_bytes());
                // Target Object
                let target_sec = match sec {
                    ".text" => 1,
                    ".data" => 2,
                    _ => panic!()
                };
                le_header.fixup_records.push(target_sec);
                // Target Offset
                let target_offset = sym.address() as u32 +
                    if rel.has_implicit_addend() & (rel.kind() == object::RelocationKind::Absolute) {
                        let mut arr: [u8; 4] = [0; 4];
                        arr.copy_from_slice(&current_section.data().unwrap()[loc as usize..loc as usize+4]);
                        u32::from_le_bytes(arr)
                    } else { 0 };
                if dwordoffset {
                    // 32-bit offset
                    le_header.fixup_records.extend_from_slice(&(target_offset as u32).to_le_bytes());
                } else {
                    // 16-bit offset
                    le_header.fixup_records.extend_from_slice(&(target_offset as u16).to_le_bytes());
                }


                while page > current_page {
                    // new page
                    le_header.fixup_page_offsets.push(reloc_idx);
                    current_page += 1;

                    if verbose { println!("\nFixup page rollover at {:05x} for page {}", reloc_idx, current_page); }
                }
                if verbose { print!("{}:0x{:05x}->{}:0x{:05x} ", current_page, loc, target_sec, target_offset); }
                if dwordoffset {
                    // 32-bit offset
                    reloc_idx += 2;
                }
            }
            _ => eprintln!("Unsupported Relocation")
        }
        reloc_idx += 7;
    }
    while current_page < le_header.num_text_pages as u64 {
        le_header.fixup_page_offsets.push(reloc_idx);
        current_page += 1;
    }
    if verbose { println!(); }
    current_page = 0;
    let current_section = obj_file.section_by_name(".data").unwrap();
    let mut data_relocations: Vec<(u64, Relocation)> = current_section.relocations().collect();
    data_relocations.sort_by(|(loca, _), (locb, _)| loca.cmp(locb));
    if verbose { println!("\t[LE Data Relocations]"); }
    for (loc, rel) in data_relocations {
        match rel.target() {
            object::RelocationTarget::Symbol(s) => {
                let sym = obj_file.symbol_by_index(s).unwrap();
                let sec_index = obj_file.section_by_index(sym.section_index().unwrap()).unwrap();
                let sec = sec_index.name().unwrap();
                // Type
                le_header.fixup_records.push(match rel.kind() {
                    object::RelocationKind::Absolute => 0x07,
                    object::RelocationKind::Relative => 0x08,
                    object::RelocationKind::PltRelative => 0x08,
                    _ => panic!()
                });
                // Flags
                let dwordoffset = sym.address() >= 0x10000;
                if dwordoffset {
                    // 32-bit Target Offset Flag
                    le_header.fixup_records.push(0x10);
                } else {
                    le_header.fixup_records.push(0);
                }
                let page = loc / 0x1000;
                let src_in_page = loc % 0x1000;
                // Source Offset in Page
                le_header.fixup_records.extend_from_slice(&(src_in_page as u16).to_le_bytes());
                // Target Object
                let target_sec = match sec {
                    ".text" => 1,
                    ".data" => 2,
                    _ => panic!()
                };
                le_header.fixup_records.push(target_sec);
                // Target Offset
                let target_offset = sym.address() as u32 +
                    if rel.has_implicit_addend() & (rel.kind() == object::RelocationKind::Absolute) {
                        let mut arr: [u8; 4] = [0; 4];
                        arr.copy_from_slice(&current_section.data().unwrap()[loc as usize..loc as usize+4]);
                        u32::from_le_bytes(arr)
                    } else { 0 };
                if dwordoffset {
                    // 32-bit offset
                    le_header.fixup_records.extend_from_slice(&(target_offset as u32).to_le_bytes());
                } else {
                    // 16-bit offset
                    le_header.fixup_records.extend_from_slice(&(target_offset as u16).to_le_bytes());
                }


                while page > current_page {
                    // new page
                    le_header.fixup_page_offsets.push(reloc_idx);
                    current_page += 1;
                    if verbose { println!("\nFixup page rollover at {:05x} for page {}", reloc_idx, current_page); }
                }
                if verbose { print!("{}:0x{:05x}->{}:0x{:05x} ", current_page, loc, target_sec, target_offset); }
                if dwordoffset {
                    // 32-bit offset
                    reloc_idx += 2;
                }
            }
            _ => eprintln!("Unsupported Relocation")
        }
        reloc_idx += 7;
    }
    while current_page < le_header.num_data_pages as u64 {
        le_header.fixup_page_offsets.push(reloc_idx);
        current_page += 1;
    }
    if verbose { println!(); }
    // End of Fixup page table
    le_header.fixup_page_offsets.push(reloc_idx);

    println!("{} bytes of relocations", reloc_idx);
}

fn convert(data: &[u8], verbose: bool) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let obj_file = object::File::parse(data)?;

    if verbose {
        for section in obj_file.sections() {
            println!("\tSECTION [{}]\tKIND {}", section.name().unwrap(), match section.kind() {
                object::SectionKind::Text => "text",
                object::SectionKind::Data => "data",
                object::SectionKind::ReadOnlyData => "rodata",
                object::SectionKind::UninitializedData => "bss",
                _ => "Other",
            });
            println!("\tRELOCATIONS FOR [{}]", section.name().unwrap());
            print_section_relocations(&section, &obj_file);
            println!();
        }
    }
    for symbol in obj_file.symbols() {
        let sym_sec_idx = symbol.section_index();
        let sym_sec = if let Some(idx) = sym_sec_idx {
            Some(obj_file.section_by_index(idx)?)
        } else {
            None
        };
        let sym_sec_name = if let Some(sec) = &sym_sec {
            sec.name().unwrap()
        } else {
            "None"
        };
        if verbose { println!("SYMBOL [{}]\tKIND {}\tSECTION {}", symbol.name().unwrap(), match symbol.kind() {
            object::SymbolKind::Text => "Func",
            object::SymbolKind::Data => "Data",
            object::SymbolKind::Section => "Section",
            object::SymbolKind::Label => "Label",
            _ => "Other",
        }, sym_sec_name); } 
    }

    let mut new_elf = object::write::Object::new(object::BinaryFormat::Elf, object::Architecture::I386, object::Endianness::Little);
    let text_sec_name = Vec::from(".text");
    let null_seg = Vec::new();
    let new_text_sec = new_elf.add_section(null_seg, text_sec_name, object::SectionKind::Text);
    let data_sec_name = Vec::from(".data");
    let null_seg = Vec::new();
    let new_data_sec = new_elf.add_section(null_seg, data_sec_name, object::SectionKind::Data);

    let mut new_sym_map = std::collections::HashMap::new();
    // get start section first
    if let Some(start_section) = obj_file.section_by_name(".text.start") {
        let new_symbol = new_elf.add_symbol(object::write::Symbol {
            name: Vec::from(".text.start"),
            value: 0x0,
            size: 0x0,
            kind: object::SymbolKind::Text,
            scope: object::SymbolScope::Compilation,
            weak: false,
            section: object::write::SymbolSection::Section(new_text_sec),
            flags: object::SymbolFlags::None,
        });
        new_elf.add_symbol_data(new_symbol, new_text_sec, start_section.data().unwrap(), 1);
        new_sym_map.insert(String::from(".text.start"), new_symbol);
    }
    for section in obj_file.sections() {
        if section.name().unwrap() == ".text.start" { continue }
        match section.kind() {
            object::SectionKind::Text => {
                let new_symbol = new_elf.add_symbol(object::write::Symbol {
                    name: Vec::from(section.name().unwrap()),
                    value: 0x0,
                    size: 0x0,
                    kind: object::SymbolKind::Text,
                    scope: object::SymbolScope::Compilation,
                    weak: false,
                    section: object::write::SymbolSection::Section(new_text_sec),
                    flags: object::SymbolFlags::None,
                });
                new_elf.add_symbol_data(new_symbol, new_text_sec, section.data().unwrap(), 1);
                new_sym_map.insert(String::from(section.name().unwrap()), new_symbol);
            },
            object::SectionKind::Data | object::SectionKind::ReadOnlyData => {
                let new_symbol = new_elf.add_symbol(object::write::Symbol {
                    name: Vec::from(section.name().unwrap()),
                    value: 0x0,
                    size: 0x0,
                    kind: object::SymbolKind::Data,
                    scope: object::SymbolScope::Compilation,
                    weak: false,
                    section: object::write::SymbolSection::Section(new_data_sec),
                    flags: object::SymbolFlags::None,
                });
                new_elf.add_symbol_data(new_symbol, new_data_sec, section.data().unwrap(), 1);
                new_sym_map.insert(String::from(section.name().unwrap()), new_symbol);
            },
            object::SectionKind::UninitializedData => {
                let new_symbol = new_elf.add_symbol(object::write::Symbol {
                    name: Vec::from(section.name().unwrap()),
                    value: 0x0,
                    size: 0x0,
                    kind: object::SymbolKind::Data,
                    scope: object::SymbolScope::Compilation,
                    weak: false,
                    section: object::write::SymbolSection::Section(new_data_sec),
                    flags: object::SymbolFlags::None,
                });
                let bss_zeros = vec![0; section.size() as usize];
                new_elf.add_symbol_data(new_symbol, new_data_sec, &bss_zeros, 1);
                new_sym_map.insert(String::from(section.name().unwrap()), new_symbol);
            }
            _ => {}
        }
    }
    for symbol in obj_file.symbols() {
        let sym_sec_idx = symbol.section_index();
        let sym_sec = if let Some(idx) = sym_sec_idx {
            Some(obj_file.section_by_index(idx)?)
        } else {
            None
        };
        let new_offset = if let Some(sec) = &sym_sec {
            let old_section_name = sec.name().unwrap();
            match new_sym_map.get(old_section_name) {
                Some(id) => new_elf.symbol(*id).value + symbol.address(),
                None => symbol.address()
            }
        } else {
            symbol.address()
        };
        match symbol.kind() {
            object::SymbolKind::Text => {
                let new_symbol = new_elf.add_symbol(object::write::Symbol {
                    name: Vec::from(symbol.name().unwrap()),
                    value: new_offset,
                    size: symbol.size(),
                    kind: object::SymbolKind::Text,
                    scope: object::SymbolScope::Compilation,
                    weak: false,
                    section: object::write::SymbolSection::Section(new_text_sec),
                    flags: object::SymbolFlags::None,
                });
                new_sym_map.insert(String::from(symbol.name().unwrap()), new_symbol);
            }
            object::SymbolKind::Data => {
                let new_symbol = new_elf.add_symbol(object::write::Symbol {
                    name: Vec::from(symbol.name().unwrap()),
                    value: new_offset,
                    size: symbol.size(),
                    kind: object::SymbolKind::Data,
                    scope: object::SymbolScope::Compilation,
                    weak: false,
                    section: object::write::SymbolSection::Section(new_data_sec),
                    flags: object::SymbolFlags::None,
                });
                new_sym_map.insert(String::from(symbol.name().unwrap()), new_symbol);
            }
            _ => {}
        }
    }

    for section in obj_file.sections() {
        if section.kind() != object::SectionKind::Text &&
           section.kind() != object::SectionKind::Data &&
           section.kind() != object::SectionKind::ReadOnlyData &&
           section.kind() != object::SectionKind::UninitializedData {
                continue
        }
        let new_src_sym_id = new_sym_map.get(section.name().unwrap()).unwrap();
        let base_addr = new_elf.symbol(*new_src_sym_id).value;
        for (src, reloc) in section.relocations() {
            match reloc.target() {
                RelocationTarget::Symbol(sym_idx) => {
                    let old_sym = obj_file.symbol_by_index(sym_idx).unwrap();
                    // ???????
                    if old_sym.section_index().is_none() { continue }
                    let old_sec = obj_file.section_by_index(old_sym.section_index().unwrap()).unwrap();
                    let new_sym = if let Some(new_sym) = new_sym_map.get(old_sym.name().unwrap()) {
                        Some(new_sym)
                    } else {
                        if let Some(new_sym) = new_sym_map.get(old_sec.name().unwrap()) {
                            Some(new_sym)
                        } else {
                            None
                        }
                    };
                    if let Some(new) = new_sym {
                        let new_src_sec_id = new_elf.symbol(*new_src_sym_id).section.id().unwrap();
                        new_elf.add_relocation(new_src_sec_id, object::write::Relocation {
                            offset: base_addr + src,
                            size: reloc.size(),
                            kind: reloc.kind(),
                            encoding: reloc.encoding(),
                            symbol: *new,
                            addend: reloc.addend(),
                        })?;
                        if verbose { println!("reloc {:04x} in {} -> {} in {} {:04x}@{:04x} Became {:04x} in {} -> {} in {} ({:04x})",
                            src,
                            section.name().unwrap(),
                            old_sym.name().unwrap(),
                            old_sec.name().unwrap(),
                            old_sym.address(),
                            old_sec.address(),
                            base_addr + src,
                            new_elf.section(new_src_sec_id).name().unwrap(),
                            new_elf.symbol(*new).name().unwrap(),
                            new_elf.section(new_elf.symbol(*new).section.id().unwrap()).name().unwrap(),
                            new_elf.symbol(*new).value
                        ); }
                    } else {
                        if verbose { eprintln!("Warning: Couldn't find new equivalent of {:04x} -> symbol {} in {} ({:04x}@{:04x})", src,
                            old_sym.name().unwrap(),
                            old_sec.name().unwrap(),
                            old_sym.address(),
                            old_sec.address()
                        ); }
                    }
                },
                _ => { panic!("Unsupported Relocation Type"); },
            }
        }
    }

    {
        let mut out_elf = fs::File::create("new.elf")?;
        new_elf.write_stream(&mut out_elf)?;
    }

    if verbose { print!("\n\n\t"); }
    println!("Wrote [new.elf]! Trying to produce LE executable.");

    let new_file = fs::read("new.elf")?;
    let new_obj = object::File::parse(&*new_file)?;

    if verbose { println!(); }
    let text_len = new_obj.section_by_name(".text").unwrap().data().unwrap().len();
    let data_len = new_obj.section_by_name(".data").unwrap().data().unwrap().len();
    let text_pages = if text_len % 0x1000 == 0 {
        text_len / 0x1000
    } else {
        (text_len / 0x1000) + 1
    };
    let data_pages = if data_len % 0x1000 == 0 {
        data_len / 0x1000
    } else {
        (data_len / 0x1000) + 1
    };
    println!("[.text] size: 0x{:08x} ({} pages)", text_len, text_pages);
    println!("[.data] size: 0x{:08x} ({} pages)", data_len, data_pages);

    if verbose {
        let text_sec = new_obj.section_by_name(".text").unwrap();
        println!("\n\tRELOCATIONS FOR [.text]");
        print_section_relocations(&text_sec, &new_obj);
        println!("\n\tRELOCATIONS FOR [.data]");
        let data_sec = new_obj.section_by_name(".data").unwrap();
        print_section_relocations(&data_sec, &new_obj);
    }

    let le_stub: Vec<u8> = Vec::from(*LE_STUB);
    let mut out_file = fs::File::create("a.exe")?;
    out_file.write(&le_stub)?;
    out_file.set_len(0x2000)?;
    let mut header = LEHeader {
        num_text_pages: text_pages as u32,
        num_data_pages: data_pages as u32,
        last_page_bytes: 0x1000,
        fixup_page_offsets: Vec::new(),
        fixup_records: Vec::new(),
    };
    output_le_relocations(&new_obj, &mut header, verbose);
    let data_pages_offset = write_le_header(&header, &mut out_file)?;
    println!("Data Pages Offset: 0x{:04x}", data_pages_offset);
    out_file.write_all_at(new_obj.section_by_name(".text").unwrap().data().unwrap(), data_pages_offset as u64)?;
    let data_loc = (header.num_text_pages * 0x1000) + data_pages_offset;
    out_file.write_all_at(new_obj.section_by_name(".data").unwrap().data().unwrap(), data_loc as u64)?;

    println!("Wrote a.exe, {} bytes.", data_loc as usize + new_obj.section_by_name(".data").unwrap().data().unwrap().len());

    Ok(())
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 { eprintln!("Not enough args"); }

    let verbose = args[1] == "-v";

    if verbose && args.len() < 3 { eprintln!("Not enough args"); }

    let path = if verbose { &args[2] } else { &args[1] };
    let data = fs::read(path)?;
    convert(&data, verbose)?;

    Ok(())
}
