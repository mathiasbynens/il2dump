#![allow(dead_code, unused_variables, unused_imports, unused_mut)]
use crate::binary_reader::Endianness;
use crate::config::Config;
use crate::il2cpp_binary_structures::Il2CppTypeEnum;
use crate::il2cpp_executor::Il2CppExecutor;
use crate::metadata::{Il2CppMethodDefinition, Il2CppRGCTXDefinition, Il2CppTypeDefinition};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{self, BufWriter, Read, Seek, Write};
use std::path::Path;

// Constant flags
const FIELD_ATTRIBUTE_FIELD_ACCESS_MASK: u32 = 0x0007;
const FIELD_ATTRIBUTE_PRIVATE: u32 = 0x0001;
const FIELD_ATTRIBUTE_FAMILY: u32 = 0x0004;
const FIELD_ATTRIBUTE_ASSEMBLY: u32 = 0x0003;
const FIELD_ATTRIBUTE_PUBLIC: u32 = 0x0006;
const FIELD_ATTRIBUTE_STATIC: u32 = 0x0010;
const FIELD_ATTRIBUTE_LITERAL: u32 = 0x0040;
const FIELD_ATTRIBUTE_INIT_ONLY: u32 = 0x0020;

const METHOD_ATTRIBUTE_MEMBER_ACCESS_MASK: u16 = 0x0007;
const METHOD_ATTRIBUTE_PRIVATE: u16 = 0x0001;
const METHOD_ATTRIBUTE_FAMILY: u16 = 0x0004;
const METHOD_ATTRIBUTE_ASSEM: u16 = 0x0003;
const METHOD_ATTRIBUTE_PUBLIC: u16 = 0x0006;
const METHOD_ATTRIBUTE_STATIC: u16 = 0x0010;
const METHOD_ATTRIBUTE_FINAL: u16 = 0x0020;
const METHOD_ATTRIBUTE_VIRTUAL: u16 = 0x0040;
const METHOD_ATTRIBUTE_ABSTRACT: u16 = 0x0400;

