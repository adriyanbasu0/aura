use std::fs::File;
use std::io::Write;

use super::{Relocation, Symbol};

pub fn write_aura_binary(
    object: &super::AuraObject,
    path: &std::path::Path,
) -> std::io::Result<()> {
    let mut file = File::create(path)?;

    let header = AuraBinaryHeader {
        magic: *b"AURA",
        version: 1,
        flags: 0,
        reserved: 0,
        entry_point: object.entry_point,
        stack_size: 4096,
        text_offset: std::mem::size_of::<AuraBinaryHeader>() as u64,
        text_size: object.text.len() as u64,
        data_offset: (std::mem::size_of::<AuraBinaryHeader>() + align_to(object.text.len(), 16))
            as u64,
        data_size: object.data.len() as u64,
        bss_size: object.bss_size as u64,
        reloc_count: object.relocations.len() as u64,
        symbol_count: object.symbols.len() as u64,
    };

    file.write_all(&header.as_bytes())?;
    file.write_all(&object.text)?;

    let text_pad = align_to(object.text.len(), 16) - object.text.len();
    if text_pad > 0 {
        file.write_all(&vec![0u8; text_pad])?;
    }

    file.write_all(&object.data)?;

    let data_pad = align_to(object.data.len(), 16) - object.data.len();
    if data_pad > 0 {
        file.write_all(&vec![0u8; data_pad])?;
    }

    for reloc in &object.relocations {
        file.write_all(&reloc.as_bytes())?;
    }

    for sym in &object.symbols {
        file.write_all(&sym.as_bytes())?;
    }

    Ok(())
}

fn align_to(size: usize, align: usize) -> usize {
    if align == 0 {
        size
    } else {
        ((size + align - 1) / align) * align
    }
}

#[repr(C)]
struct AuraBinaryHeader {
    magic: [u8; 4],
    version: u8,
    flags: u8,
    reserved: u16,
    entry_point: u64,
    stack_size: u64,
    text_offset: u64,
    text_size: u64,
    data_offset: u64,
    data_size: u64,
    bss_size: u64,
    reloc_count: u64,
    symbol_count: u64,
}

impl AuraBinaryHeader {
    fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.magic);
        bytes.push(self.version);
        bytes.push(self.flags);
        bytes.extend_from_slice(&self.reserved.to_le_bytes());
        bytes.extend_from_slice(&self.entry_point.to_le_bytes());
        bytes.extend_from_slice(&self.stack_size.to_le_bytes());
        bytes.extend_from_slice(&self.text_offset.to_le_bytes());
        bytes.extend_from_slice(&self.text_size.to_le_bytes());
        bytes.extend_from_slice(&self.data_offset.to_le_bytes());
        bytes.extend_from_slice(&self.data_size.to_le_bytes());
        bytes.extend_from_slice(&self.bss_size.to_le_bytes());
        bytes.extend_from_slice(&self.reloc_count.to_le_bytes());
        bytes.extend_from_slice(&self.symbol_count.to_le_bytes());
        bytes
    }
}

impl Relocation {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.offset.to_le_bytes());
        let sym_len = (self.symbol.len() as u64).to_le_bytes();
        bytes.extend_from_slice(&sym_len);
        bytes.extend_from_slice(self.symbol.as_bytes());
        bytes.push(0);
        bytes.push(self.kind.clone() as u8);
        bytes
    }
}

impl Symbol {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let name_len = (self.name.len() as u64).to_le_bytes();
        bytes.extend_from_slice(&name_len);
        bytes.extend_from_slice(self.name.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(&self.offset.to_le_bytes());
        bytes.extend_from_slice(&self.size.to_le_bytes());
        bytes.push(self.kind.clone() as u8);
        bytes
    }
}

pub struct AuraBinary;

