#![allow(dead_code, unused_variables, unused_imports, unused_mut)]
use crate::binary::BinaryFile;
use crate::binary_reader::BinaryReader;
use crate::il2cpp_binary_structures::*;
use crate::metadata::{Il2CppRGCTXDefinition, Metadata};
use std::collections::HashMap;
use std::io::{self, Cursor, Read, Seek};

lazy_static::lazy_static! {
    static ref TYPE_STRING: HashMap<u8, &'static str> = {
        let mut m = HashMap::new();
        m.insert(1, "void");
        m.insert(2, "bool");
        m.insert(3, "char");
        m.insert(4, "sbyte");
        m.insert(5, "byte");
        m.insert(6, "short");
        m.insert(7, "ushort");
        m.insert(8, "int");
        m.insert(9, "uint");
        m.insert(10, "long");
        m.insert(11, "ulong");
        m.insert(12, "float");
        m.insert(13, "double");
        m.insert(14, "string");
        m.insert(22, "TypedReference");
        m.insert(24, "IntPtr");
        m.insert(25, "UIntPtr");
        m.insert(28, "object");
        m
    };
}

pub struct Il2CppExecutor {
    pub metadata: Metadata,
    pub binary: BinaryFile,
    pub code_registration: Il2CppCodeRegistration,
    pub metadata_registration: Il2CppMetadataRegistration,
    pub generic_method_pointers: Vec<u64>,
    pub invoker_pointers: Vec<u64>,
    pub custom_attribute_generators: Vec<u64>,
    pub metadata_usages: Vec<u64>,
    pub reverse_pinvoke_wrappers: Vec<u64>,
    pub unresolved_virtual_call_pointers: Vec<u64>,
    pub generic_inst_pointers: Vec<u64>,
    pub generic_insts: Vec<Il2CppGenericInst>,
    pub field_offsets: Vec<u64>,
    pub types: Vec<Il2CppType>,
    pub type_dic: HashMap<u64, Il2CppType>,
    pub code_gen_modules: HashMap<String, Il2CppCodeGenModule>,
    pub code_gen_module_method_pointers: HashMap<String, Vec<u64>>,
    pub rgctxs_dictionary: HashMap<String, HashMap<u32, Vec<Il2CppRGCTXDefinition>>>,
    pub method_pointers: Vec<u64>,
    pub generic_method_table: Vec<Il2CppGenericMethodFunctionsDefinitions>,
    pub method_specs: Vec<Il2CppMethodSpec>,

    // Auxiliary maps
    pub method_definition_method_specs: HashMap<i32, Vec<(usize, Il2CppMethodSpec)>>,
    pub method_spec_generic_method_pointers: HashMap<usize, u64>, // index of MethodSpec -> generic method pointer
    pub field_offsets_are_pointers: bool,
    pub code_registration_address: u64,
    pub metadata_registration_address: u64,
}

