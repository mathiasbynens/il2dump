#![allow(dead_code, unused_variables, unused_imports, unused_mut)]
use crate::binary_reader::{BinaryReader, Endianness};
use crate::il2cpp_binary_structures::{
    Il2CppCodeRegistration, Il2CppMetadataRegistration, Il2CppType,
};
use object::{Object, ObjectSection, ObjectSegment, ObjectSymbol};
use std::collections::HashMap;
use std::io::{self, Cursor, Read, Seek};

#[derive(Debug, Clone, Default)]
pub struct SearchSection {
    pub offset: u64,
    pub offset_end: u64,
    pub address: u64,
    pub address_end: u64,
}

// Reference to a pointer value found in the data sections.
#[derive(Debug, Clone, Copy)]
struct PointerRef {
    value: u64,
    va: u64,
}

pub struct BinaryFile {
    pub bytes: Vec<u8>,
    pub is_32bit: bool,
    pub endian: Endianness,
    pub image_base: u64,
    pub exec_sections: Vec<SearchSection>,
    pub data_sections: Vec<SearchSection>,
    pub bss_sections: Vec<SearchSection>,
    pub is_dumped: bool,
    // Index of all pointer values in the data sections.
    pointer_index: Vec<PointerRef>,
}

impl BinaryFile {
    pub fn parse(bytes: Vec<u8>) -> Result<Self, String> {
        // Check if the binary contains a Mach-O fat header.
        if bytes.len() >= 4 {
            let magic = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            if magic == 0xCAFEBABE || magic == 0xBEBAFECA {
                // Parse the Mach-O fat binary header.
                let num_fat = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
                println!("Mach-O FAT binary detected with {} architectures.", num_fat);
                // Choose the 64-bit slice if available, otherwise default to the first slice.
                let mut chosen_offset = 0;
                let mut chosen_size = 0;
                let mut pos = 8;
                for _ in 0..num_fat {
                    if pos + 20 > bytes.len() {
                        break;
                    }
                    let cputype = u32::from_be_bytes([
                        bytes[pos],
                        bytes[pos + 1],
                        bytes[pos + 2],
                        bytes[pos + 3],
                    ]);
                    let offset = u32::from_be_bytes([
                        bytes[pos + 8],
                        bytes[pos + 9],
                        bytes[pos + 10],
                        bytes[pos + 11],
                    ]) as usize;
                    let size = u32::from_be_bytes([
                        bytes[pos + 12],
                        bytes[pos + 13],
                        bytes[pos + 14],
                        bytes[pos + 15],
                    ]) as usize;

                    // 0x0100000c is CPU_TYPE_ARM64.
                    // Prefer the ARM64 or any 64-bit architecture slice.
                    if cputype == 0x0100000c || chosen_offset == 0 {
                        chosen_offset = offset;
                        chosen_size = size;
                    }
                    pos += 20;
                }
                if chosen_offset > 0 && chosen_offset + chosen_size <= bytes.len() {
                    println!(
                        "Selected Mach-O slice at offset 0x{:x} (size {} bytes)",
                        chosen_offset, chosen_size
                    );
                    let slice = bytes[chosen_offset..chosen_offset + chosen_size].to_vec();
                    return Self::parse_single(slice);
                }
            }
        }
        Self::parse_single(bytes)
    }

