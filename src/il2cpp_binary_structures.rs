#![allow(dead_code, clippy::field_reassign_with_default)]
use crate::binary_reader::BinaryReader;
use std::io::{self, Read, Seek};

#[derive(Debug, Clone, Default)]
pub struct Il2CppCodeRegistration {
    pub method_pointers_count: u64,                          // <= 24.1
    pub method_pointers: u64,                                // <= 24.1
    pub delegate_wrappers_from_native_to_managed_count: u64, // <= 21.0
    pub delegate_wrappers_from_native_to_managed: u64,       // <= 21.0
    pub reverse_pinvoke_wrapper_count: u64,                  // >= 22.0
    pub reverse_pinvoke_wrappers: u64,                       // >= 22.0
    pub delegate_wrappers_from_managed_to_native_count: u64, // <= 22.0
    pub delegate_wrappers_from_managed_to_native: u64,       // <= 22.0
    pub marshaling_functions_count: u64,                     // <= 22.0
    pub marshaling_functions: u64,                           // <= 22.0
    pub ccw_marshaling_functions_count: u64,                 // >= 21.0, <= 22.0
    pub ccw_marshaling_functions: u64,                       // >= 21.0, <= 22.0
    pub generic_method_pointers_count: u64,
    pub generic_method_pointers: u64,
    pub generic_adjustor_thunks: u64, // 24.5 only, or >= 27.1
    pub invoker_pointers_count: u64,
    pub invoker_pointers: u64,
    pub custom_attribute_count: u64,            // <= 24.5
    pub custom_attribute_generators: u64,       // <= 24.5
    pub guid_count: u64,                        // >= 21.0, <= 22.0
    pub guids: u64,                             // >= 21.0, <= 22.0
    pub unresolved_virtual_call_count: u64,     // >= 22.0
    pub unresolved_virtual_call_pointers: u64,  // >= 22.0
    pub unresolved_instance_call_pointers: u64, // >= 29.1
    pub unresolved_static_call_pointers: u64,   // >= 29.1
    pub interop_data_count: u64,                // >= 23.0
    pub interop_data: u64,                      // >= 23.0
    pub windows_runtime_factory_count: u64,     // >= 24.3
    pub windows_runtime_factory_table: u64,     // >= 24.3
    pub code_gen_modules_count: u64,            // >= 24.2
    pub code_gen_modules: u64,                  // >= 24.2
}