impl Il2CppExecutor {
    pub fn new(
        metadata: Metadata,
        binary: BinaryFile,
        code_reg_addr: u64,
        metadata_reg_addr: u64,
    ) -> io::Result<Self> {
        let version = metadata.version;

        let mut code_reg = {
            let mut r = binary.get_reader_at(code_reg_addr)?;
            Il2CppCodeRegistration::decode(&mut r, version)?
        };

        // Shift version if needed (heuristics from AutoPlusInit)
        let mut adjusted_version = version;
        let limit = 0x50000u64; // Limit for size checks
        let ptr_size = if binary.is_32bit { 4 } else { 8 };
        let mut adjusted_code_reg_addr = code_reg_addr;

        if version >= 24.2 {
            if version == 31.0 {
                if code_reg.generic_method_pointers_count > limit {
                    adjusted_code_reg_addr -= ptr_size * 2;
                } else {
                    adjusted_version = 29.0;
                    println!("Change il2cpp version to: {}", adjusted_version);
                }
            }
            if adjusted_version == 29.0 && code_reg.generic_method_pointers_count > limit {
                adjusted_version = 29.1;
                adjusted_code_reg_addr -= ptr_size * 2;
                println!("Change il2cpp version to: {}", adjusted_version);
            }
            if adjusted_version == 27.0 && code_reg.reverse_pinvoke_wrapper_count > limit {
                adjusted_version = 27.1;
                adjusted_code_reg_addr -= ptr_size;
                println!("Change il2cpp version to: {}", adjusted_version);
            }
            if adjusted_version == 24.4 {
                adjusted_code_reg_addr -= ptr_size * 2;
                if code_reg.reverse_pinvoke_wrapper_count > limit {
                    adjusted_version = 24.5;
                    adjusted_code_reg_addr -= ptr_size;
                    println!("Change il2cpp version to: {}", adjusted_version);
                }
            }
            if adjusted_version == 24.2 && code_reg.interop_data_count == 0 {
                adjusted_version = 24.3;
                adjusted_code_reg_addr -= ptr_size * 2;
                println!("Change il2cpp version to: {}", adjusted_version);
            }
            // Re-decode with adjusted version and address.
            let mut r = binary.get_reader_at(adjusted_code_reg_addr)?;
            code_reg = Il2CppCodeRegistration::decode(&mut r, adjusted_version)?;
        }

        let mut metadata_reg = {
            let mut r = binary.get_reader_at(metadata_reg_addr)?;
            Il2CppMetadataRegistration::decode(&mut r, adjusted_version)?
        };

        // Read generic method pointers and invoker pointers
        let generic_method_pointers = read_ptr_array(
            &binary,
            code_reg.generic_method_pointers,
            code_reg.generic_method_pointers_count as usize,
        )?;
        let invoker_pointers = read_ptr_array(
            &binary,
            code_reg.invoker_pointers,
            code_reg.invoker_pointers_count as usize,
        )?;

        let mut custom_attribute_generators = Vec::new();
        if adjusted_version < 27.0 {
            custom_attribute_generators = read_ptr_array(
                &binary,
                code_reg.custom_attribute_generators,
                code_reg.custom_attribute_count as usize,
            )?;
        }

        let mut metadata_usages = Vec::new();
        if adjusted_version > 16.0 && adjusted_version < 27.0 {
            metadata_usages = read_ptr_array(
                &binary,
                metadata_reg.metadata_usages,
                metadata.metadata_usages_count,
            )?;
        }

        let mut reverse_pinvoke_wrappers = Vec::new();
        let mut unresolved_virtual_call_pointers = Vec::new();
        if adjusted_version >= 22.0 {
            if code_reg.reverse_pinvoke_wrapper_count > 0 {
                reverse_pinvoke_wrappers = read_ptr_array(
                    &binary,
                    code_reg.reverse_pinvoke_wrappers,
                    code_reg.reverse_pinvoke_wrapper_count as usize,
                )?;
            }
            if code_reg.unresolved_virtual_call_count > 0 {
                unresolved_virtual_call_pointers = read_ptr_array(
                    &binary,
                    code_reg.unresolved_virtual_call_pointers,
                    code_reg.unresolved_virtual_call_count as usize,
                )?;
            }
        }

        // Generic insts
        let generic_inst_pointers = read_ptr_array(
            &binary,
            metadata_reg.generic_insts,
            metadata_reg.generic_insts_count as usize,
        )?;
        let mut generic_insts = Vec::with_capacity(generic_inst_pointers.len());
        for &ptr in &generic_inst_pointers {
            let mut r = binary.get_reader_at(ptr)?;
            generic_insts.push(Il2CppGenericInst::decode(&mut r)?);
        }

        // Field offsets
        let field_offsets_are_pointers = adjusted_version > 21.0;
        let mut field_offsets = Vec::new();
        if field_offsets_are_pointers {
            field_offsets = read_ptr_array(
                &binary,
                metadata_reg.field_offsets,
                metadata_reg.field_offsets_count as usize,
            )?;
        } else {
            let count = metadata_reg.field_offsets_count as usize;
            if count > 0 {
                let mut r = binary.get_reader_at(metadata_reg.field_offsets)?;
                for _ in 0..count {
                    field_offsets.push(r.read_u32()? as u64);
                }
            }
        }

        // Types
        let p_types = read_ptr_array(
            &binary,
            metadata_reg.types,
            metadata_reg.types_count as usize,
        )?;
        let mut types = Vec::with_capacity(p_types.len());
        let mut type_dic = HashMap::new();
        for &ptr in &p_types {
            let mut r = binary.get_reader_at(ptr)?;
            let mut ty = Il2CppType::decode(&mut r, adjusted_version)?;
            types.push(ty.clone());
            type_dic.insert(ptr, ty);
        }

        // CodeGenModules (version >= 24.2)
        let mut code_gen_modules = HashMap::new();
        let mut code_gen_module_method_pointers = HashMap::new();
        let mut rgctxs_dictionary = HashMap::new();
        let mut method_pointers = Vec::new();

        if adjusted_version >= 24.2 {
            let p_code_gen_modules = read_ptr_array(
                &binary,
                code_reg.code_gen_modules,
                code_reg.code_gen_modules_count as usize,
            )?;
            for &p_mod in &p_code_gen_modules {
                let mut r = binary.get_reader_at(p_mod)?;
                let module = Il2CppCodeGenModule::decode(&mut r, adjusted_version)?;
                let module_name = binary.read_string_to_null(module.module_name)?;

                code_gen_modules.insert(module_name.clone(), module.clone());

                let mut module_pointers = Vec::new();
                if module.method_pointers > 0 && module.method_pointer_count > 0 {
                    if let Ok(ptrs) = read_ptr_array(
                        &binary,
                        module.method_pointers,
                        module.method_pointer_count as usize,
                    ) {
                        module_pointers = ptrs;
                    } else {
                        // Fall back to a zero-filled array of the expected size, matching C# behavior.
                        module_pointers = vec![0; module.method_pointer_count as usize];
                    }
                }
                code_gen_module_method_pointers.insert(module_name.clone(), module_pointers);

                let mut rgctxs_def_dic = HashMap::new();
                if module.rgctxs_count > 0 {
                    let mut r_rgctxs = binary.get_reader_at(module.rgctxs)?;
                    let mut rgctxs = Vec::with_capacity(module.rgctxs_count as usize);
                    for _ in 0..module.rgctxs_count {
                        rgctxs.push(Il2CppRGCTXDefinition::decode(
                            &mut r_rgctxs,
                            adjusted_version,
                        )?);
                    }

                    let mut r_ranges = binary.get_reader_at(module.rgctx_ranges)?;
                    let mut rgctx_ranges = Vec::with_capacity(module.rgctx_ranges_count as usize);
                    for _ in 0..module.rgctx_ranges_count {
                        rgctx_ranges.push(Il2CppTokenRangePair::decode(&mut r_ranges)?);
                    }

                    for pair in rgctx_ranges {
                        let start = pair.range.start as usize;
                        let len = pair.range.length as usize;
                        if start + len <= rgctxs.len() {
                            let range_defs = rgctxs[start..start + len].to_vec();
                            rgctxs_def_dic.insert(pair.token, range_defs);
                        }
                    }
                }
                rgctxs_dictionary.insert(module_name.clone(), rgctxs_def_dic);
            }
        } else {
            method_pointers = read_ptr_array(
                &binary,
                code_reg.method_pointers,
                code_reg.method_pointers_count as usize,
            )?;
        }

        // Generic method table & specs
        let mut generic_method_table = Vec::new();
        if metadata_reg.generic_method_table_count > 0 {
            let mut r = binary.get_reader_at(metadata_reg.generic_method_table)?;
            for _ in 0..metadata_reg.generic_method_table_count {
                generic_method_table.push(Il2CppGenericMethodFunctionsDefinitions::decode(
                    &mut r,
                    adjusted_version,
                )?);
            }
        }

        let mut method_specs = Vec::new();
        if metadata_reg.method_specs_count > 0 {
            let mut r = binary.get_reader_at(metadata_reg.method_specs)?;
            for _ in 0..metadata_reg.method_specs_count {
                method_specs.push(Il2CppMethodSpec::decode(&mut r)?);
            }
        }

        let mut method_definition_method_specs = HashMap::new();
        let mut method_spec_generic_method_pointers = HashMap::new();
        for table in &generic_method_table {
            let spec_idx = table.generic_method_index as usize;
            if spec_idx < method_specs.len() {
                let spec = &method_specs[spec_idx];
                method_definition_method_specs
                    .entry(spec.method_definition_index)
                    .or_insert_with(Vec::new)
                    .push((spec_idx, spec.clone()));

                let pointer_idx = table.indices.method_index as usize;
                if pointer_idx < generic_method_pointers.len() {
                    method_spec_generic_method_pointers
                        .insert(spec_idx, generic_method_pointers[pointer_idx]);
                }
            }
        }

        // Parse custom attributes generators in version >= 27
        if (27.0..29.0).contains(&adjusted_version) {
            let total_attributes = metadata
                .image_defs
                .iter()
                .map(|x| x.custom_attribute_count as usize)
                .sum();
            custom_attribute_generators = vec![0; total_attributes];
            for image_def in &metadata.image_defs {
                let image_name =
                    metadata.get_string_from_index(image_name_index_fallback(image_def));
                if let Some(code_gen_module) = code_gen_modules.get(&image_name)
                    && image_def.custom_attribute_count > 0
                    && let Ok(pointers) = read_ptr_array(
                        &binary,
                        code_gen_module.custom_attribute_cache_generator,
                        image_def.custom_attribute_count as usize,
                    )
                {
                    let start = image_def.custom_attribute_start as usize;
                    let end = start + image_def.custom_attribute_count as usize;
                    if end <= custom_attribute_generators.len() {
                        custom_attribute_generators[start..end].copy_from_slice(&pointers);
                    }
                }
            }
        }

        Ok(Self {
            metadata,
            binary,
            code_registration: code_reg,
            metadata_registration: metadata_reg,
            generic_method_pointers,
            invoker_pointers,
            custom_attribute_generators,
            metadata_usages,
            reverse_pinvoke_wrappers,
            unresolved_virtual_call_pointers,
            generic_inst_pointers,
            generic_insts,
            field_offsets,
            types,
            type_dic,
            code_gen_modules,
            code_gen_module_method_pointers,
            rgctxs_dictionary,
            method_pointers,
            generic_method_table,
            method_specs,
            method_definition_method_specs,
            method_spec_generic_method_pointers,
            field_offsets_are_pointers,
            code_registration_address: adjusted_code_reg_addr,
            metadata_registration_address: metadata_reg_addr,
        })
    }