const TYPE_ATTRIBUTE_VISIBILITY_MASK: u32 = 0x00000007;
const TYPE_ATTRIBUTE_PUBLIC: u32 = 0x00000001;
const TYPE_ATTRIBUTE_NESTED_PUBLIC: u32 = 0x00000002;
const TYPE_ATTRIBUTE_NESTED_PRIVATE: u32 = 0x00000003;
const TYPE_ATTRIBUTE_NESTED_FAMILY: u32 = 0x00000004;
const TYPE_ATTRIBUTE_NESTED_ASSEMBLY: u32 = 0x00000005;
const TYPE_ATTRIBUTE_NESTED_FAM_AND_ASSEM: u32 = 0x00000006;
const TYPE_ATTRIBUTE_NESTED_FAM_OR_ASSEM: u32 = 0x00000007;
const TYPE_ATTRIBUTE_INTERFACE: u32 = 0x00000020;
const TYPE_ATTRIBUTE_ABSTRACT: u32 = 0x00000080;
const TYPE_ATTRIBUTE_SEALED: u32 = 0x00000100;
const TYPE_ATTRIBUTE_SERIALIZABLE: u32 = 0x00002000;

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct ScriptJson {
    script_method: Vec<ScriptMethod>,
    script_string: Vec<ScriptString>,
    script_metadata: Vec<ScriptMetadata>,
    script_metadata_method: Vec<ScriptMetadataMethod>,
    addresses: Vec<u64>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct ScriptMethod {
    address: u64,
    name: String,
    signature: String,
    type_signature: String,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct ScriptString {
    address: u64,
    value: String,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct ScriptMetadata {
    address: u64,
    name: String,
    signature: String,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct ScriptMetadataMethod {
    address: u64,
    name: String,
    method_address: u64,
}

pub struct Decompiler<'a> {
    executor: &'a Il2CppExecutor,
}

impl<'a> Decompiler<'a> {
    pub fn new(executor: &'a Il2CppExecutor) -> Self {
        Self { executor }
    }

    pub fn decompile_to_memory(&self, config: &Config) -> io::Result<(String, String)> {
        let mut w_bytes = Vec::new();
        let mut w = BufWriter::new(&mut w_bytes);

        let metadata = &self.executor.metadata;
        // Write the metadata info for each image.
        for (image_index, image_def) in metadata.image_defs.iter().enumerate() {
            let name = metadata.get_string_from_index(image_def.name_index);
            writeln!(
                w,
                "// Image {}: {} - {}",
                image_index, name, image_def.type_start
            )?;
        }

        let mut script_methods = Vec::<ScriptMethod>::new();
        let mut script_strings = Vec::<ScriptString>::new();
        let mut script_metadata = Vec::<ScriptMetadata>::new();
        let mut script_metadata_method = Vec::<ScriptMetadataMethod>::new();

        // Write the metadata types.
        for (image_index, image_def) in metadata.image_defs.iter().enumerate() {
            let image_name = metadata.get_string_from_index(image_def.name_index);
            let type_end = image_def.type_start + image_def.type_count as i32;

            for type_def_idx in image_def.type_start..type_end {
                let type_def = &metadata.type_defs[type_def_idx as usize];
                let mut extends = Vec::new();

                if type_def.parent_index >= 0
                    && let Some(parent_ty) = self.executor.types.get(type_def.parent_index as usize)
                {
                    let parent_name = self.executor.get_type_name(parent_ty, false, false);
                    if !type_def.is_value_type() && !type_def.is_enum() && parent_name != "object" {
                        extends.push(parent_name);
                    }
                }

                if type_def.interfaces_count > 0 {
                    for i in 0..type_def.interfaces_count {
                        let idx = metadata.interface_indices
                            [type_def.interfaces_start as usize + i as usize];
                        if let Some(iface_ty) = self.executor.types.get(idx as usize) {
                            extends.push(self.executor.get_type_name(iface_ty, false, false));
                        }
                    }
                }

                let namespace = metadata.get_string_from_index(type_def.namespace_index);
                writeln!(w, "\n// Namespace: {}", namespace)?;

                if config.dump_attribute {
                    let attrs = self.get_custom_attributes(
                        image_index,
                        type_def.custom_attribute_index,
                        type_def.token,
                    );
                    for attr in attrs {
                        writeln!(w, "{}", attr)?;
                    }
                }

                if config.dump_attribute && (type_def.flags & TYPE_ATTRIBUTE_SERIALIZABLE) != 0 {
                    writeln!(w, "[Serializable]")?;
                }

                // Write class visibility.
                let visibility = type_def.flags & TYPE_ATTRIBUTE_VISIBILITY_MASK;
                match visibility {
                    TYPE_ATTRIBUTE_PUBLIC | TYPE_ATTRIBUTE_NESTED_PUBLIC => write!(w, "public ")?,
                    TYPE_ATTRIBUTE_NESTED_PRIVATE => write!(w, "private ")?,
                    TYPE_ATTRIBUTE_NESTED_FAMILY => write!(w, "protected ")?,
                    _ => write!(w, "internal ")?,
                }

                // Write class attributes.
                if (type_def.flags & TYPE_ATTRIBUTE_ABSTRACT) != 0
                    && (type_def.flags & TYPE_ATTRIBUTE_SEALED) != 0
                {
                    write!(w, "static ")?;
                } else if (type_def.flags & TYPE_ATTRIBUTE_INTERFACE) == 0
                    && (type_def.flags & TYPE_ATTRIBUTE_ABSTRACT) != 0
                {
                    write!(w, "abstract ")?;
                } else if !type_def.is_value_type()
                    && !type_def.is_enum()
                    && (type_def.flags & TYPE_ATTRIBUTE_SEALED) != 0
                {
                    write!(w, "sealed ")?;
                }

                // Write the class type kind (class, struct, enum, or interface).
                if (type_def.flags & TYPE_ATTRIBUTE_INTERFACE) != 0 {
                    write!(w, "interface ")?;
                } else if type_def.is_enum() {
                    write!(w, "enum ")?;
                } else if type_def.is_value_type() {
                    write!(w, "struct ")?;
                } else {
                    write!(w, "class ")?;
                }

                let type_name = self.executor.get_type_name_from_def(type_def, false, false);
                write!(w, "{}", type_name)?;

                if !extends.is_empty() {
                    write!(w, " : {}", extends.join(", "))?;
                }

                let is_empty = type_def.field_count == 0
                    && type_def.property_count == 0
                    && type_def.method_count == 0;
                if is_empty {
                    if config.dump_type_def_index {
                        writeln!(w, " // TypeDefIndex: {}", type_def_idx)?;
                    } else {
                        writeln!(w)?;
                    }
                    writeln!(w, "{{}}")?;
                    continue;
                }

                if config.dump_type_def_index {
                    writeln!(w, " // TypeDefIndex: {}", type_def_idx)?;
                } else {
                    writeln!(w)?;
                }
                write!(w, "{{")?;

                let implements_proto = self.implements_imessage(type_def);
                let proto_tags = if implements_proto {
                    self.trace_protobuf_tags(image_index, &image_name, type_def)
                } else {
                    HashMap::new()
                };

                // Write the fields of the type.
                if config.dump_field && type_def.field_count > 0 {
                    writeln!(w, "\n\t// Fields")?;
                    let field_end = type_def.field_start + type_def.field_count as i32;
                    for i in type_def.field_start..field_end {
                        let field_def = &metadata.field_defs[i as usize];
                        let field_type = &self.executor.types[field_def.type_index as usize];
                        let is_static = (field_type.attrs & FIELD_ATTRIBUTE_STATIC) != 0;
                        let is_const = (field_type.attrs & FIELD_ATTRIBUTE_LITERAL) != 0;
                        let is_readonly = (field_type.attrs & FIELD_ATTRIBUTE_INIT_ONLY) != 0;

                        if config.dump_attribute {
                            let attrs = self.get_custom_attributes(
                                image_index,
                                field_def.custom_attribute_index,
                                field_def.token,
                            );
                            for attr in attrs {
                                writeln!(w, "\t{}", attr)?;
                            }
                        }

                        write!(w, "\t")?;
                        let access = field_type.attrs & FIELD_ATTRIBUTE_FIELD_ACCESS_MASK;
                        match access {
                            FIELD_ATTRIBUTE_PRIVATE => write!(w, "private ")?,
                            FIELD_ATTRIBUTE_FAMILY => write!(w, "protected ")?,
                            FIELD_ATTRIBUTE_PUBLIC => write!(w, "public ")?,
                            _ => write!(w, "internal ")?,
                        }

                        if is_const {
                            write!(w, "const ")?;
                        } else {
                            if is_static {
                                write!(w, "static ")?;
                            }
                            if is_readonly {
                                write!(w, "readonly ")?;
                            }
                        }

                        let field_type_name = self.executor.get_type_name(field_type, false, false);
                        let field_name = metadata.get_string_from_index(field_def.name_index);
                        write!(w, "{} {}", field_type_name, field_name)?;

                        if let Some(fdv) = metadata.field_default_values_dic.get(&{ i })
                            && let Some(val) = self
                                .executor
                                .get_default_value(fdv.type_index, fdv.data_index)
                        {
                            write!(w, " = {}", val)?;
                        }
                        write!(w, ";")?;

                        if config.dump_field_offset && !is_const {
                            let offset = self.executor.get_field_offset_from_index(
                                type_def_idx as usize,
                                (i - type_def.field_start) as usize,
                                i as usize,
                                type_def.is_value_type(),
                                is_static,
                            );
                            write!(w, " // 0x{:X}", offset)?;
                            if let Some(tag) = proto_tags.get(&(offset as u32)) {
                                write!(w, " [ProtoTag: {}]", tag)?;
                            }
                        }
                        writeln!(w)?;
                    }
                }

                // Write the properties of the type.
                if config.dump_property && type_def.property_count > 0 {
                    writeln!(w, "\n\t// Properties")?;
                    let property_end = type_def.property_start + type_def.property_count as i32;
                    for i in type_def.property_start..property_end {
                        let property_def = &metadata.property_defs[i as usize];
                        if config.dump_attribute {
                            let attrs = self.get_custom_attributes(
                                image_index,
                                property_def.custom_attribute_index,
                                property_def.token,
                            );
                            for attr in attrs {
                                writeln!(w, "\t{}", attr)?;
                            }
                        }

                        write!(w, "\t")?;

                        if property_def.get >= 0 {
                            let method_def = &metadata.method_defs
                                [(type_def.method_start + property_def.get) as usize];
                            let access = method_def.flags & METHOD_ATTRIBUTE_MEMBER_ACCESS_MASK;
                            match access {
                                METHOD_ATTRIBUTE_PRIVATE => write!(w, "private ")?,
                                METHOD_ATTRIBUTE_FAMILY => write!(w, "protected ")?,
                                METHOD_ATTRIBUTE_PUBLIC => write!(w, "public ")?,
                                _ => write!(w, "internal ")?,
                            }
                            if (method_def.flags & METHOD_ATTRIBUTE_STATIC) != 0 {
                                write!(w, "static ")?;
                            }

                            let property_type =
                                &self.executor.types[method_def.return_type as usize];
                            let property_type_name =
                                self.executor.get_type_name(property_type, false, false);
                            let property_name =
                                metadata.get_string_from_index(property_def.name_index);
                            write!(w, "{} {} {{ ", property_type_name, property_name)?;
                        } else if property_def.set >= 0 {
                            let method_def = &metadata.method_defs
                                [(type_def.method_start + property_def.set) as usize];
                            let access = method_def.flags & METHOD_ATTRIBUTE_MEMBER_ACCESS_MASK;
                            match access {
                                METHOD_ATTRIBUTE_PRIVATE => write!(w, "private ")?,
                                METHOD_ATTRIBUTE_FAMILY => write!(w, "protected ")?,
                                METHOD_ATTRIBUTE_PUBLIC => write!(w, "public ")?,
                                _ => write!(w, "internal ")?,
                            }
                            if (method_def.flags & METHOD_ATTRIBUTE_STATIC) != 0 {
                                write!(w, "static ")?;
                            }

                            let param_def =
                                &metadata.parameter_defs[method_def.parameter_start as usize];
                            let property_type = &self.executor.types[param_def.type_index as usize];
                            let property_type_name =
                                self.executor.get_type_name(property_type, false, false);
                            let property_name =
                                metadata.get_string_from_index(property_def.name_index);
                            write!(w, "{} {} {{ ", property_type_name, property_name)?;
                        }

                        if property_def.get >= 0 {
                            write!(w, "get; ")?;
                        }
                        if property_def.set >= 0 {
                            write!(w, "set; ")?;
                        }
                        writeln!(w, "}}")?;
                    }
                }

                // Write the methods of the type.
                if config.dump_method && type_def.method_count > 0 {
                    writeln!(w, "\n\t// Methods")?;
                    let method_end = type_def.method_start + type_def.method_count as i32;
                    for i in type_def.method_start..method_end {
                        writeln!(w)?;
                        let method_def = &metadata.method_defs[i as usize];
                        if config.dump_attribute {
                            let attrs = self.get_custom_attributes(
                                image_index,
                                method_def.custom_attribute_index,
                                method_def.token,
                            );
                            for attr in attrs {
                                writeln!(w, "\t{}", attr)?;
                            }
                        }

                        // Resolve the virtual address pointer for the method first.
                        let method_ptr = if self.executor.metadata.version >= 24.2 {
                            if let Some(ptrs) = self
                                .executor
                                .code_gen_module_method_pointers
                                .get(&image_name)
                            {
                                let method_pointer_index = (method_def.token & 0x00FFFFFF) as usize;
                                if method_pointer_index > 0 && method_pointer_index <= ptrs.len() {
                                    ptrs[method_pointer_index - 1]
                                } else {
                                    0
                                }
                            } else {
                                0
                            }
                        } else {
                            let method_ptr_idx = method_def.method_index as usize;
                            self.executor
                                .method_pointers
                                .get(method_ptr_idx)
                                .cloned()
                                .unwrap_or(0)
                        };

                        let is_abstract = (method_def.flags & METHOD_ATTRIBUTE_ABSTRACT) != 0;

                        if config.dump_method_offset {
                            if !is_abstract && method_ptr > 0 {
                                let rva = self.executor.binary.get_rva(method_ptr);
                                write!(
                                    w,
                                    "\t// RVA: 0x{:X} Offset: 0x{:X} VA: 0x{:X}",
                                    rva,
                                    self.executor.binary.map_vatr(method_ptr).unwrap_or(0),
                                    method_ptr
                                )?;
                            } else {
                                write!(w, "\t// RVA: -1 Offset: -1")?;
                            }
                            if method_def.slot != 65535 {
                                write!(w, " Slot: {}", method_def.slot)?;
                            }
                            writeln!(w)?;
                        }

                        let mut method_name = metadata.get_string_from_index(method_def.name_index);
                        if method_def.generic_container_index >= 0
                            && let Some(gc) = metadata
                                .generic_containers
                                .get(method_def.generic_container_index as usize)
                        {
                            method_name = format!(
                                "{}{}",
                                method_name,
                                self.executor.get_generic_container_params(gc)
                            );
                        }

                        let return_type = &self.executor.types[method_def.return_type as usize];
                        let return_type_name =
                            self.executor.get_type_name(return_type, false, false);
                        let ref_str = if return_type.byref == 1 { "ref " } else { "" };

                        write!(w, "\t")?;
                        // Write method access modifiers.
                        let access = method_def.flags & METHOD_ATTRIBUTE_MEMBER_ACCESS_MASK;
                        match access {
                            METHOD_ATTRIBUTE_PRIVATE => write!(w, "private ")?,
                            METHOD_ATTRIBUTE_PUBLIC => write!(w, "public ")?,
                            METHOD_ATTRIBUTE_FAMILY => write!(w, "protected ")?,
                            METHOD_ATTRIBUTE_ASSEM | 0x0002 => write!(w, "internal ")?,
                            0x0005 => write!(w, "protected internal ")?,
                            _ => {}
                        }

                        if (method_def.flags & METHOD_ATTRIBUTE_STATIC) != 0 {
                            write!(w, "static ")?;
                        }

                        let vtable_layout = method_def.flags & 0x0100; // METHOD_ATTRIBUTE_VTABLE_LAYOUT_MASK
                        if is_abstract {
                            write!(w, "abstract ")?;
                            if vtable_layout == 0 {
                                // METHOD_ATTRIBUTE_REUSE_SLOT
                                write!(w, "override ")?;
                            }
                        } else if (method_def.flags & METHOD_ATTRIBUTE_FINAL) != 0 {
                            if vtable_layout == 0 {
                                write!(w, "sealed override ")?;
                            }
                        } else if (method_def.flags & METHOD_ATTRIBUTE_VIRTUAL) != 0 {
                            if vtable_layout == 0x0100 {
                                // METHOD_ATTRIBUTE_NEW_SLOT
                                write!(w, "virtual ")?;
                            } else {
                                write!(w, "override ")?;
                            }
                        }

                        // Format and write the method parameters.
                        let mut param_strs = Vec::new();
                        let mut param_types = Vec::new();
                        for p in 0..method_def.parameter_count {
                            let param_def = &metadata.parameter_defs
                                [method_def.parameter_start as usize + p as usize];
                            let param_name = metadata.get_string_from_index(param_def.name_index);
                            let param_type = &self.executor.types[param_def.type_index as usize];
                            let param_type_name =
                                self.executor.get_type_name(param_type, false, false);

                            let mut attr_str = String::new();
                            if config.dump_attribute {
                                let attrs = self.get_custom_attributes(
                                    image_index,
                                    param_def.custom_attribute_index,
                                    param_def.token,
                                );
                                if !attrs.is_empty() {
                                    attr_str = format!("{} ", attrs.join(" "));
                                }
                            }

                            let mut modifier = String::new();
                            if param_type.byref == 1 {
                                let is_in = (param_type.attrs & 0x0001) != 0;
                                let is_out = (param_type.attrs & 0x0002) != 0;
                                if is_out && !is_in {
                                    modifier = "out ".to_string();
                                } else if is_in && !is_out {
                                    modifier = "in ".to_string();
                                } else {
                                    modifier = "ref ".to_string();
                                }
                            } else {
                                if (param_type.attrs & 0x0001) != 0 {
                                    modifier += "[In] ";
                                }
                                if (param_type.attrs & 0x0002) != 0 {
                                    modifier += "[Out] ";
                                }
                            }

                            let mut default_val = String::new();
                            let param_idx = method_def.parameter_start + p as i32;
                            if let Some(pdv) = metadata.parameter_default_values_dic.get(&param_idx)
                                && pdv.data_index != -1
                                && let Some(val) = self
                                    .executor
                                    .get_default_value(pdv.type_index, pdv.data_index)
                            {
                                default_val = format!(" = {}", val);
                            }

                            param_strs.push(format!(
                                "{}{}{} {}{}",
                                attr_str, modifier, param_type_name, param_name, default_val
                            ));
                            param_types.push(param_type_name);
                        }

                        let is_interface = (type_def.flags & TYPE_ATTRIBUTE_INTERFACE) != 0;
                        let suffix = if is_abstract || is_interface {
                            ";"
                        } else {
                            " { }"
                        };

                        writeln!(
                            w,
                            "{}{} {}({}){}",
                            ref_str,
                            return_type_name,
                            method_name,
                            param_strs.join(", "),
                            suffix
                        )?;

                        // Add the method entry to the script JSON list.
                        if method_ptr > 0 {
                            let rva = self.executor.binary.get_rva(method_ptr);
                            let full_method_name = format!("{}$${}", type_name, method_name);
                            let signature =
                                format!("{}({})", return_type_name, param_types.join(", "));
                            script_methods.push(ScriptMethod {
                                address: rva,
                                name: full_method_name,
                                signature,
                                type_signature: self
                                    .get_method_type_signature(method_def, type_def),
                            });
                        }

                        if let Some(specs) =
                            self.executor.method_definition_method_specs.get(&{ i })
                        {
                            writeln!(w, "\t/* GenericInstMethod :")?;
                            let mut groups: BTreeMap<
                                u64,
                                Vec<&crate::il2cpp_binary_structures::Il2CppMethodSpec>,
                            > = BTreeMap::new();
                            for &(spec_idx, ref spec) in specs {
                                let ptr = self
                                    .executor
                                    .method_spec_generic_method_pointers
                                    .get(&spec_idx)
                                    .cloned()
                                    .unwrap_or(0);
                                groups.entry(ptr).or_default().push(spec);
                            }
                            for (ptr, spec_list) in groups {
                                writeln!(w, "\t|")?;
                                if ptr > 0 {
                                    let rva = self.executor.binary.get_rva(ptr);
                                    let offset = self.executor.binary.map_vatr(ptr).unwrap_or(0);
                                    writeln!(
                                        w,
                                        "\t|-RVA: 0x{:X} Offset: 0x{:X} VA: 0x{:X}",
                                        rva, offset, ptr
                                    )?;
                                } else {
                                    writeln!(w, "\t|-RVA: -1 Offset: -1")?;
                                }
                                for spec in spec_list {
                                    let (spec_type_name, spec_method_name) =
                                        self.executor.get_method_spec_name(spec, false);
                                    writeln!(w, "\t|-{}.{}", spec_type_name, spec_method_name)?;

                                    if ptr > 0 {
                                        let rva = self.executor.binary.get_rva(ptr);
                                        let (spec_type_name_ns, spec_method_name_ns) =
                                            self.executor.get_method_spec_name(spec, true);
                                        let method_full_name = format!(
                                            "{}$${}",
                                            spec_type_name_ns, spec_method_name_ns
                                        );
                                        let signature = format!(
                                            "{} {}(...);",
                                            return_type_name,
                                            method_full_name.replace("$$", "_")
                                        );

                                        script_methods.push(ScriptMethod {
                                            address: rva,
                                            name: method_full_name,
                                            signature,
                                            type_signature: self
                                                .get_method_type_signature(method_def, type_def),
                                        });
                                    }
                                }
                            }
                            writeln!(w, "\t*/")?;
                        }
                    }
                }

                writeln!(w, "}}")?;
            }
        }
        w.flush()?;

        // Collect and sort all method addresses.
        let mut ordered_pointers = Vec::new();
        for ptrs in self.executor.code_gen_module_method_pointers.values() {
            ordered_pointers.extend(ptrs.iter().cloned());
        }
        ordered_pointers.extend(self.executor.method_pointers.iter().cloned());
        ordered_pointers.extend(self.executor.generic_method_pointers.iter().cloned());
        ordered_pointers.extend(self.executor.invoker_pointers.iter().cloned());
        if self.executor.metadata.version < 29.0 {
            ordered_pointers.extend(self.executor.custom_attribute_generators.iter().cloned());
        }
        if self.executor.metadata.version >= 22.0 {
            ordered_pointers.extend(self.executor.reverse_pinvoke_wrappers.iter().cloned());
            ordered_pointers.extend(
                self.executor
                    .unresolved_virtual_call_pointers
                    .iter()
                    .cloned(),
            );
        }
        ordered_pointers.sort_unstable();
        ordered_pointers.dedup();
        ordered_pointers.retain(|&x| x != 0);

        let addresses: Vec<u64> = ordered_pointers
            .iter()
            .map(|&x| self.executor.binary.get_rva(x))
            .collect();

        script_strings.clear();
        script_metadata.clear();
        script_metadata_method.clear();

        if self.executor.metadata.version >= 27.0 {
            let metadata = &self.executor.metadata;
            let binary = &self.executor.binary;

            for sec in &binary.data_sections {
                let offset = sec.offset as usize;
                let offset_end = sec.offset_end as usize;
                let ptr_size = if binary.is_32bit { 4 } else { 8 };
                let end = offset_end.min(binary.bytes.len()) - ptr_size;

                let mut pos = offset;
                while pos < end {
                    let val = if binary.is_32bit {
                        let mut b = [0u8; 4];
                        b.copy_from_slice(&binary.bytes[pos..pos + 4]);
                        if binary.endian == Endianness::Little {
                            u32::from_le_bytes(b) as u64
                        } else {
                            u32::from_be_bytes(b) as u64
                        }
                    } else {
                        let mut b = [0u8; 8];
                        b.copy_from_slice(&binary.bytes[pos..pos + 8]);
                        if binary.endian == Endianness::Little {
                            u64::from_le_bytes(b)
                        } else {
                            u64::from_be_bytes(b)
                        }
                    };

                    if val < u32::MAX as u64 {
                        let encoded_token = val as u32;
                        let usage = get_encoded_index_type(encoded_token);
                        if usage > 0 && usage <= 6 {
                            let decoded_index = get_decoded_method_index(encoded_token);
                            if val == (((usage as u64) << 29) | ((decoded_index as u64) << 1)) + 1 {
                                let va = sec.address + (pos - offset) as u64;
                                let rva = binary.get_rva(va);

                                match usage {
                                    1 => {
                                        // kIl2CppMetadataUsageTypeInfo
                                        if decoded_index < self.executor.types.len() as u32 {
                                            let type_obj =
                                                &self.executor.types[decoded_index as usize];
                                            let type_name =
                                                self.executor.get_type_name(type_obj, true, false);
                                            let signature =
                                                get_il2cpp_struct_name(type_obj, self.executor);
                                            let signature_str = if signature.ends_with("_array") {
                                                "Il2CppClass*".to_string()
                                            } else {
                                                format!("{}_c*", signature)
                                            };
                                            script_metadata.push(ScriptMetadata {
                                                address: rva,
                                                name: format!("{}_TypeInfo", type_name),
                                                signature: signature_str,
                                            });
                                        }
                                    }
                                    2 => {
                                        // kIl2CppMetadataUsageIl2CppType
                                        if decoded_index < self.executor.types.len() as u32 {
                                            let type_obj =
                                                &self.executor.types[decoded_index as usize];
                                            let type_name =
                                                self.executor.get_type_name(type_obj, true, false);
                                            script_metadata.push(ScriptMetadata {
                                                address: rva,
                                                name: format!("{}_var", type_name),
                                                signature: "Il2CppType*".to_string(),
                                            });
                                        }
                                    }
                                    3 => {
                                        // kIl2CppMetadataUsageMethodDef
                                        if decoded_index < metadata.method_defs.len() as u32 {
                                            let method_def =
                                                &metadata.method_defs[decoded_index as usize];
                                            if method_def.declaring_type >= 0
                                                && (method_def.declaring_type as usize)
                                                    < metadata.type_defs.len()
                                            {
                                                let type_def = &metadata.type_defs
                                                    [method_def.declaring_type as usize];
                                                let type_name_full = self
                                                    .executor
                                                    .get_type_name_from_def(type_def, true, false);
                                                let method_name = metadata
                                                    .get_string_from_index(method_def.name_index);
                                                let full_name = format!(
                                                    "Method${}.{}()",
                                                    type_name_full, method_name
                                                );

                                                let mut method_address = 0;
                                                if let Some(image_name) =
                                                    self.executor.get_image_name_for_type(type_def)
                                                    && let Some(ptrs) = self
                                                        .executor
                                                        .code_gen_module_method_pointers
                                                        .get(&image_name)
                                                {
                                                    let method_pointer_index =
                                                        (method_def.token & 0x00FFFFFF) as usize;
                                                    if method_pointer_index > 0
                                                        && method_pointer_index <= ptrs.len()
                                                    {
                                                        let ptr = ptrs[method_pointer_index - 1];
                                                        method_address = binary.get_rva(ptr);
                                                    }
                                                }

                                                script_metadata_method.push(ScriptMetadataMethod {
                                                    address: rva,
                                                    name: full_name,
                                                    method_address,
                                                });
                                            }
                                        }
                                    }
                                    4 => {
                                        // kIl2CppMetadataUsageFieldInfo
                                        if decoded_index < metadata.field_refs.len() as u32 {
                                            let field_ref =
                                                &metadata.field_refs[decoded_index as usize];
                                            if field_ref.type_index >= 0
                                                && (field_ref.type_index as usize)
                                                    < self.executor.types.len()
                                            {
                                                let type_obj = &self.executor.types
                                                    [field_ref.type_index as usize];
                                                if let Some(type_def) = self
                                                    .executor
                                                    .get_type_definition_from_il2cpp_type(type_obj)
                                                {
                                                    let field_idx = type_def.field_start as i64
                                                        + field_ref.field_index as i64;
                                                    if field_idx >= 0
                                                        && (field_idx as usize)
                                                            < metadata.field_defs.len()
                                                    {
                                                        let field_def = &metadata.field_defs
                                                            [field_idx as usize];
                                                        let field_name = metadata
                                                            .get_string_from_index(
                                                                field_def.name_index,
                                                            );
                                                        let type_name = self.executor.get_type_name(
                                                            type_obj, true, false,
                                                        );
                                                        script_metadata.push(ScriptMetadata {
                                                            address: rva,
                                                            name: format!(
                                                                "Field${}.{}",
                                                                type_name, field_name
                                                            ),
                                                            signature: String::new(),
                                                        });
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    5 => {
                                        // kIl2CppMetadataUsageStringLiteral
                                        if decoded_index < metadata.string_literals.len() as u32
                                            && let Ok(val) = metadata
                                                .get_string_literal_from_index(decoded_index)
                                            {
                                                script_strings.push(ScriptString {
                                                    address: rva,
                                                    value: val,
                                                });
                                            }
                                    }
                                    6
                                        // kIl2CppMetadataUsageMethodRef
                                        if decoded_index < self.executor.method_specs.len() as u32 => {
                                            let method_spec =
                                                &self.executor.method_specs[decoded_index as usize];
                                            let (spec_type_name, spec_method_name) = self
                                                .executor
                                                .get_method_spec_name(method_spec, true);
                                            let full_name = format!(
                                                "Method${}.{}()",
                                                spec_type_name, spec_method_name
                                            );

                                            let mut method_address = 0;
                                            if let Some(&generic_method_pointer) = self
                                                .executor
                                                .method_spec_generic_method_pointers
                                                .get(&(decoded_index as usize))
                                                && generic_method_pointer > 0 {
                                                    method_address =
                                                        binary.get_rva(generic_method_pointer);
                                                }

                                            script_metadata_method.push(ScriptMetadataMethod {
                                                address: rva,
                                                name: full_name,
                                                method_address,
                                            });
                                        }
                                    _ => {}
                                }
                            }
                        }
                    }
                    pos += ptr_size;
                }
            }
        } else {
            // For version < 27
            for i in 0..metadata.string_literals.len() {
                if let Ok(val) = metadata.get_string_literal_from_index(i as u32)
                    && let Some(map) = metadata.metadata_usage_dic.get(&5)
                    && let Some(&usage_index) = map.get(&(i as u32))
                    && let Some(&va) = self.executor.metadata_usages.get(usage_index as usize)
                {
                    let rva = self.executor.binary.get_rva(va);
                    script_strings.push(ScriptString {
                        address: rva,
                        value: val,
                    });
                }
            }

            if let Some(map) = metadata.metadata_usage_dic.get(&1) {
                // TypeInfo
                for (&key, &val) in map {
                    if val < self.executor.types.len() as u32
                        && let Some(&va) = self.executor.metadata_usages.get(key as usize)
                    {
                        let rva = self.executor.binary.get_rva(va);
                        let type_obj = &self.executor.types[val as usize];
                        let type_name = self.executor.get_type_name(type_obj, true, false);
                        let signature = get_il2cpp_struct_name(type_obj, self.executor);
                        let signature_str = if signature.ends_with("_array") {
                            "Il2CppClass*".to_string()
                        } else {
                            format!("{}_c*", signature)
                        };
                        script_metadata.push(ScriptMetadata {
                            address: rva,
                            name: format!("{}_TypeInfo", type_name),
                            signature: signature_str,
                        });
                    }
                }
            }

            if let Some(map) = metadata.metadata_usage_dic.get(&2) {
                // Il2CppType
                for (&key, &val) in map {
                    if val < self.executor.types.len() as u32
                        && let Some(&va) = self.executor.metadata_usages.get(key as usize)
                    {
                        let rva = self.executor.binary.get_rva(va);
                        let type_obj = &self.executor.types[val as usize];
                        let type_name = self.executor.get_type_name(type_obj, true, false);
                        script_metadata.push(ScriptMetadata {
                            address: rva,
                            name: format!("{}_var", type_name),
                            signature: "Il2CppType*".to_string(),
                        });
                    }
                }
            }

            if let Some(map) = metadata.metadata_usage_dic.get(&3) {
                // MethodDef
                for (&key, &val) in map {
                    if val < metadata.method_defs.len() as u32
                        && let Some(&va) = self.executor.metadata_usages.get(key as usize)
                    {
                        let rva = self.executor.binary.get_rva(va);
                        let method_def = &metadata.method_defs[val as usize];
                        let type_def = &metadata.type_defs[method_def.declaring_type as usize];
                        let type_name_full =
                            self.executor.get_type_name_from_def(type_def, true, false);
                        let method_name = metadata.get_string_from_index(method_def.name_index);
                        let full_name = format!("Method${}.{}()", type_name_full, method_name);

                        let mut method_address = 0;
                        let method_ptr_idx = method_def.method_index as usize;
                        if let Some(&ptr) = self.executor.method_pointers.get(method_ptr_idx) {
                            method_address = self.executor.binary.get_rva(ptr);
                        }

                        script_metadata_method.push(ScriptMetadataMethod {
                            address: rva,
                            name: full_name,
                            method_address,
                        });
                    }
                }
            }

            if let Some(map) = metadata.metadata_usage_dic.get(&4) {
                // FieldInfo
                for (&key, &val) in map {
                    if val < metadata.field_refs.len() as u32
                        && let Some(&va) = self.executor.metadata_usages.get(key as usize)
                    {
                        let rva = self.executor.binary.get_rva(va);
                        let field_ref = &metadata.field_refs[val as usize];
                        let type_obj = &self.executor.types[field_ref.type_index as usize];
                        if let Some(type_def) =
                            self.executor.get_type_definition_from_il2cpp_type(type_obj)
                        {
                            let field_def = &metadata.field_defs
                                [type_def.field_start as usize + field_ref.field_index as usize];
                            let field_name = metadata.get_string_from_index(field_def.name_index);
                            let type_name = self.executor.get_type_name(type_obj, true, false);
                            script_metadata.push(ScriptMetadata {
                                address: rva,
                                name: format!("Field${}.{}", type_name, field_name),
                                signature: String::new(),
                            });
                        }
                    }
                }
            }

            if let Some(map) = metadata.metadata_usage_dic.get(&6) {
                // MethodRef
                for (&key, &val) in map {
                    if val < self.executor.method_specs.len() as u32
                        && let Some(&va) = self.executor.metadata_usages.get(key as usize)
                    {
                        let rva = self.executor.binary.get_rva(va);
                        let method_spec = &self.executor.method_specs[val as usize];
                        let (spec_type_name, spec_method_name) =
                            self.executor.get_method_spec_name(method_spec, true);
                        let full_name = format!("Method${}.{}()", spec_type_name, spec_method_name);

                        let mut method_address = 0;
                        if let Some(&generic_method_pointer) = self
                            .executor
                            .method_spec_generic_method_pointers
                            .get(&(val as usize))
                            && generic_method_pointer > 0
                        {
                            method_address = self.executor.binary.get_rva(generic_method_pointer);
                        }

                        script_metadata_method.push(ScriptMetadataMethod {
                            address: rva,
                            name: full_name,
                            method_address,
                        });
                    }
                }
            }
        }

        // Write the script JSON output file.
        let script_json = ScriptJson {
            script_method: script_methods,
            script_string: script_strings,
            script_metadata,
            script_metadata_method,
            addresses,
        };

        drop(w);
        let dump_cs = String::from_utf8(w_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"\t");
        let mut buf = Vec::new();
        let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
        script_json.serialize(&mut ser)?;
        let mut json_str =
            String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        json_str = json_str.replace('\u{2028}', "\\u2028");
        json_str = json_str.replace('\u{2029}', "\\u2029");

        Ok((dump_cs, json_str))
    }

    pub fn decompile(&self, config: &Config, output_dir: &Path) -> io::Result<()> {
        let (dump_cs, script_json) = self.decompile_to_memory(config)?;
        std::fs::write(output_dir.join("dump.cs"), dump_cs)?;
        std::fs::write(output_dir.join("script.json"), script_json)?;
        Ok(())
    }

    fn get_custom_attributes(
        &self,
        image_index: usize,
        custom_attribute_index: i32,
        token: u32,
    ) -> Vec<String> {
        let metadata = &self.executor.metadata;
        if metadata.version < 21.0 {
            return Vec::new();
        }

        let attribute_index =
            metadata.get_custom_attribute_index(image_index, custom_attribute_index, token);
        if attribute_index < 0 || attribute_index as usize >= metadata.attribute_data_ranges.len() {
            return Vec::new();
        }

        if metadata.version < 29.0 {
            return Vec::new();
        }

        let start_range = &metadata.attribute_data_ranges[attribute_index as usize];
        if attribute_index as usize + 1 >= metadata.attribute_data_ranges.len() {
            return Vec::new();
        }
        let end_range = &metadata.attribute_data_ranges[attribute_index as usize + 1];

        let start =
            metadata.header.attribute_data_offset as usize + start_range.start_offset as usize;
        let end = metadata.header.attribute_data_offset as usize + end_range.start_offset as usize;
        if start >= metadata.raw_bytes.len() || end > metadata.raw_bytes.len() || start > end {
            return Vec::new();
        }

        let buff = &metadata.raw_bytes[start..end];
        let cursor = std::io::Cursor::new(buff);
        let mut r = crate::binary_reader::BinaryReader::new(
            cursor,
            self.executor.binary.is_32bit,
            self.executor.binary.endian,
        );

        let mut attributes = Vec::new();
        if let Ok(count) = r.read_compressed_uint32() {
            let mut ctor_buffer = r.position().unwrap_or(0);
            let mut data_buffer = ctor_buffer + count as u64 * 4;

            for _ in 0..count {
                if r.seek(ctor_buffer).is_err() {
                    break;
                }
                let ctor_index = match r.read_i32() {
                    Ok(idx) => idx,
                    Err(_) => break,
                };
                ctor_buffer = r.position().unwrap_or(0);

                if ctor_index < 0 || ctor_index as usize >= metadata.method_defs.len() {
                    continue;
                }
                let method_def = &metadata.method_defs[ctor_index as usize];
                let type_def = &metadata.type_defs[method_def.declaring_type as usize];
                let attr_type_name = metadata
                    .get_string_from_index(type_def.name_index)
                    .replace("Attribute", "");

                if r.seek(data_buffer).is_err() {
                    break;
                }

                let arg_count = r.read_compressed_uint32().unwrap_or(0);
                let field_count = r.read_compressed_uint32().unwrap_or(0);
                let prop_count = r.read_compressed_uint32().unwrap_or(0);

                let mut args = Vec::new();
                let mut decode_ok = true;

                for _ in 0..arg_count {
                    if let Some(val_str) = self.read_attribute_data_value(&mut r) {
                        args.push(val_str);
                    } else {
                        decode_ok = false;
                        break;
                    }
                }

                if !decode_ok {
                    data_buffer = r.position().unwrap_or(0);
                    continue;
                }

                for _ in 0..field_count {
                    if let Some(val_str) = self.read_attribute_data_value(&mut r)
                        && let Ok(member_index) = r.read_compressed_int32()
                    {
                        let (declaring_class, field_idx) = if member_index >= 0 {
                            (type_def, member_index)
                        } else {
                            let decl_type_index = r.read_compressed_uint32().unwrap_or(0);
                            (
                                metadata
                                    .type_defs
                                    .get(decl_type_index as usize)
                                    .unwrap_or(type_def),
                                -(member_index + 1),
                            )
                        };
                        let field_def_idx =
                            declaring_class.field_start as usize + field_idx as usize;
                        if field_def_idx < metadata.field_defs.len() {
                            let field_def = &metadata.field_defs[field_def_idx];
                            let field_name = metadata.get_string_from_index(field_def.name_index);
                            args.push(format!("{} = {}", field_name, val_str));
                        }
                    }
                }

                for _ in 0..prop_count {
                    if let Some(val_str) = self.read_attribute_data_value(&mut r)
                        && let Ok(member_index) = r.read_compressed_int32()
                    {
                        let (declaring_class, prop_idx) = if member_index >= 0 {
                            (type_def, member_index)
                        } else {
                            let decl_type_index = r.read_compressed_uint32().unwrap_or(0);
                            (
                                metadata
                                    .type_defs
                                    .get(decl_type_index as usize)
                                    .unwrap_or(type_def),
                                -(member_index + 1),
                            )
                        };
                        let prop_def_idx =
                            declaring_class.property_start as usize + prop_idx as usize;
                        if prop_def_idx < metadata.property_defs.len() {
                            let prop_def = &metadata.property_defs[prop_def_idx];
                            let prop_name = metadata.get_string_from_index(prop_def.name_index);
                            args.push(format!("{} = {}", prop_name, val_str));
                        }
                    }
                }

                data_buffer = r.position().unwrap_or(0);

                if args.is_empty() {
                    attributes.push(format!("[{}]", attr_type_name));
                } else {
                    attributes.push(format!("[{}({})]", attr_type_name, args.join(", ")));
                }
            }
        }

        attributes
    }

    fn read_attribute_data_value<R: Read + Seek>(
        &self,
        r: &mut crate::binary_reader::BinaryReader<R>,
    ) -> Option<String> {
        let type_byte = r.read_u8().ok()?;
        let mut type_enum = crate::il2cpp_binary_structures::Il2CppTypeEnum::from_u8(type_byte);
        let mut enum_type = None;

        if type_enum == crate::il2cpp_binary_structures::Il2CppTypeEnum::Enum {
            let enum_type_index = r.read_compressed_int32().ok()?;
            if let Some(et) = self.executor.types.get(enum_type_index as usize) {
                enum_type = Some(et);
                if let Some(type_def) = self.executor.get_type_definition_from_il2cpp_type(et) {
                    let elem_type = self
                        .executor
                        .types
                        .get(type_def.element_type_index as usize)?;
                    type_enum = elem_type.type_enum();
                }
            }
        }

        self.read_constant_value_from_blob(r, type_enum, enum_type)
    }

    fn read_constant_value_from_blob<R: Read + Seek>(
        &self,
        r: &mut crate::binary_reader::BinaryReader<R>,
        type_enum: crate::il2cpp_binary_structures::Il2CppTypeEnum,
        enum_type: Option<&crate::il2cpp_binary_structures::Il2CppType>,
    ) -> Option<String> {
        match type_enum {
            Il2CppTypeEnum::Boolean => {
                let b = r.read_u8().ok()?;
                Some(if b != 0 {
                    "True".to_string()
                } else {
                    "False".to_string()
                })
            }
            Il2CppTypeEnum::Char => {
                let mut buf = [0u8; 2];
                r.read_exact(&mut buf).ok()?;
                let c = if r.endian == Endianness::Little {
                    u16::from_le_bytes(buf)
                } else {
                    u16::from_be_bytes(buf)
                };
                if let Some(ch) = std::char::from_u32(c as u32) {
                    Some(format!("'{}'", ch.escape_default()))
                } else {
                    Some(format!("(char){}", c))
                }
            }
            Il2CppTypeEnum::I1 => Some(format!("{}", r.read_i8().ok()?)),
            Il2CppTypeEnum::U1 => Some(format!("{}", r.read_u8().ok()?)),
            Il2CppTypeEnum::I2 => Some(format!("{}", r.read_i16().ok()?)),
            Il2CppTypeEnum::U2 => Some(format!("{}", r.read_u16().ok()?)),
            Il2CppTypeEnum::I4 => Some(format!("{}", r.read_compressed_int32().ok()?)),
            Il2CppTypeEnum::U4 => Some(format!("{}", r.read_compressed_uint32().ok()?)),
            Il2CppTypeEnum::I8 => Some(format!("{}", r.read_i64().ok()?)),
            Il2CppTypeEnum::U8 => Some(format!("{}", r.read_u64().ok()?)),
            Il2CppTypeEnum::R4 => Some(format!("{}", r.read_f32().ok()?)),
            Il2CppTypeEnum::R8 => Some(format!("{}", r.read_f64().ok()?)),
            Il2CppTypeEnum::String => {
                let len = r.read_compressed_int32().ok()?;
                if len == -1 {
                    Some("null".to_string())
                } else if len <= 0 || len > 10_000_000 {
                    Some("\"\"".to_string())
                } else {
                    let mut buf = vec![0u8; len as usize];
                    r.read_exact(&mut buf).ok()?;
                    let s = String::from_utf8(buf).ok()?;
                    Some(format!("\"{}\"", s.escape_default()))
                }
            }
            Il2CppTypeEnum::SzArray => {
                let len = r.read_compressed_int32().ok()?;
                if len == -1 || len < 0 || len > 10_000 {
                    Some("null".to_string())
                } else {
                    let elem_type_byte = r.read_u8().ok()?;
                    let mut elem_type_enum =
                        crate::il2cpp_binary_structures::Il2CppTypeEnum::from_u8(elem_type_byte);
                    let mut elem_enum_type = None;
                    if elem_type_enum == crate::il2cpp_binary_structures::Il2CppTypeEnum::Enum {
                        let enum_type_index = r.read_compressed_int32().ok()?;
                        if let Some(et) = self.executor.types.get(enum_type_index as usize) {
                            elem_enum_type = Some(et);
                            if let Some(type_def) =
                                self.executor.get_type_definition_from_il2cpp_type(et)
                            {
                                let elem_type = self
                                    .executor
                                    .types
                                    .get(type_def.element_type_index as usize)?;
                                elem_type_enum = elem_type.type_enum();
                            }
                        }
                    }

                    let elements_are_different = r.read_u8().ok()? == 1;

                    let mut items = Vec::new();
                    for _ in 0..len {
                        let mut current_elem_type_enum = elem_type_enum;
                        let mut current_elem_enum_type = elem_enum_type;

                        if elements_are_different {
                            let curr_type_byte = r.read_u8().ok()?;
                            current_elem_type_enum =
                                crate::il2cpp_binary_structures::Il2CppTypeEnum::from_u8(
                                    curr_type_byte,
                                );
                            if current_elem_type_enum
                                == crate::il2cpp_binary_structures::Il2CppTypeEnum::Enum
                            {
                                let enum_type_index = r.read_compressed_int32().ok()?;
                                if let Some(et) = self.executor.types.get(enum_type_index as usize)
                                {
                                    current_elem_enum_type = Some(et);
                                    if let Some(type_def) =
                                        self.executor.get_type_definition_from_il2cpp_type(et)
                                    {
                                        let elem_type = self
                                            .executor
                                            .types
                                            .get(type_def.element_type_index as usize)?;
                                        current_elem_type_enum = elem_type.type_enum();
                                    }
                                }
                            }
                        }

                        items.push(self.read_constant_value_from_blob(
                            r,
                            current_elem_type_enum,
                            current_elem_enum_type,
                        )?);
                    }
                    Some(format!("new[] {{ {} }}", items.join(", ")))
                }
            }
            Il2CppTypeEnum::Index => {
                let type_idx = r.read_compressed_int32().ok()?;
                if type_idx == -1 {
                    Some("null".to_string())
                } else if let Some(ty) = self.executor.types.get(type_idx as usize) {
                    Some(format!(
                        "typeof({})",
                        self.executor.get_type_name(ty, false, false)
                    ))
                } else {
                    Some("typeof(object)".to_string())
                }
            }
            _ => None,
        }
    }

    fn implements_imessage(&self, type_def: &Il2CppTypeDefinition) -> bool {
        let metadata = &self.executor.metadata;
        if type_def.interfaces_count > 0 {
            for i in 0..type_def.interfaces_count {
                let idx =
                    metadata.interface_indices[type_def.interfaces_start as usize + i as usize];
                if let Some(iface_ty) = self.executor.types.get(idx as usize) {
                    let name = self.executor.get_type_name(iface_ty, false, false);
                    if name.contains("IMessage") {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn trace_protobuf_tags(
        &self,
        image_index: usize,
        image_name: &str,
        type_def: &Il2CppTypeDefinition,
    ) -> HashMap<u32, u32> {
        let mut field_to_tag = HashMap::new();
        let metadata = &self.executor.metadata;
        let binary = &self.executor.binary;

        let method_end = type_def.method_start + type_def.method_count as i32;
        let mut icy_ptr = 0;
        for i in type_def.method_start..method_end {
            let method_def = &metadata.method_defs[i as usize];

            let mut matches_parse_context = false;
            if method_def.parameter_count == 1 {
                let param_def = &metadata.parameter_defs[method_def.parameter_start as usize];
                let param_type = &self.executor.types[param_def.type_index as usize];
                let param_type_name = self.executor.get_type_name(param_type, false, false);
                if param_type_name.contains("ParseContext") {
                    matches_parse_context = true;
                }
            }

            if matches_parse_context {
                icy_ptr = if self.executor.metadata.version >= 24.2 {
                    if let Some(ptrs) = self
                        .executor
                        .code_gen_module_method_pointers
                        .get(image_name)
                    {
                        let method_pointer_index = (method_def.token & 0x00FFFFFF) as usize;
                        if method_pointer_index > 0 && method_pointer_index <= ptrs.len() {
                            ptrs[method_pointer_index - 1]
                        } else {
                            0
                        }
                    } else {
                        0
                    }
                } else {
                    let method_ptr_idx = method_def.method_index as usize;
                    self.executor
                        .method_pointers
                        .get(method_ptr_idx)
                        .cloned()
                        .unwrap_or(0)
                };
                if icy_ptr > 0 {
                    break;
                }
            }
        }

        if icy_ptr == 0 {
            return field_to_tag;
        }

        let mut next_ptr = 0;
        if let Some(ptrs) = self
            .executor
            .code_gen_module_method_pointers
            .get(image_name)
        {
            for &ptr in ptrs {
                if ptr > icy_ptr && (next_ptr == 0 || ptr < next_ptr) {
                    next_ptr = ptr;
                }
            }
        }

        let max_len = if next_ptr > icy_ptr {
            (next_ptr - icy_ptr) as usize
        } else {
            4000
        };

        let method_len = std::cmp::min(max_len, 8000);

        let file_offset = binary.map_vatr(icy_ptr).unwrap_or(0) as usize;
        if file_offset == 0 || file_offset + method_len > binary.bytes.len() {
            return field_to_tag;
        }

        let mut instructions = Vec::new();
        for offset in (file_offset..file_offset + method_len).step_by(4) {
            let b = &binary.bytes[offset..offset + 4];
            let inst = u32::from_le_bytes([b[0], b[1], b[2], b[3]]);
            instructions.push((offset, inst));
        }

        let decode_store_load = |inst: u32| -> Option<(&'static str, usize, usize, i32)> {
            let op10 = (inst >> 22) & 0x3ff;
            let rt = (inst & 0x1f) as usize;
            let rn = ((inst >> 5) & 0x1f) as usize;
            let imm12 = (inst >> 10) & 0xfff;

            let op_low8 = op10 & 0xff;
            if op_low8 == 0xe4 || op_low8 == 0xe5 || op_low8 == 0xf4 || op_low8 == 0xf5 {
                let size_shift = (op10 >> 8) & 3;
                let offset = (imm12 << size_shift) as i32;
                let op_name = if op_low8 == 0xe4 || op_low8 == 0xf4 {
                    "STR"
                } else {
                    "LDR"
                };
                return Some((op_name, rn, rt, offset));
            }

            let op11 = (inst >> 21) & 0x7ff;
            let op11_low8 = op11 & 0xff;
            if op11_low8 == 0xc0 || op11_low8 == 0xc2 || op11_low8 == 0xe0 || op11_low8 == 0xe2 {
                let mut imm9 = ((inst >> 12) & 0x1ff) as i32;
                if (imm9 & 0x100) != 0 {
                    imm9 -= 0x200;
                }
                let op_name = if op11_low8 == 0xc0 || op11_low8 == 0xe0 {
                    "STUR"
                } else {
                    "LDUR"
                };
                return Some((op_name, rn, rt, imm9));
            }

            None
        };

        let mut base_reg_opt = None;
        for &(_, inst) in instructions.iter().take(25) {
            if (inst & 0xffffffe0) == 0xaa0003e0 {
                let rd = (inst & 0x1f) as usize;
                if (19..=28).contains(&rd) {
                    base_reg_opt = Some(rd);
                    break;
                }
            }
        }

        let base_reg = if let Some(reg) = base_reg_opt {
            reg
        } else {
            let mut rn_counts = HashMap::new();
            for &(_, inst) in &instructions {
                if let Some((op_name, rn, _, _)) =
                    decode_store_load(inst).filter(|&(op_name, rn, _, _)| {
                        (19..=28).contains(&rn)
                            && (op_name.starts_with("STR") || op_name.starts_with("STUR"))
                    })
                {
                    *rn_counts.entry(rn).or_insert(0) += 1;
                }
            }
            if rn_counts.is_empty() {
                return field_to_tag;
            }
            *rn_counts.iter().max_by_key(|&(_, count)| count).unwrap().0
        };

        let mut target_to_tag = BTreeMap::new();
        let mut last_cmp_val = None;
        let mut last_cmp_addr = None;

        for &(addr, inst) in &instructions {
            let op = inst >> 22;
            if op == 0x1c4 {
                let rd = inst & 0x1f;
                let imm12 = (inst >> 10) & 0xfff;
                if rd == 31 && imm12 >= 8 {
                    last_cmp_val = Some(imm12);
                    last_cmp_addr = Some(addr);
                }
            } else if (inst >> 24) == 0x54 {
                let cond = inst & 0x1f;
                let mut imm19 = ((inst >> 5) & 0x7ffff) as i32;
                if (imm19 & 0x40000) != 0 {
                    imm19 -= 0x80000;
                }
                let target_addr = (addr as i32 + imm19 * 4) as usize;

                if last_cmp_addr == Some(addr - 4) {
                    if let Some(val) = last_cmp_val {
                        if cond == 0 {
                            target_to_tag.insert(target_addr, val);
                        } else if cond == 1 {
                            target_to_tag.insert(addr + 4, val);
                        }
                    }
                    last_cmp_val = None;
                    last_cmp_addr = None;
                }
            }
        }

        for (&target_addr, &tag) in &target_to_tag {
            let mut target_idx = None;
            for (idx, &(addr, _)) in instructions.iter().enumerate() {
                if addr == target_addr {
                    target_idx = Some(idx);
                    break;
                }
            }

            if let Some(t_idx) = target_idx {
                let limit = std::cmp::min(t_idx + 40, instructions.len());
                for &(_, inst) in &instructions[t_idx..limit] {
                    if (inst >> 26) == 5 {
                        break;
                    }
                    if let Some((_, _, _, offset)) = decode_store_load(inst)
                        .filter(|&(_, rn, _, offset)| rn == base_reg && offset >= 0)
                    {
                        field_to_tag.insert(offset as u32, tag >> 3);
                        break;
                    }
                }
            }
        }

        field_to_tag
    }

    fn get_method_type_signature(
        &self,
        method_def: &Il2CppMethodDefinition,
        type_def: &Il2CppTypeDefinition,
    ) -> String {
        let metadata = &self.executor.metadata;
        let mut method_type_sigs = Vec::new();

        // 1. Return type
        let return_type_desc = &self.executor.types[method_def.return_type as usize];
        method_type_sigs.push(if return_type_desc.byref == 1 {
            Il2CppTypeEnum::Ptr
        } else {
            return_type_desc.type_enum()
        });

        // 2. This pointer (if non-static)
        if (method_def.flags & METHOD_ATTRIBUTE_STATIC) == 0 {
            method_type_sigs
                .push(self.executor.types[type_def.byval_type_index as usize].type_enum());
        } else if self.executor.metadata.version <= 24.0 {
            method_type_sigs.push(Il2CppTypeEnum::Ptr);
        }

        // 3. Parameters
        for p in 0..method_def.parameter_count {
            let param_def =
                &metadata.parameter_defs[method_def.parameter_start as usize + p as usize];
            let param_type = &self.executor.types[param_def.type_index as usize];
            method_type_sigs.push(if param_type.byref == 1 {
                Il2CppTypeEnum::Ptr
            } else {
                param_type.type_enum()
            });
        }

        // 4. MethodInfo pointer
        method_type_sigs.push(Il2CppTypeEnum::Ptr);

        let mut signature = String::new();
        for ty in method_type_sigs {
            let char_code = match ty {
                Il2CppTypeEnum::Void => 'v',
                Il2CppTypeEnum::Boolean
                | Il2CppTypeEnum::Char
                | Il2CppTypeEnum::I1
                | Il2CppTypeEnum::U1
                | Il2CppTypeEnum::I2
                | Il2CppTypeEnum::U2
                | Il2CppTypeEnum::I4
                | Il2CppTypeEnum::U4 => 'i',
                Il2CppTypeEnum::I8 | Il2CppTypeEnum::U8 => 'j',
                Il2CppTypeEnum::R4 => 'f',
                Il2CppTypeEnum::R8 => 'd',
                Il2CppTypeEnum::String
                | Il2CppTypeEnum::Ptr
                | Il2CppTypeEnum::ValueType
                | Il2CppTypeEnum::Class
                | Il2CppTypeEnum::Var
                | Il2CppTypeEnum::Array
                | Il2CppTypeEnum::GenericInst
                | Il2CppTypeEnum::TypedByRef
                | Il2CppTypeEnum::I
                | Il2CppTypeEnum::U
                | Il2CppTypeEnum::Object
                | Il2CppTypeEnum::SzArray
                | Il2CppTypeEnum::MVar => 'i',
                _ => 'i',
            };
            signature.push(char_code);
        }
        signature
    }
}

// Extend BinaryFile with an RVA helper.
impl crate::binary::BinaryFile {
    pub fn get_rva(&self, ptr: u64) -> u64 {
        ptr - self.image_base
    }
}

fn get_encoded_index_type(encoded_token: u32) -> u32 {
    (encoded_token & 0xE0000000) >> 29
}

fn get_decoded_method_index(encoded_token: u32) -> u32 {
    (encoded_token & 0x1FFFFFFE) >> 1
}

fn fix_name(name: &str) -> String {
    let mut res = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_ascii_alphanumeric() || c == '_' {
            if i == 0 && c.is_ascii_digit() {
                res.push('_');
            }
            res.push(c);
        } else {
            res.push('_');
        }
    }
    res
}

fn get_il2cpp_struct_name(
    ty: &crate::il2cpp_binary_structures::Il2CppType,
    executor: &Il2CppExecutor,
) -> String {
    match ty.type_enum() {
        Il2CppTypeEnum::Array | Il2CppTypeEnum::SzArray => {
            let element_ptr = if ty.type_enum() == Il2CppTypeEnum::SzArray {
                ty.datapoint
            } else {
                0
            };
            let mut elem_name = "object".to_string();
            if element_ptr > 0
                && let Some(elem_ty) = executor.get_il2cpp_type(element_ptr)
            {
                elem_name = get_il2cpp_struct_name(elem_ty, executor);
            }
            format!("{}_array", elem_name)
        }
        Il2CppTypeEnum::Ptr => {
            if let Some(ori_ty) = executor.get_il2cpp_type(ty.datapoint) {
                get_il2cpp_struct_name(ori_ty, executor)
            } else {
                "object".to_string()
            }
        }
        _ => {
            if let Some(type_def) = executor.get_type_definition_from_il2cpp_type(ty) {
                let name = executor.get_type_name_from_def(type_def, true, false);
                fix_name(&name)
            } else {
                "object".to_string()
            }
        }
    }
}