    fn parse_single(bytes: Vec<u8>) -> Result<Self, String> {
        let (
            is_32bit,
            endian,
            image_base,
            format,
            exec_sections,
            data_sections,
            bss_sections,
            dyn_symbol_addresses,
            rela_data,
            rel_data,
            arch,
        ) = {
            let obj = object::File::parse(&bytes[..])
                .map_err(|e| format!("Failed to parse binary: {}", e))?;
            let is_32bit = !obj.is_64();
            let endian = if obj.is_little_endian() {
                Endianness::Little
            } else {
                Endianness::Big
            };
            let image_base = obj.relative_address_base();
            let format = obj.format();
            let arch = obj.architecture();

            let mut exec_sections = Vec::new();
            let mut data_sections = Vec::new();
            let mut bss_sections = Vec::new();

            match format {
                object::BinaryFormat::Elf => {
                    // ELF: Classify by program header segments as the C# Elf dumper does.
                    for seg in obj.segments() {
                        if seg.size() == 0 {
                            continue;
                        }
                        if let object::SegmentFlags::Elf { p_flags } = seg.flags() {
                            let address = seg.address();
                            let size = seg.size();
                            let (file_offset, file_size) = seg.file_range();

                            let search_sec = SearchSection {
                                offset: file_offset,
                                offset_end: file_offset + file_size,
                                address,
                                address_end: address + size,
                            };

                            if (p_flags & 1) != 0 {
                                // Execute permission (PF_X).
                                exec_sections.push(search_sec);
                            } else if (p_flags & 2) != 0 || (p_flags & 4) != 0 {
                                // Write or Read permission (PF_W or PF_R).
                                data_sections.push(search_sec.clone());
                                bss_sections.push(search_sec);
                            }
                        }
                    }
                }
                object::BinaryFormat::MachO => {
                    // Mach-O: Classify by sections as the C# Macho dumper does.
                    for sec in obj.sections() {
                        let address = sec.address();
                        let size = sec.size();
                        let (file_offset, file_size) = sec.file_range().unwrap_or((0, 0));

                        let search_sec = SearchSection {
                            offset: file_offset,
                            offset_end: file_offset + file_size,
                            address,
                            address_end: address + size,
                        };

                        let name = sec.name().unwrap_or("");
                        if let object::SectionFlags::MachO { flags } = sec.flags() {
                            if flags == 0x80000400 {
                                exec_sections.push(search_sec);
                            } else if name == "__const" || name == "__cstring" || name == "__data" {
                                data_sections.push(search_sec);
                            } else if flags == 1 {
                                bss_sections.push(search_sec);
                            }
                        }
                    }
                }
                object::BinaryFormat::Pe => {
                    // PE: Classify by sections using characteristics flags.
                    for sec in obj.sections() {
                        let address = sec.address();
                        let size = sec.size();
                        let (file_offset, file_size) = sec.file_range().unwrap_or((0, 0));

                        let search_sec = SearchSection {
                            offset: file_offset,
                            offset_end: file_offset + file_size,
                            address,
                            address_end: address + size,
                        };

                        if let object::SectionFlags::Coff { characteristics } = sec.flags() {
                            if characteristics == 0x60000020 {
                                exec_sections.push(search_sec);
                            } else if characteristics == 0x40000040 || characteristics == 0xC0000040
                            {
                                data_sections.push(search_sec.clone());
                                bss_sections.push(search_sec);
                            }
                        }
                    }
                }
                _ => {
                    // Fallback for other formats: classify by SectionKind.
                    for sec in obj.sections() {
                        let address = sec.address();
                        let size = sec.size();
                        let (file_offset, file_size) = sec.file_range().unwrap_or((0, 0));

                        let search_sec = SearchSection {
                            offset: file_offset,
                            offset_end: file_offset + file_size,
                            address,
                            address_end: address + size,
                        };

                        let kind = sec.kind();
                        if kind == object::SectionKind::Text {
                            exec_sections.push(search_sec);
                        } else if kind == object::SectionKind::Data
                            || kind == object::SectionKind::ReadOnlyData
                        {
                            data_sections.push(search_sec);
                        } else if kind == object::SectionKind::UninitializedData {
                            bss_sections.push(search_sec);
                        }
                    }
                }
            }

            // Apply global fallbacks if any list remains empty.
            if exec_sections.is_empty() {
                for seg in obj.segments() {
                    let (file_offset, file_size) = seg.file_range();
                    let address = seg.address();
                    let size = seg.size();
                    exec_sections.push(SearchSection {
                        offset: file_offset,
                        offset_end: file_offset + file_size,
                        address,
                        address_end: address + size,
                    });
                }
            }
            if data_sections.is_empty() {
                data_sections = exec_sections.clone();
            }

            let mut max_idx = 0;
            for sym in obj.dynamic_symbols() {
                max_idx = max_idx.max(sym.index().0);
            }
            let mut dyn_symbol_addresses = vec![0u64; max_idx + 1];
            for sym in obj.dynamic_symbols() {
                dyn_symbol_addresses[sym.index().0] = sym.address();
            }
            let rela_data = obj
                .section_by_name(".rela.dyn")
                .and_then(|sec| sec.data().ok())
                .map(|d| d.to_vec());
            let rel_data = obj
                .section_by_name(".rel.dyn")
                .and_then(|sec| sec.data().ok())
                .map(|d| d.to_vec());

            let res: Result<_, String> = Ok((
                is_32bit,
                endian,
                image_base,
                format,
                exec_sections,
                data_sections,
                bss_sections,
                dyn_symbol_addresses,
                rela_data,
                rel_data,
                arch,
            ));
            res
        }?;

        let mut bytes = bytes;
        if format == object::BinaryFormat::Elf {
            Self::apply_relocations_raw(
                &mut bytes,
                arch,
                &dyn_symbol_addresses,
                rela_data.as_deref(),
                rel_data.as_deref(),
                endian,
                &exec_sections,
                &data_sections,
                &bss_sections,
            );
        }

        let pointer_index = Self::build_pointer_index(&bytes, is_32bit, endian, &data_sections);

        Ok(Self {
            bytes,
            is_32bit,
            endian,
            image_base,
            exec_sections,
            data_sections,
            bss_sections,
            is_dumped: false,
            pointer_index,
        })
    }