    pub fn get_il2cpp_type(&self, pointer: u64) -> Option<&Il2CppType> {
        self.type_dic.get(&pointer)
    }

    pub fn get_field_offset_from_index(
        &self,
        type_index: usize,
        field_index_in_type: usize,
        field_index: usize,
        is_value_type: bool,
        is_static: bool,
    ) -> i32 {
        let mut offset = -1;
        if self.field_offsets_are_pointers {
            if type_index < self.field_offsets.len() {
                let ptr = self.field_offsets[type_index];
                if ptr > 0 {
                    let va = ptr + 4 * field_index_in_type as u64;
                    if let Ok(mut r) = self.binary.get_reader_at(va)
                        && let Ok(val) = r.read_i32()
                    {
                        offset = val;
                    }
                }
            }
        } else {
            if field_index < self.field_offsets.len() {
                offset = self.field_offsets[field_index] as i32;
            }
        }
        if offset > 0 && is_value_type && !is_static {
            if self.binary.is_32bit {
                offset -= 8;
            } else {
                offset -= 16;
            }
        }
        offset
    }

    pub fn get_generic_parameter_from_type(
        &self,
        ty: &Il2CppType,
    ) -> Option<&crate::metadata::Il2CppGenericParameter> {
        let index = ty.generic_parameter_index() as usize;
        self.metadata.generic_parameters.get(index)
    }