impl Il2CppCodeRegistration {
    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>, version: f64) -> io::Result<Self> {
        let mut c = Self::default();
        if version <= 24.1 {
            c.method_pointers_count = r.read_ptr()?;
            c.method_pointers = r.read_ptr()?;
        }
        if version <= 21.0 {
            c.delegate_wrappers_from_native_to_managed_count = r.read_ptr()?;
            c.delegate_wrappers_from_native_to_managed = r.read_ptr()?;
        }
        if version >= 22.0 {
            c.reverse_pinvoke_wrapper_count = r.read_ptr()?;
            c.reverse_pinvoke_wrappers = r.read_ptr()?;
        }
        if version <= 22.0 {
            c.delegate_wrappers_from_managed_to_native_count = r.read_ptr()?;
            c.delegate_wrappers_from_managed_to_native = r.read_ptr()?;
            c.marshaling_functions_count = r.read_ptr()?;
            c.marshaling_functions = r.read_ptr()?;
        }
        if (21.0..=22.0).contains(&version) {
            c.ccw_marshaling_functions_count = r.read_ptr()?;
            c.ccw_marshaling_functions = r.read_ptr()?;
        }
        c.generic_method_pointers_count = r.read_ptr()?;
        c.generic_method_pointers = r.read_ptr()?;
        if (24.5..25.0).contains(&version) || version >= 27.1 {
            c.generic_adjustor_thunks = r.read_ptr()?;
        }
        c.invoker_pointers_count = r.read_ptr()?;
        c.invoker_pointers = r.read_ptr()?;
        if version <= 24.5 {
            c.custom_attribute_count = r.read_ptr()?;
            c.custom_attribute_generators = r.read_ptr()?;
        }
        if (21.0..=22.0).contains(&version) {
            c.guid_count = r.read_ptr()?;
            c.guids = r.read_ptr()?;
        }
        if version >= 22.0 {
            c.unresolved_virtual_call_count = r.read_ptr()?;
            c.unresolved_virtual_call_pointers = r.read_ptr()?;
        }
        if version >= 29.1 {
            c.unresolved_instance_call_pointers = r.read_ptr()?;
            c.unresolved_static_call_pointers = r.read_ptr()?;
        }
        if version >= 23.0 {
            c.interop_data_count = r.read_ptr()?;
            c.interop_data = r.read_ptr()?;
        }
        if version >= 24.3 {
            c.windows_runtime_factory_count = r.read_ptr()?;
            c.windows_runtime_factory_table = r.read_ptr()?;
        }
        if version >= 24.2 {
            c.code_gen_modules_count = r.read_ptr()?;
            c.code_gen_modules = r.read_ptr()?;
        }
        Ok(c)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Il2CppMetadataRegistration {
    pub generic_classes_count: i64,
    pub generic_classes: u64,
    pub generic_insts_count: i64,
    pub generic_insts: u64,
    pub generic_method_table_count: i64,
    pub generic_method_table: u64,
    pub types_count: i64,
    pub types: u64,
    pub method_specs_count: i64,
    pub method_specs: u64,
    pub method_references_count: i64, // <= 16
    pub method_references: u64,       // <= 16
    pub field_offsets_count: i64,
    pub field_offsets: u64,
    pub type_definitions_sizes_count: i64,
    pub type_definitions_sizes: u64,
    pub metadata_usages_count: u64, // >= 19
    pub metadata_usages: u64,       // >= 19
}

impl Il2CppMetadataRegistration {
    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>, version: f64) -> io::Result<Self> {
        let mut m = Self::default();
        m.generic_classes_count = r.read_iptr()?;
        m.generic_classes = r.read_ptr()?;
        m.generic_insts_count = r.read_iptr()?;
        m.generic_insts = r.read_ptr()?;
        m.generic_method_table_count = r.read_iptr()?;
        m.generic_method_table = r.read_ptr()?;
        m.types_count = r.read_iptr()?;
        m.types = r.read_ptr()?;
        m.method_specs_count = r.read_iptr()?;
        m.method_specs = r.read_ptr()?;
        if version <= 16.0 {
            m.method_references_count = r.read_iptr()?;
            m.method_references = r.read_ptr()?;
        }
        m.field_offsets_count = r.read_iptr()?;
        m.field_offsets = r.read_ptr()?;
        m.type_definitions_sizes_count = r.read_iptr()?;
        m.type_definitions_sizes = r.read_ptr()?;
        if version >= 19.0 {
            m.metadata_usages_count = r.read_ptr()?;
            m.metadata_usages = r.read_ptr()?;
        }
        Ok(m)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Il2CppTypeEnum {
    Void = 0x01,
    Boolean = 0x02,
    Char = 0x03,
    I1 = 0x04,
    U1 = 0x05,
    I2 = 0x06,
    U2 = 0x07,
    I4 = 0x08,
    U4 = 0x09,
    I8 = 0x0a,
    U8 = 0x0b,
    R4 = 0x0c,
    R8 = 0x0d,
    String = 0x0e,
    Ptr = 0x0f,
    ByRef = 0x10,
    ValueType = 0x11,
    Class = 0x12,
    Var = 0x13,
    Array = 0x14,
    GenericInst = 0x15,
    TypedByRef = 0x16,
    I = 0x18,
    U = 0x19,
    FnPtr = 0x1b,
    Object = 0x1c,
    SzArray = 0x1d,
    MVar = 0x1e,
    CModReqd = 0x1f,
    CModOpt = 0x20,
    Internal = 0x21,
    Modifier = 0x40,
    Sentinel = 0x41,
    Pinned = 0x45,
    Enum = 0x55,
    Index = 0xff,
}

impl Il2CppTypeEnum {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0x01 => Self::Void,
            0x02 => Self::Boolean,
            0x03 => Self::Char,
            0x04 => Self::I1,
            0x05 => Self::U1,
            0x06 => Self::I2,
            0x07 => Self::U2,
            0x08 => Self::I4,
            0x09 => Self::U4,
            0x0a => Self::I8,
            0x0b => Self::U8,
            0x0c => Self::R4,
            0x0d => Self::R8,
            0x0e => Self::String,
            0x0f => Self::Ptr,
            0x10 => Self::ByRef,
            0x11 => Self::ValueType,
            0x12 => Self::Class,
            0x13 => Self::Var,
            0x14 => Self::Array,
            0x15 => Self::GenericInst,
            0x16 => Self::TypedByRef,
            0x18 => Self::I,
            0x19 => Self::U,
            0x1b => Self::FnPtr,
            0x1c => Self::Object,
            0x1d => Self::SzArray,
            0x1e => Self::MVar,
            0x1f => Self::CModReqd,
            0x20 => Self::CModOpt,
            0x21 => Self::Internal,
            0x40 => Self::Modifier,
            0x41 => Self::Sentinel,
            0x45 => Self::Pinned,
            0x55 => Self::Enum,
            _ => Self::Index,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Il2CppType {
    pub datapoint: u64,
    pub bits: u32,
    pub attrs: u32,
    pub ty: u8, // Il2CppTypeEnum represented as u8
    pub num_mods: u32,
    pub byref: u32,
    pub pinned: u32,
    pub valuetype: u32,
}

impl Il2CppType {
    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>, version: f64) -> io::Result<Self> {
        let datapoint = r.read_ptr()?;
        let bits = r.read_u32()?;
        let attrs = bits & 0xffff;
        let ty = ((bits >> 16) & 0xff) as u8;
        let num_mods;
        let byref;
        let pinned;
        let valuetype;
        if version >= 27.2 {
            num_mods = (bits >> 24) & 0x1f;
            byref = (bits >> 29) & 1;
            pinned = (bits >> 30) & 1;
            valuetype = bits >> 31;
        } else {
            num_mods = (bits >> 24) & 0x3f;
            byref = (bits >> 30) & 1;
            pinned = bits >> 31;
            valuetype = 0; // Wait! in C# version: valuetype = bits >> 31 in v27.2+, and 0 in earlier version. Oh, in earlier versions `pinned` was bits >> 31, and `valuetype` didn't exist or was determined from somewhere else?
            // Actually, in the C# code:
            // num_mods = (bits >> 24) & 0x3f;
            // byref = (bits >> 30) & 1;
            // pinned = bits >> 31;
            // valuetype is default 0.
        }
        Ok(Self {
            datapoint,
            bits,
            attrs,
            ty,
            num_mods,
            byref,
            pinned,
            valuetype,
        })
    }

    pub fn type_enum(&self) -> Il2CppTypeEnum {
        Il2CppTypeEnum::from_u8(self.ty)
    }

    pub fn klass_index(&self) -> i64 {
        self.datapoint as i64
    }

    pub fn type_handle(&self) -> u64 {
        self.datapoint
    }

    pub fn array_type_ptr(&self) -> u64 {
        self.datapoint
    }

    pub fn generic_parameter_index(&self) -> i64 {
        self.datapoint as i64
    }

    pub fn generic_class_ptr(&self) -> u64 {
        self.datapoint
    }
}

#[derive(Debug, Clone, Default)]
pub struct Il2CppGenericClass {
    pub type_def_index: i64, // <= 24.5
    pub type_ptr: u64,       // >= 27.0
    pub context: Il2CppGenericContext,
    pub cached_class: u64,
}

impl Il2CppGenericClass {
    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>, version: f64) -> io::Result<Self> {
        let mut gc = Self::default();
        if version <= 24.5 {
            gc.type_def_index = r.read_iptr()?;
        } else {
            gc.type_ptr = r.read_ptr()?;
        }
        gc.context = Il2CppGenericContext::decode(r)?;
        gc.cached_class = r.read_ptr()?;
        Ok(gc)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Il2CppGenericContext {
    pub class_inst: u64,
    pub method_inst: u64,
}

impl Il2CppGenericContext {
    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>) -> io::Result<Self> {
        Ok(Self {
            class_inst: r.read_ptr()?,
            method_inst: r.read_ptr()?,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Il2CppGenericInst {
    pub type_argc: i64,
    pub type_argv: u64,
}

impl Il2CppGenericInst {
    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>) -> io::Result<Self> {
        Ok(Self {
            type_argc: r.read_iptr()?,
            type_argv: r.read_ptr()?,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Il2CppArrayType {
    pub etype: u64,
    pub rank: u8,
    pub numsizes: u8,
    pub numlobounds: u8,
    pub sizes: u64,
    pub lobounds: u64,
}

impl Il2CppArrayType {
    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>) -> io::Result<Self> {
        let etype = r.read_ptr()?;
        let rank = r.read_u8()?;
        let numsizes = r.read_u8()?;
        let numlobounds = r.read_u8()?;
        // Align reader? Usually the C# struct has layout pack or padding.
        // Wait, C# uses default pack which aligns fields of ArrayType.
        // Let's check how BinaryStream reads it. In C#, fields are:
        // public ulong etype;
        // public byte rank;
        // public byte numsizes;
        // public byte numlobounds;
        // public ulong sizes;
        // public ulong lobounds;
        // Since rank, numsizes, numlobounds occupy 3 bytes, there will be 5 bytes of padding to align the ulong `sizes` field on 64-bit boundaries (or 1 byte on 32-bit boundaries depending on pointer size).
        // Let's check: ReadPrimitive reads field by field. Does it do alignment padding?
        // Wait, in `IO/BinaryStream.cs` `ReadClass` method, it DOES NOT do any padding alignment unless configured!
        // Wait, C# `GetFields()` returns fields in layout order, but when reading via reflection:
        // `i.SetValue(t, ReadPrimitive(fieldType))`
        // It reads them directly one after another! So no padding!
        // Oh! `BinaryStream` in C# reads fields consecutively without padding!
        // Yes, `BinaryReader` reads bytes sequentially. So no padding is skipped.
        // So we just read rank, numsizes, numlobounds, then immediately read sizes and lobounds!
        let sizes = r.read_ptr()?;
        let lobounds = r.read_ptr()?;
        Ok(Self {
            etype,
            rank,
            numsizes,
            numlobounds,
            sizes,
            lobounds,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Il2CppGenericMethodFunctionsDefinitions {
    pub generic_method_index: i32,
    pub indices: Il2CppGenericMethodIndices,
}

impl Il2CppGenericMethodFunctionsDefinitions {
    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>, version: f64) -> io::Result<Self> {
        let generic_method_index = r.read_i32()?;
        let indices = Il2CppGenericMethodIndices::decode(r, version)?;
        Ok(Self {
            generic_method_index,
            indices,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Il2CppGenericMethodIndices {
    pub method_index: i32,
    pub invoker_index: i32,
    pub adjustor_thunk: i32, // >= 24.5, <= 24.5 or >= 27.1
}

impl Il2CppGenericMethodIndices {
    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>, version: f64) -> io::Result<Self> {
        let method_index = r.read_i32()?;
        let invoker_index = r.read_i32()?;
        let mut adjustor_thunk = 0;
        if (24.5..25.0).contains(&version) || version >= 27.1 {
            adjustor_thunk = r.read_i32()?;
        }
        Ok(Self {
            method_index,
            invoker_index,
            adjustor_thunk,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Il2CppMethodSpec {
    pub method_definition_index: i32,
    pub class_index_index: i32,
    pub method_index_index: i32,
}

impl Il2CppMethodSpec {
    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>) -> io::Result<Self> {
        Ok(Self {
            method_definition_index: r.read_i32()?,
            class_index_index: r.read_i32()?,
            method_index_index: r.read_i32()?,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Il2CppCodeGenModule {
    pub module_name: u64,
    pub method_pointer_count: i64,
    pub method_pointers: u64,
    pub adjustor_thunk_count: i64,
    pub adjustor_thunks: u64,
    pub invoker_indices: u64,
    pub reverse_pinvoke_wrapper_count: u64,
    pub reverse_pinvoke_wrapper_indices: u64,
    pub rgctx_ranges_count: i64,
    pub rgctx_ranges: u64,
    pub rgctxs_count: i64,
    pub rgctxs: u64,
    pub debugger_metadata: u64,
    pub custom_attribute_cache_generator: u64, // >= 27.0, <= 27.2
    pub module_initializer: u64,               // >= 27.0
    pub static_constructor_type_indices: u64,  // >= 27.0
    pub metadata_registration: u64,            // >= 27.0
    pub code_registration: u64,                // >= 27.0
}

impl Il2CppCodeGenModule {
    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>, version: f64) -> io::Result<Self> {
        let mut c = Self::default();
        c.module_name = r.read_ptr()?;
        c.method_pointer_count = r.read_iptr()?;
        c.method_pointers = r.read_ptr()?;
        if (24.5..25.0).contains(&version) || version >= 27.1 {
            c.adjustor_thunk_count = r.read_iptr()?;
            c.adjustor_thunks = r.read_ptr()?;
        }
        c.invoker_indices = r.read_ptr()?;
        c.reverse_pinvoke_wrapper_count = r.read_ptr()?;
        c.reverse_pinvoke_wrapper_indices = r.read_ptr()?;
        c.rgctx_ranges_count = r.read_iptr()?;
        c.rgctx_ranges = r.read_ptr()?;
        c.rgctxs_count = r.read_iptr()?;
        c.rgctxs = r.read_ptr()?;
        c.debugger_metadata = r.read_ptr()?;
        if (27.0..=27.2).contains(&version) {
            c.custom_attribute_cache_generator = r.read_ptr()?;
        }
        if version >= 27.0 {
            c.module_initializer = r.read_ptr()?;
            c.static_constructor_type_indices = r.read_ptr()?;
            c.metadata_registration = r.read_ptr()?;
            c.code_registration = r.read_ptr()?;
        }
        Ok(c)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Il2CppRange {
    pub start: i32,
    pub length: i32,
}

impl Il2CppRange {
    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>) -> io::Result<Self> {
        Ok(Self {
            start: r.read_i32()?,
            length: r.read_i32()?,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Il2CppTokenRangePair {
    pub token: u32,
    pub range: Il2CppRange,
}

impl Il2CppTokenRangePair {
    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>) -> io::Result<Self> {
        let token = r.read_u32()?;
        let range = Il2CppRange::decode(r)?;
        Ok(Self { token, range })
    }
}