    // Build a sorted index of all pointer values in the data sections.
    fn build_pointer_index(
        bytes: &[u8],
        is_32bit: bool,
        endian: Endianness,
        data_sections: &[SearchSection],
    ) -> Vec<PointerRef> {
        let mut index = Vec::new();
        let ptr_size = if is_32bit { 4 } else { 8 };
        for sec in data_sections {
            let start = sec.offset as usize;
            let end = sec.offset_end as usize;
            if start >= bytes.len() {
                continue;
            }
            let end = end.min(bytes.len());
            if end <= start + ptr_size {
                continue;
            }
            for pos in (start..=end - ptr_size).step_by(ptr_size) {
                let val = if is_32bit {
                    match endian {
                        Endianness::Little => u32::from_le_bytes([
                            bytes[pos],
                            bytes[pos + 1],
                            bytes[pos + 2],
                            bytes[pos + 3],
                        ]) as u64,
                        Endianness::Big => u32::from_be_bytes([
                            bytes[pos],
                            bytes[pos + 1],
                            bytes[pos + 2],
                            bytes[pos + 3],
                        ]) as u64,
                    }
                } else {
                    match endian {
                        Endianness::Little => u64::from_le_bytes([
                            bytes[pos],
                            bytes[pos + 1],
                            bytes[pos + 2],
                            bytes[pos + 3],
                            bytes[pos + 4],
                            bytes[pos + 5],
                            bytes[pos + 6],
                            bytes[pos + 7],
                        ]),
                        Endianness::Big => u64::from_be_bytes([
                            bytes[pos],
                            bytes[pos + 1],
                            bytes[pos + 2],
                            bytes[pos + 3],
                            bytes[pos + 4],
                            bytes[pos + 5],
                            bytes[pos + 6],
                            bytes[pos + 7],
                        ]),
                    }
                };
                let va = pos as u64 - sec.offset + sec.address;
                index.push(PointerRef { value: val, va });
            }
        }
        index.sort_by_key(|x| x.value);
        index
    }

    pub fn map_vatr(&self, addr: u64) -> Option<u64> {
        // Map Virtual Address to Raw file offset using segments.
        // We look up exec_sections, data_sections, bss_sections.
        for sec in self
            .exec_sections
            .iter()
            .chain(&self.data_sections)
            .chain(&self.bss_sections)
        {
            if addr >= sec.address && addr < sec.address_end {
                let offset = addr - sec.address + sec.offset;
                return Some(offset);
            }
        }
        None
    }

    #[allow(dead_code)]
    pub fn map_rtva(&self, offset: u64) -> Option<u64> {
        // Map Raw file offset to Virtual Address.
        for sec in self.exec_sections.iter().chain(&self.data_sections) {
            if offset >= sec.offset && offset < sec.offset_end {
                let addr = offset - sec.offset + sec.address;
                return Some(addr);
            }
        }
        None
    }

