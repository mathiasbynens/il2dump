#![allow(dead_code, clippy::field_reassign_with_default)]
use crate::binary_reader::{BinaryReader, Endianness};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::io::{self, Read, Seek};

#[derive(Debug, Clone, Default)]
pub struct Il2CppGlobalMetadataHeader {
    pub sanity: u32,
    pub version: i32,
    pub string_literal_offset: u32,
    pub string_literal_size: i32,
    pub string_literal_data_offset: u32,
    pub string_literal_data_size: i32,
    pub string_offset: u32,
    pub string_size: i32,
    pub events_offset: u32,
    pub events_size: i32,
    pub properties_offset: u32,
    pub properties_size: i32,
    pub methods_offset: u32,
    pub methods_size: i32,
    pub parameter_default_values_offset: u32,
    pub parameter_default_values_size: i32,
    pub field_default_values_offset: u32,
    pub field_default_values_size: i32,
    pub field_and_parameter_default_value_data_offset: u32,
    pub field_and_parameter_default_value_data_size: i32,
    pub field_marshaled_sizes_offset: i32,
    pub field_marshaled_sizes_size: i32,
    pub parameters_offset: u32,
    pub parameters_size: i32,
    pub fields_offset: u32,
    pub fields_size: i32,
    pub generic_parameters_offset: u32,
    pub generic_parameters_size: i32,
    pub generic_parameter_constraints_offset: u32,
    pub generic_parameter_constraints_size: i32,
    pub generic_containers_offset: u32,
    pub generic_containers_size: i32,
    pub nested_types_offset: u32,
    pub nested_types_size: i32,
    pub interfaces_offset: u32,
    pub interfaces_size: i32,
    pub vtable_methods_offset: u32,
    pub vtable_methods_size: i32,
    pub interface_offsets_offset: i32,
    pub interface_offsets_size: i32,
    pub type_definitions_offset: u32,
    pub type_definitions_size: i32,
    pub rgctx_entries_offset: u32,
    pub rgctx_entries_count: i32,
    pub images_offset: u32,
    pub images_size: i32,
    pub assemblies_offset: u32,
    pub assemblies_size: i32,
    pub metadata_usage_lists_offset: u32,
    pub metadata_usage_lists_count: i32,
    pub metadata_usage_pairs_offset: u32,
    pub metadata_usage_pairs_count: i32,
    pub field_refs_offset: u32,
    pub field_refs_size: i32,
    pub referenced_assemblies_offset: i32,
    pub referenced_assemblies_size: i32,
    pub attributes_info_offset: u32,
    pub attributes_info_count: i32,
    pub attribute_types_offset: u32,
    pub attribute_types_count: i32,
    pub attribute_data_offset: u32,
    pub attribute_data_size: i32,
    pub attribute_data_range_offset: u32,
    pub attribute_data_range_size: i32,
    pub unresolved_virtual_call_parameter_types_offset: i32,
    pub unresolved_virtual_call_parameter_types_size: i32,
    pub unresolved_virtual_call_parameter_ranges_offset: i32,
    pub unresolved_virtual_call_parameter_ranges_size: i32,
    pub windows_runtime_type_names_offset: i32,
    pub windows_runtime_type_names_size: i32,
    pub windows_runtime_strings_offset: i32,
    pub windows_runtime_strings_size: i32,
    pub exported_type_definitions_offset: i32,
    pub exported_type_definitions_size: i32,
}