    pub fn get_generic_class_type_definition(
        &self,
        generic_class: &Il2CppGenericClass,
    ) -> Option<&crate::metadata::Il2CppTypeDefinition> {
        if self.metadata.version >= 27.0 {
            let ty = self.get_il2cpp_type(generic_class.type_ptr)?;
            self.metadata.type_defs.get(ty.klass_index() as usize)
        } else {
            let index = generic_class.type_def_index as usize;
            self.metadata.type_defs.get(index)
        }
    }

    pub fn get_type_name(&self, ty: &Il2CppType, add_namespace: bool, is_nested: bool) -> String {
        let type_enum = ty.type_enum();
        match type_enum {
            Il2CppTypeEnum::Array => {
                if let Ok(mut r) = self.binary.get_reader_at(ty.array_type_ptr())
                    && let Ok(arr_ty) = Il2CppArrayType::decode(&mut r)
                    && let Some(elem_ty) = self.get_il2cpp_type(arr_ty.etype)
                {
                    return format!(
                        "{}[{}]",
                        self.get_type_name(elem_ty, add_namespace, false),
                        ",".repeat(arr_ty.rank as usize - 1)
                    );
                }
                "object[]".to_string()
            }
            Il2CppTypeEnum::SzArray => {
                if let Some(elem_ty) = self.get_il2cpp_type(ty.type_handle()) {
                    return format!("{}[]", self.get_type_name(elem_ty, add_namespace, false));
                }
                "object[]".to_string()
            }
            Il2CppTypeEnum::Ptr => {
                if let Some(elem_ty) = self.get_il2cpp_type(ty.type_handle()) {
                    return format!("{}*", self.get_type_name(elem_ty, add_namespace, false));
                }
                "void*".to_string()
            }
            Il2CppTypeEnum::Var | Il2CppTypeEnum::MVar => {
                if let Some(param) = self.get_generic_parameter_from_type(ty) {
                    return self.metadata.get_string_from_index(param.name_index);
                }
                "T".to_string()
            }
            Il2CppTypeEnum::Class | Il2CppTypeEnum::ValueType | Il2CppTypeEnum::GenericInst => {
                let mut type_def_opt = None;
                let mut generic_class_opt = None;
                if type_enum == Il2CppTypeEnum::GenericInst {
                    if let Ok(mut r) = self.binary.get_reader_at(ty.generic_class_ptr())
                        && let Ok(gc) = Il2CppGenericClass::decode(&mut r, self.metadata.version)
                    {
                        type_def_opt = self.get_generic_class_type_definition(&gc);
                        generic_class_opt = Some(gc);
                    }
                } else {
                    let index = ty.klass_index() as usize;
                    type_def_opt = self.metadata.type_defs.get(index);
                }

                if let Some(type_def) = type_def_opt {
                    let mut prefix = String::new();
                    if type_def.declaring_type_index != -1 {
                        if let Some(declaring_type) =
                            self.types.get(type_def.declaring_type_index as usize)
                        {
                            let decl_name = self.get_type_name(declaring_type, add_namespace, true);
                            prefix = format!("{}.", decl_name);
                        }
                    } else if add_namespace {
                        let ns = self
                            .metadata
                            .get_string_from_index(type_def.namespace_index);
                        if !ns.is_empty() {
                            prefix = format!("{}.", ns);
                        }
                    }

                    let mut name = self.metadata.get_string_from_index(type_def.name_index);
                    if let Some(idx) = name.find('`') {
                        name = name[..idx].to_string();
                    }

                    if is_nested {
                        return format!("{}{}", prefix, name);
                    }

                    if generic_class_opt.is_some() {
                        // Append generic arguments, e.g. List<int>
                        let mut gen_args = Vec::new();
                        if let Some(gc) = generic_class_opt
                            && let Ok(mut r_inst) = self.binary.get_reader_at(gc.context.class_inst)
                            && let Ok(inst) = Il2CppGenericInst::decode(&mut r_inst)
                            && let Ok(argv) = read_ptr_array(
                                &self.binary,
                                inst.type_argv,
                                inst.type_argc as usize,
                            )
                        {
                            for ptr in argv {
                                if let Some(arg_ty) = self.get_il2cpp_type(ptr) {
                                    gen_args.push(self.get_type_name(arg_ty, add_namespace, false));
                                }
                            }
                        }
                        if !gen_args.is_empty() {
                            name = format!("{}<{}>", name, gen_args.join(", "));
                        }
                    } else if type_def.generic_container_index >= 0
                        && let Some(gc) = self
                            .metadata
                            .generic_containers
                            .get(type_def.generic_container_index as usize)
                    {
                        name = format!("{}{}", name, self.get_generic_container_params(gc));
                    }

                    format!("{}{}", prefix, name)
                } else {
                    "object".to_string()
                }
            }
            _ => {
                let code = ty.ty;
                if let Some(&name) = TYPE_STRING.get(&code) {
                    name.to_string()
                } else {
                    "object".to_string()
                }
            }
        }
    }