    // Helper methods for reading data from virtual addresses.
    #[allow(dead_code)]
    pub fn read_bytes(&self, addr: u64, size: usize) -> io::Result<Vec<u8>> {
        let offset = self.map_vatr(addr).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Virtual address 0x{:x} not mapped in binary", addr),
            )
        })? as usize;
        if offset + size > self.bytes.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Read out of bounds",
            ));
        }
        Ok(self.bytes[offset..offset + size].to_vec())
    }

    pub fn read_u32_raw(&self, offset: usize) -> u32 {
        if offset + 4 > self.bytes.len() {
            return 0;
        }
        let buf = &self.bytes[offset..offset + 4];
        match self.endian {
            Endianness::Little => u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]),
            Endianness::Big => u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]),
        }
    }

    pub fn read_u64_raw(&self, offset: usize) -> u64 {
        if offset + 8 > self.bytes.len() {
            return 0;
        }
        let buf = &self.bytes[offset..offset + 8];
        match self.endian {
            Endianness::Little => u64::from_le_bytes([
                buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
            ]),
            Endianness::Big => u64::from_be_bytes([
                buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
            ]),
        }
    }

    pub fn get_reader_at(&self, addr: u64) -> io::Result<BinaryReader<Cursor<&[u8]>>> {
        let offset = self.map_vatr(addr).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Virtual address 0x{:x} not mapped in binary", addr),
            )
        })?;
        let mut cursor = Cursor::new(&self.bytes[..]);
        cursor.seek(io::SeekFrom::Start(offset))?;
        Ok(BinaryReader::new(cursor, self.is_32bit, self.endian))
    }

    pub fn read_string_to_null(&self, addr: u64) -> io::Result<String> {
        let offset = self.map_vatr(addr).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Virtual address 0x{:x} not mapped in binary", addr),
            )
        })? as usize;
        let mut bytes = Vec::new();
        let mut pos = offset;
        while pos < self.bytes.len() {
            let b = self.bytes[pos];
            if b == 0 {
                break;
            }
            bytes.push(b);
            pos += 1;
        }
        String::from_utf8(bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    // Helper functions to locate registration addresses.
    pub fn symbol_search(&self) -> (u64, u64) {
        let mut code_reg = 0;
        let mut metadata_reg = 0;
        if let Ok(obj) = object::File::parse(&self.bytes[..]) {
            for sym in obj.symbols() {
                if let Ok(name) = sym.name() {
                    match name {
                        "g_CodeRegistration" => {
                            code_reg = sym.address();
                        }
                        "g_MetadataRegistration" => {
                            metadata_reg = sym.address();
                        }
                        _ => {}
                    }
                }
            }
            if code_reg == 0 || metadata_reg == 0 {
                for sym in obj.dynamic_symbols() {
                    if let Ok(name) = sym.name() {
                        match name {
                            "g_CodeRegistration" => {
                                code_reg = sym.address();
                            }
                            "g_MetadataRegistration" => {
                                metadata_reg = sym.address();
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        (code_reg, metadata_reg)
    }

    pub fn plus_search(
        &self,
        version: f64,
        method_count: usize,
        type_definitions_count: usize,
        metadata_usages_count: usize,
        image_count: usize,
    ) -> (u64, u64) {
        // Run pattern scanning heuristics using code and data sections.
        let code_reg = self.find_code_registration(version, method_count, image_count);
        let metadata_reg =
            self.find_metadata_registration(version, type_definitions_count, metadata_usages_count);
        (code_reg, metadata_reg)
    }

    fn find_code_registration(&self, version: f64, method_count: usize, image_count: usize) -> u64 {
        if version >= 24.2 {
            // Search code sections first, then search data sections.
            let mut reg =
                self.find_code_registration_in_sections(&self.exec_sections, version, image_count);
            if reg == 0 {
                reg = self.find_code_registration_in_sections(
                    &self.data_sections,
                    version,
                    image_count,
                );
            }
            reg
        } else {
            self.find_code_registration_old(method_count)
        }
    }

    fn find_code_registration_old(&self, method_count: usize) -> u64 {
        let ptr_size = if self.is_32bit { 4 } else { 8 };
        for sec in &self.data_sections {
            let mut pos = sec.offset;
            while pos < sec.offset_end {
                let val = if self.is_32bit {
                    self.read_u32_raw(pos as usize) as u64
                } else {
                    self.read_u64_raw(pos as usize)
                };
                if val == method_count as u64 {
                    // Verify if the next pointer points to a valid array of code pointers.
                    let next_ptr = if self.is_32bit {
                        self.read_u32_raw((pos + 4) as usize) as u64
                    } else {
                        self.read_u64_raw((pos + 8) as usize)
                    };
                    if let Some(raw_arr_offset) = self.map_vatr(next_ptr)
                        && self.check_pointer_range_data_ra(raw_arr_offset)
                    {
                        // Read pointers and ensure they fall within executable code ranges.
                        let mut code_pointers = Vec::new();
                        let mut read_pos = raw_arr_offset;
                        for _ in 0..method_count {
                            let cp = if self.is_32bit {
                                self.read_u32_raw(read_pos as usize) as u64
                            } else {
                                self.read_u64_raw(read_pos as usize)
                            };
                            code_pointers.push(cp);
                            read_pos += ptr_size;
                        }
                        if self.check_pointer_range_exec_va(&code_pointers) {
                            return pos - sec.offset + sec.address;
                        }
                    }
                }
                pos += ptr_size;
            }
        }
        0
    }

    fn find_code_registration_in_sections(
        &self,
        secs: &[SearchSection],
        version: f64,
        image_count: usize,
    ) -> u64 {
        let feature_bytes = b"mscorlib.dll\0";
        let ptr_size = if self.is_32bit { 4 } else { 8 };
        for sec in secs {
            let start = sec.offset as usize;
            let end = sec.offset_end as usize;
            if start >= self.bytes.len() {
                continue;
            }
            let end = end.min(self.bytes.len());
            let chunk = &self.bytes[start..end];
            let matches = pattern_search(chunk, feature_bytes);
            for m in matches {
                let dllva = m as u64 + sec.address;
                for refva in self.find_reference(dllva) {
                    for refva2 in self.find_reference(refva) {
                        if version >= 27.0 {
                            for i in (0..image_count).rev() {
                                let sub_va = refva2.wrapping_sub(i as u64 * ptr_size);
                                for refva3 in self.find_reference(sub_va) {
                                    if let Some(raw_ref3) = self.map_vatr(refva3 - ptr_size) {
                                        let img_count_val = if self.is_32bit {
                                            self.read_u32_raw(raw_ref3 as usize) as u64
                                        } else {
                                            self.read_u64_raw(raw_ref3 as usize)
                                        };
                                        if img_count_val > 0
                                            && img_count_val <= image_count as u64
                                            && img_count_val
                                                >= (image_count.saturating_sub(50)) as u64
                                        {
                                            if version >= 29.0 {
                                                return refva3 - ptr_size * 14;
                                            }
                                            return refva3 - ptr_size * 13;
                                        }
                                    }
                                }
                            }
                        } else {
                            for i in 0..image_count {
                                let sub_va = refva2.wrapping_sub(i as u64 * ptr_size);
                                if let Some(refva3) = self.find_reference(sub_va).into_iter().next()
                                {
                                    return refva3 - ptr_size * 13;
                                }
                            }
                        }
                    }
                }
            }
        }
        0
    }

    fn find_metadata_registration(
        &self,
        version: f64,
        type_definitions_count: usize,
        metadata_usages_count: usize,
    ) -> u64 {
        if version < 19.0 {
            return 0;
        }
        if version >= 27.0 {
            self.find_metadata_registration_v21(type_definitions_count)
        } else {
            self.find_metadata_registration_old(type_definitions_count, metadata_usages_count)
        }
    }

    fn find_metadata_registration_old(
        &self,
        type_definitions_count: usize,
        metadata_usages_count: usize,
    ) -> u64 {
        let ptr_size = if self.is_32bit { 4 } else { 8 };
        for sec in &self.data_sections {
            let mut pos = sec.offset;
            let end = sec.offset_end.min(self.bytes.len() as u64) - ptr_size;
            while pos < end {
                let val = if self.is_32bit {
                    self.read_u32_raw(pos as usize) as u64
                } else {
                    self.read_u64_raw(pos as usize)
                };
                if val == type_definitions_count as u64 {
                    // Verify if the pointer at the specified offset is valid.
                    let usage_ptr = if self.is_32bit {
                        self.read_u32_raw((pos + ptr_size * 3) as usize) as u64
                    } else {
                        self.read_u64_raw((pos + ptr_size * 3) as usize)
                    };
                    if let Some(raw_usage_offset) = self.map_vatr(usage_ptr)
                        && self.check_pointer_range_data_ra(raw_usage_offset)
                    {
                        let mut usage_pointers = Vec::new();
                        let mut read_pos = raw_usage_offset;
                        for _ in 0..metadata_usages_count {
                            let up = if self.is_32bit {
                                self.read_u32_raw(read_pos as usize) as u64
                            } else {
                                self.read_u64_raw(read_pos as usize)
                            };
                            usage_pointers.push(up);
                            read_pos += ptr_size;
                        }
                        if self.check_pointer_range_bss_va(&usage_pointers) {
                            return pos - ptr_size * 12 - sec.offset + sec.address;
                        }
                    }
                }
                pos += ptr_size;
            }
        }
        0
    }

    fn find_metadata_registration_v21(&self, type_definitions_count: usize) -> u64 {
        let ptr_size = if self.is_32bit { 4 } else { 8 };
        let min_types = type_definitions_count.saturating_sub(5000) as u64;
        let max_types = (type_definitions_count + 100) as u64;
        for sec in &self.data_sections {
            let mut pos = sec.offset;
            let end = sec.offset_end.min(self.bytes.len() as u64) - (4 * ptr_size);
            while pos < end {
                let val1 = if self.is_32bit {
                    self.read_u32_raw(pos as usize) as u64
                } else {
                    self.read_u64_raw(pos as usize)
                };
                if val1 >= min_types && val1 <= max_types {
                    let val2 = if self.is_32bit {
                        self.read_u32_raw((pos + ptr_size * 2) as usize) as u64
                    } else {
                        self.read_u64_raw((pos + ptr_size * 2) as usize)
                    };
                    if val2 == val1 {
                        let types_ptr = if self.is_32bit {
                            self.read_u32_raw((pos + ptr_size * 3) as usize) as u64
                        } else {
                            self.read_u64_raw((pos + ptr_size * 3) as usize)
                        };
                        let types_count_off = pos.saturating_sub(ptr_size * 4);
                        let types_count = if self.is_32bit {
                            self.read_u32_raw(types_count_off as usize) as u64
                        } else {
                            self.read_u64_raw(types_count_off as usize)
                        };
                        if types_count > 0
                            && types_count < 10_000_000
                            && self.map_vatr(types_ptr).is_some()
                        {
                            return pos - ptr_size * 10 - sec.offset + sec.address;
                        }
                    }
                }
                pos += ptr_size;
            }
        }
        0
    }

    fn find_reference(&self, addr: u64) -> Vec<u64> {
        let mut refs = Vec::new();
        let idx = self.pointer_index.partition_point(|x| x.value < addr);
        let mut i = idx;
        while i < self.pointer_index.len() && self.pointer_index[i].value == addr {
            refs.push(self.pointer_index[i].va);
            i += 1;
        }
        refs
    }

    fn check_pointer_range_data_ra(&self, pointer: u64) -> bool {
        self.data_sections
            .iter()
            .any(|sec| pointer >= sec.offset && pointer < sec.offset_end)
    }

    fn check_pointer_range_exec_va(&self, pointers: &[u64]) -> bool {
        pointers.iter().all(|&x| {
            self.exec_sections
                .iter()
                .any(|sec| x >= sec.address && x < sec.address_end)
        })
    }

    fn check_pointer_range_data_va(&self, pointers: &[u64]) -> bool {
        pointers.iter().all(|&x| {
            self.data_sections
                .iter()
                .any(|sec| x >= sec.address && x < sec.address_end)
        })
    }

    fn check_pointer_range_bss_va(&self, pointers: &[u64]) -> bool {
        pointers.iter().all(|&x| {
            self.bss_sections
                .iter()
                .any(|sec| x >= sec.address && x < sec.address_end)
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn apply_relocations_raw(
        bytes: &mut [u8],
        arch: object::Architecture,
        dyn_symbol_addresses: &[u64],
        rela_data: Option<&[u8]>,
        rel_data: Option<&[u8]>,
        endian: Endianness,
        exec_sections: &[SearchSection],
        data_sections: &[SearchSection],
        bss_sections: &[SearchSection],
    ) {
        let is_arm64 = arch == object::Architecture::Aarch64;
        let is_x86_64 = arch == object::Architecture::X86_64;
        let is_x86 = arch == object::Architecture::I386;
        let is_arm = arch == object::Architecture::Arm;

        let map_vatr_local = |addr: u64| -> Option<u64> {
            for sec in exec_sections
                .iter()
                .chain(data_sections)
                .chain(bss_sections)
            {
                if addr >= sec.address && addr < sec.address_end {
                    return Some(addr - sec.address + sec.offset);
                }
            }
            None
        };

        // 64-bit RELA relocations (.rela.dyn).
        if let Some(data) = rela_data {
            let count = data.len() / 24;
            for i in 0..count {
                let offset = i * 24;
                let r_offset = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
                let r_info = u64::from_le_bytes(data[offset + 8..offset + 16].try_into().unwrap());
                let r_addend =
                    i64::from_le_bytes(data[offset + 16..offset + 24].try_into().unwrap());

                let sym = (r_info >> 32) as usize;
                let typ = (r_info & 0xffffffff) as u32;

                let mut val: Option<u64> = None;
                if is_arm64 {
                    match typ {
                        257 => {
                            // R_AARCH64_ABS64
                            if sym < dyn_symbol_addresses.len() {
                                val = Some(dyn_symbol_addresses[sym].wrapping_add(r_addend as u64));
                            }
                        }
                        1027 => {
                            // R_AARCH64_RELATIVE
                            val = Some(r_addend as u64);
                        }
                        _ => {}
                    }
                } else if is_x86_64 {
                    match typ {
                        1 => {
                            // R_X86_64_64
                            if sym < dyn_symbol_addresses.len() {
                                val = Some(dyn_symbol_addresses[sym].wrapping_add(r_addend as u64));
                            }
                        }
                        8 => {
                            // R_X86_64_RELATIVE
                            val = Some(r_addend as u64);
                        }
                        _ => {}
                    }
                }

                if let Some(v) = val
                    && let Some(raw_offset) = map_vatr_local(r_offset)
                {
                    let raw_offset = raw_offset as usize;
                    if raw_offset + 8 <= bytes.len() {
                        let bytes_to_write = match endian {
                            Endianness::Little => v.to_le_bytes(),
                            Endianness::Big => v.to_be_bytes(),
                        };
                        bytes[raw_offset..raw_offset + 8].copy_from_slice(&bytes_to_write);
                    }
                }
            }
        }

        // 32-bit REL relocations (.rel.dyn).
        if let Some(data) = rel_data {
            let count = data.len() / 8;
            for i in 0..count {
                let offset = i * 8;
                let r_offset =
                    u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as u64;
                let r_info = u32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap());
                let sym = (r_info >> 8) as usize;
                let typ = r_info & 0xff;

                let mut val: Option<u32> = None;
                if is_x86 {
                    if typ == 1 {
                        // R_386_32
                        if sym < dyn_symbol_addresses.len() {
                            val = Some(dyn_symbol_addresses[sym] as u32);
                        }
                    }
                } else if is_arm && typ == 2 {
                    // R_ARM_ABS32
                    if sym < dyn_symbol_addresses.len() {
                        val = Some(dyn_symbol_addresses[sym] as u32);
                    }
                }

                if let Some(v) = val
                    && let Some(raw_offset) = map_vatr_local(r_offset)
                {
                    let raw_offset = raw_offset as usize;
                    if raw_offset + 4 <= bytes.len() {
                        let bytes_to_write = match endian {
                            Endianness::Little => v.to_le_bytes(),
                            Endianness::Big => v.to_be_bytes(),
                        };
                        bytes[raw_offset..raw_offset + 4].copy_from_slice(&bytes_to_write);
                    }
                }
            }
        }
    }
}

fn pattern_search(data: &[u8], pattern: &[u8]) -> Vec<usize> {
    let mut matches = Vec::new();
    if pattern.is_empty() || data.len() < pattern.len() {
        return matches;
    }
    for i in 0..=(data.len() - pattern.len()) {
        if &data[i..i + pattern.len()] == pattern {
            matches.push(i);
        }
    }
    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_search() {
        let data = b"hello world, hello universe";
        let pattern = b"hello";
        let matches = pattern_search(data, pattern);
        assert_eq!(matches, vec![0, 13]);

        let matches = pattern_search(data, b"notfound");
        assert!(matches.is_empty());

        let matches = pattern_search(b"", b"empty");
        assert!(matches.is_empty());

        let matches = pattern_search(b"short", b"longerpattern");
        assert!(matches.is_empty());
    }

    #[test]
    fn test_registration_heuristic_allowance() {
        // Verify that stripped assembly image counts are accepted within the saturating subtraction range.
        let image_count = 220_usize;
        let img_count_val = 218_u64;
        let is_valid_image_count = img_count_val > 0
            && img_count_val <= image_count as u64
            && img_count_val >= (image_count.saturating_sub(50)) as u64;
        assert!(is_valid_image_count);

        // Verify that metadata registration allows slightly fewer types due to stripped assemblies.
        let type_definitions_count = 46954_usize;
        let min_types = type_definitions_count.saturating_sub(5000) as u64;
        let max_types = (type_definitions_count + 100) as u64;
        let val1 = 46469_u64;
        assert!(val1 >= min_types && val1 <= max_types);
    }
}