impl Il2CppGlobalMetadataHeader {
    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>, version: f64) -> io::Result<Self> {
        let mut h = Self::default();
        h.sanity = r.read_u32()?;
        h.version = r.read_i32()?;
        h.string_literal_offset = r.read_u32()?;
        h.string_literal_size = r.read_i32()?;
        h.string_literal_data_offset = r.read_u32()?;
        h.string_literal_data_size = r.read_i32()?;
        h.string_offset = r.read_u32()?;
        h.string_size = r.read_i32()?;
        h.events_offset = r.read_u32()?;
        h.events_size = r.read_i32()?;
        h.properties_offset = r.read_u32()?;
        h.properties_size = r.read_i32()?;
        h.methods_offset = r.read_u32()?;
        h.methods_size = r.read_i32()?;
        h.parameter_default_values_offset = r.read_u32()?;
        h.parameter_default_values_size = r.read_i32()?;
        h.field_default_values_offset = r.read_u32()?;
        h.field_default_values_size = r.read_i32()?;
        h.field_and_parameter_default_value_data_offset = r.read_u32()?;
        h.field_and_parameter_default_value_data_size = r.read_i32()?;
        h.field_marshaled_sizes_offset = r.read_i32()?;
        h.field_marshaled_sizes_size = r.read_i32()?;
        h.parameters_offset = r.read_u32()?;
        h.parameters_size = r.read_i32()?;
        h.fields_offset = r.read_u32()?;
        h.fields_size = r.read_i32()?;
        h.generic_parameters_offset = r.read_u32()?;
        h.generic_parameters_size = r.read_i32()?;
        h.generic_parameter_constraints_offset = r.read_u32()?;
        h.generic_parameter_constraints_size = r.read_i32()?;
        h.generic_containers_offset = r.read_u32()?;
        h.generic_containers_size = r.read_i32()?;
        h.nested_types_offset = r.read_u32()?;
        h.nested_types_size = r.read_i32()?;
        h.interfaces_offset = r.read_u32()?;
        h.interfaces_size = r.read_i32()?;
        h.vtable_methods_offset = r.read_u32()?;
        h.vtable_methods_size = r.read_i32()?;
        h.interface_offsets_offset = r.read_i32()?;
        h.interface_offsets_size = r.read_i32()?;
        h.type_definitions_offset = r.read_u32()?;
        h.type_definitions_size = r.read_i32()?;
        if version <= 24.1 {
            h.rgctx_entries_offset = r.read_u32()?;
            h.rgctx_entries_count = r.read_i32()?;
        }
        h.images_offset = r.read_u32()?;
        h.images_size = r.read_i32()?;
        h.assemblies_offset = r.read_u32()?;
        h.assemblies_size = r.read_i32()?;
        if (19.0..=24.5).contains(&version) {
            h.metadata_usage_lists_offset = r.read_u32()?;
            h.metadata_usage_lists_count = r.read_i32()?;
            h.metadata_usage_pairs_offset = r.read_u32()?;
            h.metadata_usage_pairs_count = r.read_i32()?;
        }
        if version >= 19.0 {
            h.field_refs_offset = r.read_u32()?;
            h.field_refs_size = r.read_i32()?;
        }
        if version >= 20.0 {
            h.referenced_assemblies_offset = r.read_i32()?;
            h.referenced_assemblies_size = r.read_i32()?;
        }
        if (21.0..=27.2).contains(&version) {
            h.attributes_info_offset = r.read_u32()?;
            h.attributes_info_count = r.read_i32()?;
            h.attribute_types_offset = r.read_u32()?;
            h.attribute_types_count = r.read_i32()?;
        }
        if version >= 29.0 {
            h.attribute_data_offset = r.read_u32()?;
            h.attribute_data_size = r.read_i32()?;
            h.attribute_data_range_offset = r.read_u32()?;
            h.attribute_data_range_size = r.read_i32()?;
        }
        if version >= 22.0 {
            h.unresolved_virtual_call_parameter_types_offset = r.read_i32()?;
            h.unresolved_virtual_call_parameter_types_size = r.read_i32()?;
            h.unresolved_virtual_call_parameter_ranges_offset = r.read_i32()?;
            h.unresolved_virtual_call_parameter_ranges_size = r.read_i32()?;
        }
        if version >= 23.0 {
            h.windows_runtime_type_names_offset = r.read_i32()?;
            h.windows_runtime_type_names_size = r.read_i32()?;
        }
        if version >= 27.0 {
            h.windows_runtime_strings_offset = r.read_i32()?;
            h.windows_runtime_strings_size = r.read_i32()?;
        }
        if version >= 24.0 {
            h.exported_type_definitions_offset = r.read_i32()?;
            h.exported_type_definitions_size = r.read_i32()?;
        }
        Ok(h)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Il2CppAssemblyNameDefinition {
    pub name_index: u32,
    pub culture_index: u32,
    pub hash_value_index: i32,
    pub public_key_index: u32,
    pub hash_alg: u32,
    pub hash_len: i32,
    pub flags: u32,
    pub major: i32,
    pub minor: i32,
    pub build: i32,
    pub revision: i32,
    pub public_key_token: [u8; 8],
}

impl Il2CppAssemblyNameDefinition {
    pub fn size(version: f64) -> usize {
        let mut size = 4 + 4; // name_index, culture_index
        if version <= 24.3 {
            size += 4; // hash_value_index
        }
        size += 4 + 4 + 4 + 4 + 4 + 4 + 4 + 4; // public_key_index, hash_alg, hash_len, flags, major, minor, build, revision
        size += 8; // public_key_token
        size
    }

    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>, version: f64) -> io::Result<Self> {
        let name_index = r.read_u32()?;
        let culture_index = r.read_u32()?;
        let mut hash_value_index = 0;
        if version <= 24.3 {
            hash_value_index = r.read_i32()?;
        }
        let public_key_index = r.read_u32()?;
        let hash_alg = r.read_u32()?;
        let hash_len = r.read_i32()?;
        let flags = r.read_u32()?;
        let major = r.read_i32()?;
        let minor = r.read_i32()?;
        let build = r.read_i32()?;
        let revision = r.read_i32()?;
        let mut public_key_token = [0u8; 8];
        r.read_exact(&mut public_key_token)?;
        Ok(Self {
            name_index,
            culture_index,
            hash_value_index,
            public_key_index,
            hash_alg,
            hash_len,
            flags,
            major,
            minor,
            build,
            revision,
            public_key_token,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Il2CppAssemblyDefinition {
    pub image_index: i32,
    pub token: u32,
    pub custom_attribute_index: i32,
    pub referenced_assembly_start: i32,
    pub referenced_assembly_count: i32,
    pub aname: Il2CppAssemblyNameDefinition,
}

impl Il2CppAssemblyDefinition {
    pub fn size(version: f64) -> usize {
        let mut size = 4; // image_index
        if version >= 24.1 {
            size += 4; // token
        }
        if version <= 24.0 {
            size += 4; // custom_attribute_index
        }
        if version >= 20.0 {
            size += 4 + 4; // referenced_assembly_start, referenced_assembly_count
        }
        size += Il2CppAssemblyNameDefinition::size(version);
        size
    }

    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>, version: f64) -> io::Result<Self> {
        let image_index = r.read_i32()?;
        let mut token = 0;
        if version >= 24.1 {
            token = r.read_u32()?;
        }
        let mut custom_attribute_index = 0;
        if version <= 24.0 {
            custom_attribute_index = r.read_i32()?;
        }
        let mut referenced_assembly_start = 0;
        let mut referenced_assembly_count = 0;
        if version >= 20.0 {
            referenced_assembly_start = r.read_i32()?;
            referenced_assembly_count = r.read_i32()?;
        }
        let aname = Il2CppAssemblyNameDefinition::decode(r, version)?;
        Ok(Self {
            image_index,
            token,
            custom_attribute_index,
            referenced_assembly_start,
            referenced_assembly_count,
            aname,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Il2CppImageDefinition {
    pub name_index: u32,
    pub assembly_index: i32,
    pub type_start: i32,
    pub type_count: u32,
    pub exported_type_start: i32,
    pub exported_type_count: u32,
    pub entry_point_index: i32,
    pub token: u32,
    pub custom_attribute_start: i32,
    pub custom_attribute_count: u32,
}

impl Il2CppImageDefinition {
    pub fn size(version: f64) -> usize {
        let mut size = 4 + 4 + 4 + 4; // name_index, assembly_index, type_start, type_count
        if version >= 24.0 {
            size += 4 + 4; // exported_type_start, exported_type_count
        }
        size += 4; // entry_point_index
        if version >= 19.0 {
            size += 4; // token
        }
        if version >= 24.1 {
            size += 4 + 4; // custom_attribute_start, custom_attribute_count
        }
        size
    }

    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>, version: f64) -> io::Result<Self> {
        let name_index = r.read_u32()?;
        let assembly_index = r.read_i32()?;
        let type_start = r.read_i32()?;
        let type_count = r.read_u32()?;
        let mut exported_type_start = 0;
        let mut exported_type_count = 0;
        if version >= 24.0 {
            exported_type_start = r.read_i32()?;
            exported_type_count = r.read_u32()?;
        }
        let entry_point_index = r.read_i32()?;
        let mut token = 0;
        if version >= 19.0 {
            token = r.read_u32()?;
        }
        let mut custom_attribute_start = 0;
        let mut custom_attribute_count = 0;
        if version >= 24.1 {
            custom_attribute_start = r.read_i32()?;
            custom_attribute_count = r.read_u32()?;
        }
        Ok(Self {
            name_index,
            assembly_index,
            type_start,
            type_count,
            exported_type_start,
            exported_type_count,
            entry_point_index,
            token,
            custom_attribute_start,
            custom_attribute_count,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Il2CppTypeDefinition {
    pub name_index: u32,
    pub namespace_index: u32,
    pub custom_attribute_index: i32,
    pub byval_type_index: i32,
    pub byref_type_index: i32,
    pub declaring_type_index: i32,
    pub parent_index: i32,
    pub element_type_index: i32,
    pub rgctx_start_index: i32,
    pub rgctx_count: i32,
    pub generic_container_index: i32,
    pub delegate_wrapper_from_managed_to_native_index: i32,
    pub marshaling_functions_index: i32,
    pub ccw_function_index: i32,
    pub guid_index: i32,
    pub flags: u32,
    pub field_start: i32,
    pub method_start: i32,
    pub event_start: i32,
    pub property_start: i32,
    pub nested_types_start: i32,
    pub interfaces_start: i32,
    pub vtable_start: i32,
    pub interface_offsets_start: i32,
    pub method_count: u16,
    pub property_count: u16,
    pub field_count: u16,
    pub event_count: u16,
    pub nested_type_count: u16,
    pub vtable_count: u16,
    pub interfaces_count: u16,
    pub interface_offsets_count: u16,
    pub bitfield: u32,
    pub token: u32,
}

impl Il2CppTypeDefinition {
    pub fn size(version: f64) -> usize {
        let mut size = 4 + 4; // name_index, namespace_index
        if version <= 24.0 {
            size += 4; // custom_attribute_index
        }
        size += 4; // byval_type_index
        if version <= 24.5 {
            size += 4; // byref_type_index
        }
        size += 4 + 4 + 4; // declaring_type_index, parent_index, element_type_index
        if version <= 24.1 {
            size += 4 + 4; // rgctx_start_index, rgctx_count
        }
        size += 4; // generic_container_index
        if version <= 22.0 {
            size += 4 + 4; // delegate_wrapper_from_managed_to_native_index, marshaling_functions_index
        }
        if (21.0..=22.0).contains(&version) {
            size += 4 + 4; // ccw_function_index, guid_index
        }
        size += 4; // flags
        size += 4 + 4 + 4 + 4 + 4 + 4 + 4 + 4; // field_start, method_start, event_start, property_start, nested_types_start, interfaces_start, vtable_start, interface_offsets_start
        size += 2 + 2 + 2 + 2 + 2 + 2 + 2 + 2; // counts
        size += 4; // bitfield
        if version >= 19.0 {
            size += 4; // token
        }
        size
    }

    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>, version: f64) -> io::Result<Self> {
        let mut s = Self::default();
        s.name_index = r.read_u32()?;
        s.namespace_index = r.read_u32()?;
        if version <= 24.0 {
            s.custom_attribute_index = r.read_i32()?;
        }
        s.byval_type_index = r.read_i32()?;
        if version <= 24.5 {
            s.byref_type_index = r.read_i32()?;
        }
        s.declaring_type_index = r.read_i32()?;
        s.parent_index = r.read_i32()?;
        s.element_type_index = r.read_i32()?;
        if version <= 24.1 {
            s.rgctx_start_index = r.read_i32()?;
            s.rgctx_count = r.read_i32()?;
        }
        s.generic_container_index = r.read_i32()?;
        if version <= 22.0 {
            s.delegate_wrapper_from_managed_to_native_index = r.read_i32()?;
            s.marshaling_functions_index = r.read_i32()?;
        }
        if (21.0..=22.0).contains(&version) {
            s.ccw_function_index = r.read_i32()?;
            s.guid_index = r.read_i32()?;
        }
        s.flags = r.read_u32()?;
        s.field_start = r.read_i32()?;
        s.method_start = r.read_i32()?;
        s.event_start = r.read_i32()?;
        s.property_start = r.read_i32()?;
        s.nested_types_start = r.read_i32()?;
        s.interfaces_start = r.read_i32()?;
        s.vtable_start = r.read_i32()?;
        s.interface_offsets_start = r.read_i32()?;
        s.method_count = r.read_u16()?;
        s.property_count = r.read_u16()?;
        s.field_count = r.read_u16()?;
        s.event_count = r.read_u16()?;
        s.nested_type_count = r.read_u16()?;
        s.vtable_count = r.read_u16()?;
        s.interfaces_count = r.read_u16()?;
        s.interface_offsets_count = r.read_u16()?;
        s.bitfield = r.read_u32()?;
        if version >= 19.0 {
            s.token = r.read_u32()?;
        }
        Ok(s)
    }

    pub fn is_value_type(&self) -> bool {
        (self.bitfield & 0x1) == 1
    }

    pub fn is_enum(&self) -> bool {
        ((self.bitfield >> 1) & 0x1) == 1
    }
}

#[derive(Debug, Clone, Default)]
pub struct Il2CppMethodDefinition {
    pub name_index: u32,
    pub declaring_type: i32,
    pub return_type: i32,
    pub return_parameter_token: i32,
    pub parameter_start: i32,
    pub custom_attribute_index: i32,
    pub generic_container_index: i32,
    pub method_index: i32,
    pub invoker_index: i32,
    pub delegate_wrapper_index: i32,
    pub rgctx_start_index: i32,
    pub rgctx_count: i32,
    pub token: u32,
    pub flags: u16,
    pub iflags: u16,
    pub slot: u16,
    pub parameter_count: u16,
}

impl Il2CppMethodDefinition {
    pub fn size(version: f64) -> usize {
        let mut size = 4 + 4 + 4; // name_index, declaring_type, return_type
        if version >= 31.0 {
            size += 4; // return_parameter_token
        }
        size += 4; // parameter_start
        if version <= 24.0 {
            size += 4; // custom_attribute_index
        }
        size += 4; // generic_container_index
        if version <= 24.1 {
            size += 4 + 4 + 4 + 4 + 4; // method_index, invoker_index, delegate_wrapper_index, rgctx_start_index, rgctx_count
        }
        size += 4; // token
        size += 2 + 2 + 2 + 2; // flags, iflags, slot, parameter_count
        size
    }

    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>, version: f64) -> io::Result<Self> {
        let mut m = Self::default();
        m.name_index = r.read_u32()?;
        m.declaring_type = r.read_i32()?;
        m.return_type = r.read_i32()?;
        if version >= 31.0 {
            m.return_parameter_token = r.read_i32()?;
        }
        m.parameter_start = r.read_i32()?;
        if version <= 24.0 {
            m.custom_attribute_index = r.read_i32()?;
        }
        m.generic_container_index = r.read_i32()?;
        if version <= 24.1 {
            m.method_index = r.read_i32()?;
            m.invoker_index = r.read_i32()?;
            m.delegate_wrapper_index = r.read_i32()?;
            m.rgctx_start_index = r.read_i32()?;
            m.rgctx_count = r.read_i32()?;
        }
        m.token = r.read_u32()?;
        m.flags = r.read_u16()?;
        m.iflags = r.read_u16()?;
        m.slot = r.read_u16()?;
        m.parameter_count = r.read_u16()?;
        Ok(m)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Il2CppParameterDefinition {
    pub name_index: u32,
    pub token: u32,
    pub custom_attribute_index: i32,
    pub type_index: i32,
}

impl Il2CppParameterDefinition {
    pub fn size(version: f64) -> usize {
        let mut size = 4 + 4; // name_index, token
        if version <= 24.0 {
            size += 4; // custom_attribute_index
        }
        size += 4; // type_index
        size
    }

    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>, version: f64) -> io::Result<Self> {
        let name_index = r.read_u32()?;
        let token = r.read_u32()?;
        let mut custom_attribute_index = 0;
        if version <= 24.0 {
            custom_attribute_index = r.read_i32()?;
        }
        let type_index = r.read_i32()?;
        Ok(Self {
            name_index,
            token,
            custom_attribute_index,
            type_index,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Il2CppFieldDefinition {
    pub name_index: u32,
    pub type_index: i32,
    pub custom_attribute_index: i32,
    pub token: u32,
}

impl Il2CppFieldDefinition {
    pub fn size(version: f64) -> usize {
        let mut size = 4 + 4; // name_index, type_index
        if version <= 24.0 {
            size += 4; // custom_attribute_index
        }
        if version >= 19.0 {
            size += 4; // token
        }
        size
    }

    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>, version: f64) -> io::Result<Self> {
        let name_index = r.read_u32()?;
        let type_index = r.read_i32()?;
        let mut custom_attribute_index = 0;
        if version <= 24.0 {
            custom_attribute_index = r.read_i32()?;
        }
        let mut token = 0;
        if version >= 19.0 {
            token = r.read_u32()?;
        }
        Ok(Self {
            name_index,
            type_index,
            custom_attribute_index,
            token,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Il2CppFieldDefaultValue {
    pub field_index: i32,
    pub type_index: i32,
    pub data_index: i32,
}

impl Il2CppFieldDefaultValue {
    pub fn size(_version: f64) -> usize {
        12
    }

    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>, _version: f64) -> io::Result<Self> {
        Ok(Self {
            field_index: r.read_i32()?,
            type_index: r.read_i32()?,
            data_index: r.read_i32()?,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Il2CppPropertyDefinition {
    pub name_index: u32,
    pub get: i32,
    pub set: i32,
    pub attrs: u32,
    pub custom_attribute_index: i32,
    pub token: u32,
}

impl Il2CppPropertyDefinition {
    pub fn size(version: f64) -> usize {
        let mut size = 4 + 4 + 4 + 4; // name_index, get, set, attrs
        if version <= 24.0 {
            size += 4; // custom_attribute_index
        }
        if version >= 19.0 {
            size += 4; // token
        }
        size
    }

    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>, version: f64) -> io::Result<Self> {
        let name_index = r.read_u32()?;
        let get = r.read_i32()?;
        let set = r.read_i32()?;
        let attrs = r.read_u32()?;
        let mut custom_attribute_index = 0;
        if version <= 24.0 {
            custom_attribute_index = r.read_i32()?;
        }
        let mut token = 0;
        if version >= 19.0 {
            token = r.read_u32()?;
        }
        Ok(Self {
            name_index,
            get,
            set,
            attrs,
            custom_attribute_index,
            token,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Il2CppCustomAttributeTypeRange {
    pub token: u32,
    pub start: i32,
    pub count: i32,
}

impl Il2CppCustomAttributeTypeRange {
    pub fn size(version: f64) -> usize {
        let mut size = 0;
        if version >= 24.1 {
            size += 4; // token
        }
        size += 4 + 4; // start, count
        size
    }

    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>, version: f64) -> io::Result<Self> {
        let mut token = 0;
        if version >= 24.1 {
            token = r.read_u32()?;
        }
        let start = r.read_i32()?;
        let count = r.read_i32()?;
        Ok(Self {
            token,
            start,
            count,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Il2CppMetadataUsageList {
    pub start: u32,
    pub count: u32,
}

impl Il2CppMetadataUsageList {
    pub fn size(_version: f64) -> usize {
        8
    }

    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>, _version: f64) -> io::Result<Self> {
        Ok(Self {
            start: r.read_u32()?,
            count: r.read_u32()?,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Il2CppMetadataUsagePair {
    pub destination_index: u32,
    pub encoded_source_index: u32,
}

impl Il2CppMetadataUsagePair {
    pub fn size(_version: f64) -> usize {
        8
    }

    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>, _version: f64) -> io::Result<Self> {
        Ok(Self {
            destination_index: r.read_u32()?,
            encoded_source_index: r.read_u32()?,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Il2CppStringLiteral {
    pub length: u32,
    pub data_index: i32,
}

impl Il2CppStringLiteral {
    pub fn size(_version: f64) -> usize {
        8
    }

    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>, _version: f64) -> io::Result<Self> {
        Ok(Self {
            length: r.read_u32()?,
            data_index: r.read_i32()?,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Il2CppParameterDefaultValue {
    pub parameter_index: i32,
    pub type_index: i32,
    pub data_index: i32,
}

impl Il2CppParameterDefaultValue {
    pub fn size(_version: f64) -> usize {
        12
    }

    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>, _version: f64) -> io::Result<Self> {
        Ok(Self {
            parameter_index: r.read_i32()?,
            type_index: r.read_i32()?,
            data_index: r.read_i32()?,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Il2CppEventDefinition {
    pub name_index: u32,
    pub type_index: i32,
    pub add: i32,
    pub remove: i32,
    pub raise: i32,
    pub custom_attribute_index: i32,
    pub token: u32,
}

impl Il2CppEventDefinition {
    pub fn size(version: f64) -> usize {
        let mut size = 4 + 4 + 4 + 4 + 4; // name_index, type_index, add, remove, raise
        if version <= 24.0 {
            size += 4; // custom_attribute_index
        }
        if version >= 19.0 {
            size += 4; // token
        }
        size
    }

    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>, version: f64) -> io::Result<Self> {
        let name_index = r.read_u32()?;
        let type_index = r.read_i32()?;
        let add = r.read_i32()?;
        let remove = r.read_i32()?;
        let raise = r.read_i32()?;
        let mut custom_attribute_index = 0;
        if version <= 24.0 {
            custom_attribute_index = r.read_i32()?;
        }
        let mut token = 0;
        if version >= 19.0 {
            token = r.read_u32()?;
        }
        Ok(Self {
            name_index,
            type_index,
            add,
            remove,
            raise,
            custom_attribute_index,
            token,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Il2CppGenericContainer {
    pub owner_index: i32,
    pub type_argc: i32,
    pub is_method: i32,
    pub generic_parameter_start: i32,
}

impl Il2CppGenericContainer {
    pub fn size(_version: f64) -> usize {
        16
    }

    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>, _version: f64) -> io::Result<Self> {
        Ok(Self {
            owner_index: r.read_i32()?,
            type_argc: r.read_i32()?,
            is_method: r.read_i32()?,
            generic_parameter_start: r.read_i32()?,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Il2CppFieldRef {
    pub type_index: i32,
    pub field_index: i32,
}

impl Il2CppFieldRef {
    pub fn size(_version: f64) -> usize {
        8
    }

    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>, _version: f64) -> io::Result<Self> {
        Ok(Self {
            type_index: r.read_i32()?,
            field_index: r.read_i32()?,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Il2CppGenericParameter {
    pub owner_index: i32,
    pub name_index: u32,
    pub constraints_start: i16,
    pub constraints_count: i16,
    pub num: u16,
    pub flags: u16,
}

impl Il2CppGenericParameter {
    pub fn size(_version: f64) -> usize {
        4 + 4 + 2 + 2 + 2 + 2
    }

    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>, _version: f64) -> io::Result<Self> {
        Ok(Self {
            owner_index: r.read_i32()?,
            name_index: r.read_u32()?,
            constraints_start: r.read_i16()?,
            constraints_count: r.read_i16()?,
            num: r.read_u16()?,
            flags: r.read_u16()?,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Il2CppRGCTXDefinitionData {
    pub rgctx_data_dummy: i32,
}

impl Il2CppRGCTXDefinitionData {
    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>) -> io::Result<Self> {
        Ok(Self {
            rgctx_data_dummy: r.read_i32()?,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Il2CppRGCTXDefinition {
    pub type_pre29: i32,
    pub type_post29: u64,
    pub data: Il2CppRGCTXDefinitionData,
    pub data_post27_2: u64,
}

impl Il2CppRGCTXDefinition {
    pub fn size(version: f64) -> usize {
        let mut size = 0;
        if version <= 27.1 {
            size += 4; // type_pre29
        } else {
            size += 8; // type_post29
        }
        if version <= 27.1 {
            size += 4; // data
        } else {
            size += 8; // data_post27_2
        }
        size
    }

    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>, version: f64) -> io::Result<Self> {
        let mut rg = Self::default();
        if version <= 27.1 {
            rg.type_pre29 = r.read_i32()?;
        } else {
            rg.type_post29 = r.read_u64()?;
        }
        if version <= 27.1 {
            rg.data = Il2CppRGCTXDefinitionData::decode(r)?;
        } else {
            rg.data_post27_2 = r.read_u64()?;
        }
        Ok(rg)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Il2CppCustomAttributeDataRange {
    pub token: u32,
    pub start_offset: u32,
}

impl Il2CppCustomAttributeDataRange {
    pub fn size(_version: f64) -> usize {
        8
    }

    pub fn decode<R: Read + Seek>(r: &mut BinaryReader<R>, _version: f64) -> io::Result<Self> {
        Ok(Self {
            token: r.read_u32()?,
            start_offset: r.read_u32()?,
        })
    }
}

pub struct Metadata {
    pub version: f64,
    pub header: Il2CppGlobalMetadataHeader,
    pub image_defs: Vec<Il2CppImageDefinition>,
    pub assembly_defs: Vec<Il2CppAssemblyDefinition>,
    pub type_defs: Vec<Il2CppTypeDefinition>,
    pub method_defs: Vec<Il2CppMethodDefinition>,
    pub parameter_defs: Vec<Il2CppParameterDefinition>,
    pub field_defs: Vec<Il2CppFieldDefinition>,
    pub field_default_values: Vec<Il2CppFieldDefaultValue>,
    pub parameter_default_values: Vec<Il2CppParameterDefaultValue>,
    pub property_defs: Vec<Il2CppPropertyDefinition>,
    pub interface_indices: Vec<i32>,
    pub nested_type_indices: Vec<i32>,
    pub event_defs: Vec<Il2CppEventDefinition>,
    pub generic_containers: Vec<Il2CppGenericContainer>,
    pub generic_parameters: Vec<Il2CppGenericParameter>,
    pub constraint_indices: Vec<i32>,
    pub vtable_methods: Vec<u32>,
    pub string_literals: Vec<Il2CppStringLiteral>,
    pub field_refs: Vec<Il2CppFieldRef>,
    pub metadata_usage_lists: Vec<Il2CppMetadataUsageList>,
    pub metadata_usage_pairs: Vec<Il2CppMetadataUsagePair>,
    pub attribute_type_ranges: Vec<Il2CppCustomAttributeTypeRange>,
    pub attribute_types: Vec<i32>,
    pub attribute_data_ranges: Vec<Il2CppCustomAttributeDataRange>,
    pub rgctx_entries: Vec<Il2CppRGCTXDefinition>,

    // Decoded indices/dictionaries
    pub field_default_values_dic: HashMap<i32, Il2CppFieldDefaultValue>,
    pub parameter_default_values_dic: HashMap<i32, Il2CppParameterDefaultValue>,
    pub string_cache: RefCell<HashMap<u32, String>>,
    pub attribute_type_ranges_dic: HashMap<usize, HashMap<u32, i32>>, // image_index -> token -> index
    pub metadata_usage_dic: HashMap<u32, BTreeMap<u32, u32>>, // usage_type -> decodedIndex -> destinationIndex
    pub metadata_usages_count: usize,
    pub raw_bytes: Vec<u8>,
}

impl Metadata {
    pub fn load(meta_bytes: Vec<u8>) -> io::Result<Self> {
        let mut r = BinaryReader::new(std::io::Cursor::new(&meta_bytes), false, Endianness::Little);
        let sanity = r.read_u32()?;
        if sanity != 0xFAB11BAF {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "ERROR: Metadata file supplied is not a valid metadata file.",
            ));
        }
        let raw_version = r.read_i32()?;
        if !(16..=1000).contains(&raw_version) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "ERROR: Metadata file supplied is not a valid metadata file.",
            ));
        }
        if !(16..=31).contains(&raw_version) {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!(
                    "ERROR: Metadata file supplied is not a supported version[{}].",
                    raw_version
                ),
            ));
        }

        let mut version = raw_version as f64;
        let mut header = r
            .seek(0)
            .and_then(|_| Il2CppGlobalMetadataHeader::decode(&mut r, version))?;

        if raw_version == 24 {
            if header.string_literal_offset == 264 {
                version = 24.2;
                header = r
                    .seek(0)
                    .and_then(|_| Il2CppGlobalMetadataHeader::decode(&mut r, version))?;
            } else {
                let images = Self::read_class_array(
                    &mut r,
                    version,
                    header.images_offset,
                    header.images_size,
                )?;
                if images
                    .iter()
                    .any(|img: &Il2CppImageDefinition| img.token != 1)
                {
                    version = 24.1;
                }
            }
        }

        let mut image_defs = Self::read_class_array::<_, Il2CppImageDefinition>(
            &mut r,
            version,
            header.images_offset,
            header.images_size,
        )?;

        if version == 24.2 && (header.assemblies_size as usize / 68) < image_defs.len() {
            version = 24.4;
        }

        let mut v241_plus = false;
        if version == 24.1 && (header.assemblies_size as usize / 64) == image_defs.len() {
            v241_plus = true;
        }
        if v241_plus {
            version = 24.4;
        }

        let assembly_defs = Self::read_class_array::<_, Il2CppAssemblyDefinition>(
            &mut r,
            version,
            header.assemblies_offset,
            header.assemblies_size,
        )?;
        if v241_plus {
            version = 24.1;
        }

        let type_defs = Self::read_class_array::<_, Il2CppTypeDefinition>(
            &mut r,
            version,
            header.type_definitions_offset,
            header.type_definitions_size,
        )?;
        let method_defs = Self::read_class_array::<_, Il2CppMethodDefinition>(
            &mut r,
            version,
            header.methods_offset,
            header.methods_size,
        )?;
        let parameter_defs = Self::read_class_array::<_, Il2CppParameterDefinition>(
            &mut r,
            version,
            header.parameters_offset,
            header.parameters_size,
        )?;
        let field_defs = Self::read_class_array::<_, Il2CppFieldDefinition>(
            &mut r,
            version,
            header.fields_offset,
            header.fields_size,
        )?;
        let field_default_values = Self::read_class_array::<_, Il2CppFieldDefaultValue>(
            &mut r,
            version,
            header.field_default_values_offset,
            header.field_default_values_size,
        )?;
        let parameter_default_values = Self::read_class_array::<_, Il2CppParameterDefaultValue>(
            &mut r,
            version,
            header.parameter_default_values_offset,
            header.parameter_default_values_size,
        )?;
        let property_defs = Self::read_class_array::<_, Il2CppPropertyDefinition>(
            &mut r,
            version,
            header.properties_offset,
            header.properties_size,
        )?;

        let interface_indices = Self::read_primitive_array::<_, i32>(
            &mut r,
            header.interfaces_offset,
            header.interfaces_size,
        )?;
        let nested_type_indices = Self::read_primitive_array::<_, i32>(
            &mut r,
            header.nested_types_offset,
            header.nested_types_size,
        )?;
        let event_defs = Self::read_class_array::<_, Il2CppEventDefinition>(
            &mut r,
            version,
            header.events_offset,
            header.events_size,
        )?;
        let generic_containers = Self::read_class_array::<_, Il2CppGenericContainer>(
            &mut r,
            version,
            header.generic_containers_offset,
            header.generic_containers_size,
        )?;
        let generic_parameters = Self::read_class_array::<_, Il2CppGenericParameter>(
            &mut r,
            version,
            header.generic_parameters_offset,
            header.generic_parameters_size,
        )?;
        let constraint_indices = Self::read_primitive_array::<_, i32>(
            &mut r,
            header.generic_parameter_constraints_offset,
            header.generic_parameter_constraints_size,
        )?;
        let vtable_methods = Self::read_primitive_array::<_, u32>(
            &mut r,
            header.vtable_methods_offset,
            header.vtable_methods_size,
        )?;
        let string_literals = Self::read_class_array::<_, Il2CppStringLiteral>(
            &mut r,
            version,
            header.string_literal_offset,
            header.string_literal_size,
        )?;

        let mut field_refs = Vec::new();
        let mut metadata_usage_lists = Vec::new();
        let mut metadata_usage_pairs = Vec::new();
        let mut attribute_type_ranges = Vec::new();
        let mut attribute_types = Vec::new();
        let mut attribute_data_ranges = Vec::new();
        let mut rgctx_entries = Vec::new();

        if version > 16.0 {
            field_refs = Self::read_class_array::<_, Il2CppFieldRef>(
                &mut r,
                version,
                header.field_refs_offset,
                header.field_refs_size,
            )?;
            if version < 27.0 {
                metadata_usage_lists = Self::read_class_array::<_, Il2CppMetadataUsageList>(
                    &mut r,
                    version,
                    header.metadata_usage_lists_offset,
                    header.metadata_usage_lists_count * 8,
                )?;
                metadata_usage_pairs = Self::read_class_array::<_, Il2CppMetadataUsagePair>(
                    &mut r,
                    version,
                    header.metadata_usage_pairs_offset,
                    header.metadata_usage_pairs_count * 8,
                )?;
            }
        }

        if version > 20.0 && version < 29.0 {
            attribute_type_ranges = Self::read_class_array::<_, Il2CppCustomAttributeTypeRange>(
                &mut r,
                version,
                header.attributes_info_offset,
                header.attributes_info_count * (if version >= 24.1 { 12 } else { 8 }),
            )?;
            attribute_types = Self::read_primitive_array::<_, i32>(
                &mut r,
                header.attribute_types_offset,
                header.attribute_types_count,
            )?;
        }
        if version >= 29.0 {
            attribute_data_ranges = Self::read_class_array::<_, Il2CppCustomAttributeDataRange>(
                &mut r,
                version,
                header.attribute_data_range_offset,
                header.attribute_data_range_size,
            )?;
        }
        if version > 24.1 {
            // We need to re-read image_defs since version changed
            r.seek(header.images_offset as u64)?;
            image_defs = Self::read_class_array::<_, Il2CppImageDefinition>(
                &mut r,
                version,
                header.images_offset,
                header.images_size,
            )?;
        }
        if version <= 24.1 && header.rgctx_entries_offset > 0 {
            rgctx_entries = Self::read_class_array::<_, Il2CppRGCTXDefinition>(
                &mut r,
                version,
                header.rgctx_entries_offset,
                header.rgctx_entries_count * 8,
            )?;
        }

        // Process maps
        let field_default_values_dic = field_default_values
            .iter()
            .cloned()
            .map(|x| (x.field_index, x))
            .collect();
        let parameter_default_values_dic = parameter_default_values
            .iter()
            .cloned()
            .map(|x| (x.parameter_index, x))
            .collect();

        let mut attribute_type_ranges_dic = HashMap::new();
        if version > 24.0 {
            for (img_idx, image_def) in image_defs.iter().enumerate() {
                let mut dic = HashMap::new();
                let end =
                    image_def.custom_attribute_start + image_def.custom_attribute_count as i32;
                for i in image_def.custom_attribute_start..end {
                    let idx = i as usize;
                    let token = if version >= 29.0 {
                        attribute_data_ranges[idx].token
                    } else {
                        attribute_type_ranges[idx].token
                    };
                    dic.insert(token, i);
                }
                attribute_type_ranges_dic.insert(img_idx, dic);
            }
        }

        let mut metadata_usage_dic = HashMap::new();
        let mut metadata_usages_count = 0;
        if version > 16.0 && version < 27.0 {
            for i in 1..=6 {
                metadata_usage_dic.insert(i, BTreeMap::new());
            }
            for list in &metadata_usage_lists {
                for i in 0..list.count {
                    let offset = (list.start + i) as usize;
                    if offset < metadata_usage_pairs.len() {
                        let pair = &metadata_usage_pairs[offset];
                        let usage = pair.encoded_source_index >> 29;
                        if (1..=6).contains(&usage) {
                            let decoded_index = pair.encoded_source_index & 0x1FFFFFFF;
                            if let Some(map) = metadata_usage_dic.get_mut(&usage) {
                                map.insert(decoded_index, pair.destination_index);
                            }
                        }
                    }
                }
            }
            metadata_usages_count = metadata_usage_pairs.len();
        }

        Ok(Self {
            version,
            header,
            image_defs,
            assembly_defs,
            type_defs,
            method_defs,
            parameter_defs,
            field_defs,
            field_default_values,
            parameter_default_values,
            property_defs,
            interface_indices,
            nested_type_indices,
            event_defs,
            generic_containers,
            generic_parameters,
            constraint_indices,
            vtable_methods,
            string_literals,
            field_refs,
            metadata_usage_lists,
            metadata_usage_pairs,
            attribute_type_ranges,
            attribute_types,
            attribute_data_ranges,
            rgctx_entries,

            field_default_values_dic,
            parameter_default_values_dic,
            string_cache: RefCell::new(HashMap::new()),
            attribute_type_ranges_dic,
            metadata_usage_dic,
            metadata_usages_count,
            raw_bytes: meta_bytes,
        })
    }

    fn read_class_array<R: Read + Seek, T>(
        r: &mut BinaryReader<R>,
        version: f64,
        offset: u32,
        size: i32,
    ) -> io::Result<Vec<T>>
    where
        T: Default + FnDecode<R>,
    {
        if size <= 0 {
            return Ok(Vec::new());
        }
        let size_of_t = T::size(version);
        let count = size as usize / size_of_t;
        r.seek(offset as u64)?;
        let mut vec = Vec::with_capacity(count);
        for _ in 0..count {
            vec.push(T::decode(r, version)?);
        }
        Ok(vec)
    }

    fn read_primitive_array<R: Read + Seek, T: PrimitiveRead>(
        r: &mut BinaryReader<R>,
        offset: u32,
        size: i32,
    ) -> io::Result<Vec<T>> {
        if size <= 0 {
            return Ok(Vec::new());
        }
        let count = size as usize / std::mem::size_of::<T>();
        r.seek(offset as u64)?;
        let mut vec = Vec::with_capacity(count);
        for _ in 0..count {
            vec.push(T::read_from(r)?);
        }
        Ok(vec)
    }

    pub fn get_string_from_index(&self, index: u32) -> String {
        let mut cache = self.string_cache.borrow_mut();
        if let Some(s) = cache.get(&index) {
            return s.clone();
        }
        let addr = (self.header.string_offset as u64 + index as u64) as usize;
        if addr < self.raw_bytes.len() {
            let mut end = addr;
            while end < self.raw_bytes.len() && self.raw_bytes[end] != 0 {
                end += 1;
            }
            if let Ok(s) = String::from_utf8(self.raw_bytes[addr..end].to_vec()) {
                cache.insert(index, s.clone());
                return s;
            }
        }
        String::new()
    }

    pub fn get_string_literal_from_index(&self, index: u32) -> io::Result<String> {
        let string_literal = &self.string_literals[index as usize];
        let addr = (self.header.string_literal_data_offset as u64
            + string_literal.data_index as u64) as usize;
        let len = string_literal.length as usize;
        if addr + len <= self.raw_bytes.len() {
            String::from_utf8(self.raw_bytes[addr..addr + len].to_vec())
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        } else {
            Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Failed to read string literal bytes",
            ))
        }
    }

    pub fn get_custom_attribute_index(
        &self,
        image_index: usize,
        custom_attribute_index: i32,
        token: u32,
    ) -> i32 {
        if self.version > 24.0 {
            if let Some(dic) = self.attribute_type_ranges_dic.get(&image_index)
                && let Some(&index) = dic.get(&token)
            {
                return index;
            }
            -1
        } else {
            custom_attribute_index
        }
    }
}

trait FnDecode<R: Read + Seek> {
    fn size(version: f64) -> usize;
    fn decode(r: &mut BinaryReader<R>, version: f64) -> io::Result<Self>
    where
        Self: Sized;
}

macro_rules! impl_fn_decode {
    ($t:ty) => {
        impl<R: Read + Seek> FnDecode<R> for $t {
            fn size(version: f64) -> usize {
                Self::size(version)
            }
            fn decode(r: &mut BinaryReader<R>, version: f64) -> io::Result<Self> {
                Self::decode(r, version)
            }
        }
    };
}

impl_fn_decode!(Il2CppImageDefinition);
impl_fn_decode!(Il2CppAssemblyDefinition);
impl_fn_decode!(Il2CppTypeDefinition);
impl_fn_decode!(Il2CppMethodDefinition);
impl_fn_decode!(Il2CppParameterDefinition);
impl_fn_decode!(Il2CppFieldDefinition);
impl_fn_decode!(Il2CppFieldDefaultValue);
impl_fn_decode!(Il2CppPropertyDefinition);
impl_fn_decode!(Il2CppCustomAttributeTypeRange);
impl_fn_decode!(Il2CppMetadataUsageList);
impl_fn_decode!(Il2CppMetadataUsagePair);
impl_fn_decode!(Il2CppStringLiteral);
impl_fn_decode!(Il2CppParameterDefaultValue);
impl_fn_decode!(Il2CppEventDefinition);
impl_fn_decode!(Il2CppGenericContainer);
impl_fn_decode!(Il2CppFieldRef);
impl_fn_decode!(Il2CppGenericParameter);
impl_fn_decode!(Il2CppRGCTXDefinition);
impl_fn_decode!(Il2CppCustomAttributeDataRange);

trait PrimitiveRead {
    fn read_from<R: Read + Seek>(r: &mut BinaryReader<R>) -> io::Result<Self>
    where
        Self: Sized;
}

impl PrimitiveRead for i32 {
    fn read_from<R: Read + Seek>(r: &mut BinaryReader<R>) -> io::Result<Self> {
        r.read_i32()
    }
}

impl PrimitiveRead for u32 {
    fn read_from<R: Read + Seek>(r: &mut BinaryReader<R>) -> io::Result<Self> {
        r.read_u32()
    }
}