    pub fn get_type_name_from_def(
        &self,
        type_def: &crate::metadata::Il2CppTypeDefinition,
        add_namespace: bool,
        is_nested: bool,
    ) -> String {
        self.get_type_name_from_def_generic(type_def, add_namespace, is_nested, true)
    }

    pub fn get_type_name_from_def_generic(
        &self,
        type_def: &crate::metadata::Il2CppTypeDefinition,
        add_namespace: bool,
        is_nested: bool,
        include_generic: bool,
    ) -> String {
        let mut name = self.metadata.get_string_from_index(type_def.name_index);
        if let Some(idx) = name.find('`') {
            name = name[..idx].to_string();
        }
        if include_generic
            && type_def.generic_container_index >= 0
            && let Some(gc) = self
                .metadata
                .generic_containers
                .get(type_def.generic_container_index as usize)
        {
            name = format!("{}{}", name, self.get_generic_container_params(gc));
        }
        if is_nested {
            return name;
        }
        let mut prefix = String::new();
        if type_def.declaring_type_index != -1 {
            if let Some(declaring_type) = self.types.get(type_def.declaring_type_index as usize) {
                let decl_name = self.get_type_name(declaring_type, add_namespace, true);
                prefix = format!("{}.", decl_name);
            }
        } else if add_namespace {
            let ns = self
                .metadata
                .get_string_from_index(type_def.namespace_index);
            if !ns.is_empty() {
                prefix = format!("{}.", ns);
            }
        }
        format!("{}{}", prefix, name)
    }