impl AuraBinary {
    pub fn dump(data: &[u8]) -> std::io::Result<()> {
        if data.len() < std::mem::size_of::<AuraBinaryHeader>() {
            eprintln!("File too small for header");
            return Ok(());
        }

        let header = AuraBinaryHeader::from_bytes(data);

        println!("=== Aura Binary Dump ===");
        println!(
            "Magic: {}",
            std::str::from_utf8(&header.magic).unwrap_or("INVALID")
        );
        println!("Version: {}", header.version);
        println!("Entry Point: 0x{:016x}", header.entry_point);
        println!("Stack Size: {}", header.stack_size);
        println!(
            "Text Offset: {}, Size: {}",
            header.text_offset, header.text_size
        );
        println!(
            "Data Offset: {}, Size: {}",
            header.data_offset, header.data_size
        );
        println!("BSS Size: {}", header.bss_size);
        println!("Relocations: {}", header.reloc_count);
        println!("Symbols: {}", header.symbol_count);

        let text_start = header.text_offset as usize;
        let text_end = text_start + header.text_size as usize;
        if text_end <= data.len() && header.text_size > 0 {
            println!("\n=== Text Section ({} bytes) ===", header.text_size);
            Self::print_hex(&data[text_start..text_end]);
        }

        let data_start = header.data_offset as usize;
        let data_end = data_start + header.data_size as usize;
        if data_end <= data.len() && header.data_size > 0 {
            println!("\n=== Data Section ({} bytes) ===", header.data_size);
            Self::print_hex(&data[data_start..data_end]);
        }

        Ok(())
    }

    fn print_hex(data: &[u8]) {
        for (i, chunk) in data.chunks(16).enumerate() {
            let offset = i * 16;
            let hex: Vec<String> = chunk.iter().map(|b| format!("{:02x}", b)).collect();
            println!("{:08x}: {:<48}", offset, hex.join(" "));
        }
    }
}

impl AuraBinaryHeader {
    fn from_bytes(data: &[u8]) -> Self {
        let mut pos = 0;
        let mut magic = [0u8; 4];
        magic.copy_from_slice(&data[0..4]);
        pos += 4;

        let version = data[pos];
        pos += 1;

        let flags = data[pos];
        pos += 1;

        let mut reserved = [0u8; 2];
        reserved.copy_from_slice(&data[pos..pos + 2]);
        pos += 2;

        let mut entry_point = [0u8; 8];
        entry_point.copy_from_slice(&data[pos..pos + 8]);
        pos += 8;

        let mut stack_size = [0u8; 8];
        stack_size.copy_from_slice(&data[pos..pos + 8]);
        pos += 8;

        let mut text_offset = [0u8; 8];
        text_offset.copy_from_slice(&data[pos..pos + 8]);
        pos += 8;

        let mut text_size = [0u8; 8];
        text_size.copy_from_slice(&data[pos..pos + 8]);
        pos += 8;

        let mut data_offset = [0u8; 8];
        data_offset.copy_from_slice(&data[pos..pos + 8]);
        pos += 8;

        let mut data_size = [0u8; 8];
        data_size.copy_from_slice(&data[pos..pos + 8]);
        pos += 8;

        let mut bss_size = [0u8; 8];
        bss_size.copy_from_slice(&data[pos..pos + 8]);
        pos += 8;

        let mut reloc_count = [0u8; 8];
        reloc_count.copy_from_slice(&data[pos..pos + 8]);
        pos += 8;

        let mut symbol_count = [0u8; 8];
        symbol_count.copy_from_slice(&data[pos..pos + 8]);

        AuraBinaryHeader {
            magic,
            version,
            flags,
            reserved: u16::from_le_bytes(reserved),
            entry_point: u64::from_le_bytes(entry_point),
            stack_size: u64::from_le_bytes(stack_size),
            text_offset: u64::from_le_bytes(text_offset),
            text_size: u64::from_le_bytes(text_size),
            data_offset: u64::from_le_bytes(data_offset),
            data_size: u64::from_le_bytes(data_size),
            bss_size: u64::from_le_bytes(bss_size),
            reloc_count: u64::from_le_bytes(reloc_count),
            symbol_count: u64::from_le_bytes(symbol_count),
        }
    }
}