    pub fn get_generic_inst_params(&self, generic_inst: &Il2CppGenericInst) -> String {
        let mut gen_args = Vec::new();
        if let Ok(argv) = read_ptr_array(
            &self.binary,
            generic_inst.type_argv,
            generic_inst.type_argc as usize,
        ) {
            for ptr in argv {
                if let Some(arg_ty) = self.get_il2cpp_type(ptr) {
                    gen_args.push(self.get_type_name(arg_ty, false, false));
                }
            }
        }
        format!("<{}>", gen_args.join(", "))
    }

    pub fn get_generic_container_params(
        &self,
        generic_container: &crate::metadata::Il2CppGenericContainer,
    ) -> String {
        let mut names = Vec::new();
        for i in 0..generic_container.type_argc {
            let idx = generic_container.generic_parameter_start as usize + i as usize;
            if idx < self.metadata.generic_parameters.len() {
                let gp = &self.metadata.generic_parameters[idx];
                let gp_name = self.metadata.get_string_from_index(gp.name_index);
                names.push(gp_name);
            }
        }
        if names.is_empty() {
            String::new()
        } else {
            format!("<{}>", names.join(", "))
        }
    }

    pub fn get_method_spec_name(
        &self,
        method_spec: &crate::il2cpp_binary_structures::Il2CppMethodSpec,
        add_namespace: bool,
    ) -> (String, String) {
        let method_def = &self.metadata.method_defs[method_spec.method_definition_index as usize];
        let type_def = &self.metadata.type_defs[method_def.declaring_type as usize];
        let mut type_name = self.get_type_name_from_def_generic(
            type_def,
            add_namespace,
            false,
            method_spec.class_index_index == -1,
        );
        if method_spec.class_index_index != -1
            && let Some(class_inst) = self
                .generic_insts
                .get(method_spec.class_index_index as usize)
        {
            type_name += &self.get_generic_inst_params(class_inst);
        }
        let mut method_name = self.metadata.get_string_from_index(method_def.name_index);
        if method_spec.method_index_index != -1
            && let Some(method_inst) = self
                .generic_insts
                .get(method_spec.method_index_index as usize)
        {
            method_name += &self.get_generic_inst_params(method_inst);
        }
        (type_name, method_name)
    }

    pub fn get_type_definition_from_il2cpp_type(
        &self,
        ty: &crate::il2cpp_binary_structures::Il2CppType,
    ) -> Option<&crate::metadata::Il2CppTypeDefinition> {
        match ty.type_enum() {
            Il2CppTypeEnum::Class | Il2CppTypeEnum::ValueType => {
                let index = ty.klass_index() as usize;
                self.metadata.type_defs.get(index)
            }
            Il2CppTypeEnum::GenericInst => {
                if let Ok(mut r) = self.binary.get_reader_at(ty.generic_class_ptr())
                    && let Ok(gc) = crate::il2cpp_binary_structures::Il2CppGenericClass::decode(
                        &mut r,
                        self.metadata.version,
                    )
                {
                    return self.get_generic_class_type_definition(&gc);
                }
                None
            }
            _ => None,
        }
    }

    pub fn get_image_name_for_type(
        &self,
        type_def: &crate::metadata::Il2CppTypeDefinition,
    ) -> Option<String> {
        let type_def_idx = self
            .metadata
            .type_defs
            .iter()
            .position(|x| std::ptr::eq(x, type_def))? as i32;
        for image_def in &self.metadata.image_defs {
            let start = image_def.type_start;
            let end = start + image_def.type_count as i32;
            if type_def_idx >= start && type_def_idx < end {
                return Some(
                    self.metadata
                        .get_string_from_index(image_name_index_fallback(image_def)),
                );
            }
        }
        None
    }

    pub fn get_default_value(&self, type_index: i32, data_index: i32) -> Option<String> {
        if data_index == -1 {
            return None;
        }

        let pointer = self
            .metadata
            .header
            .field_and_parameter_default_value_data_offset as usize
            + data_index as usize;
        if pointer >= self.metadata.raw_bytes.len() {
            return None;
        }

        let default_value_type = self.types.get(type_index as usize)?;

        let cursor = std::io::Cursor::new(&self.metadata.raw_bytes[pointer..]);
        let mut r = crate::binary_reader::BinaryReader::new(
            cursor,
            self.binary.is_32bit,
            self.binary.endian,
        );

        match default_value_type.type_enum() {
            Il2CppTypeEnum::Boolean => {
                if let Ok(b) = r.read_u8() {
                    return Some(if b != 0 {
                        "True".to_string()
                    } else {
                        "False".to_string()
                    });
                }
            }
            Il2CppTypeEnum::I1 => {
                if let Ok(b) = r.read_i8() {
                    return Some(format!("{}", b));
                }
            }
            Il2CppTypeEnum::U1 => {
                if let Ok(b) = r.read_u8() {
                    return Some(format!("{}", b));
                }
            }
            Il2CppTypeEnum::I2 => {
                if let Ok(b) = r.read_i16() {
                    return Some(format!("{}", b));
                }
            }
            Il2CppTypeEnum::U2 => {
                if let Ok(b) = r.read_u16() {
                    return Some(format!("{}", b));
                }
            }
            Il2CppTypeEnum::I4 => {
                if self.metadata.version >= 29.0 {
                    if let Ok(b) = r.read_compressed_int32() {
                        return Some(format!("{}", b));
                    }
                } else {
                    if let Ok(b) = r.read_i32() {
                        return Some(format!("{}", b));
                    }
                }
            }
            Il2CppTypeEnum::U4 => {
                if self.metadata.version >= 29.0 {
                    if let Ok(b) = r.read_compressed_uint32() {
                        return Some(format!("{}", b));
                    }
                } else {
                    if let Ok(b) = r.read_u32() {
                        return Some(format!("{}", b));
                    }
                }
            }
            Il2CppTypeEnum::I8 => {
                if let Ok(b) = r.read_i64() {
                    return Some(format!("{}", b));
                }
            }
            Il2CppTypeEnum::U8 => {
                if let Ok(b) = r.read_u64() {
                    return Some(format!("{}", b));
                }
            }
            Il2CppTypeEnum::R4 => {
                if let Ok(b) = r.read_f32() {
                    return Some(format!("{}", b));
                }
            }
            Il2CppTypeEnum::R8 => {
                if let Ok(b) = r.read_f64() {
                    return Some(format!("{}", b));
                }
            }
            Il2CppTypeEnum::String => {
                use std::io::Read;
                if self.metadata.version >= 29.0 {
                    if let Ok(len) = r.read_compressed_int32() {
                        if len == -1 {
                            return Some("null".to_string());
                        }
                        let mut buf = vec![0u8; len as usize];
                        if r.read_exact(&mut buf).is_ok()
                            && let Ok(s) = String::from_utf8(buf)
                        {
                            return Some(format!("\"{}\"", s.escape_default()));
                        }
                    }
                } else {
                    if let Ok(len) = r.read_i32() {
                        let mut buf = vec![0u8; len as usize];
                        if r.read_exact(&mut buf).is_ok()
                            && let Ok(s) = String::from_utf8(buf)
                        {
                            return Some(format!("\"{}\"", s.escape_default()));
                        }
                    }
                }
            }
            _ => {}
        }

        None
    }
}

fn read_ptr_array(binary: &BinaryFile, addr: u64, count: usize) -> io::Result<Vec<u64>> {
    if count == 0 || addr == 0 {
        return Ok(Vec::new());
    }
    let mut r = binary.get_reader_at(addr)?;
    let mut vec = Vec::with_capacity(count);
    for _ in 0..count {
        vec.push(r.read_ptr()?);
    }
    Ok(vec)
}

fn image_name_index_fallback(image_def: &crate::metadata::Il2CppImageDefinition) -> u32 {
    image_def.name_index
}
