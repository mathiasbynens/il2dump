use byteorder::{LittleEndian, WriteBytesExt};
use std::env;
use std::fs;
use std::io::Cursor;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

// Run the dumper integration end-to-end.
fn run_dumper(exec_bytes: &[u8], metadata_bytes: &[u8]) -> Result<(String, String), String> {
    let metadata = il2dump_lib::metadata::Metadata::load(metadata_bytes.to_vec())
        .map_err(|e| format!("Metadata load error: {}", e))?;

    let binary_file = il2dump_lib::binary::BinaryFile::parse(exec_bytes.to_vec())
        .map_err(|e| format!("Binary parse error: {}", e))?;

    let version = metadata.version;

    let method_count = metadata
        .method_defs
        .iter()
        .filter(|x| x.method_index >= 0)
        .count();
    let (code_reg, metadata_reg) = binary_file.plus_search(
        version,
        method_count,
        metadata.type_defs.len(),
        metadata.metadata_usages_count,
        metadata.image_defs.len(),
    );

    if code_reg == 0 || metadata_reg == 0 {
        return Err(format!(
            "Failed to locate registration structures via heuristic search (code_reg: 0x{:x}, metadata_reg: 0x{:x})",
            code_reg, metadata_reg
        ));
    }

    let executor = il2dump_lib::il2cpp_executor::Il2CppExecutor::new(
        metadata,
        binary_file,
        code_reg,
        metadata_reg,
    )
    .map_err(|e| format!("Executor init error: {}", e))?;

    let decompiler = il2dump_lib::decompiler::Decompiler::new(&executor);
    let config = il2dump_lib::config::Config::default();

    decompiler
        .decompile_to_memory(&config)
        .map_err(|e| format!("Decompilation error: {}", e))
}

// Helper to build string literals.
fn write_string(writer: &mut Vec<u8>, s: &str) -> u32 {
    let offset = writer.len() as u32;
    writer.extend_from_slice(s.as_bytes());
    writer.push(0); // Null terminator
    offset
}

#[test]
fn test_end_to_end_mock() {
    // 1. Compile mock_il2cpp.c to a shared library.
    let target_dir =
        PathBuf::from(env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string()));
    let out_dir = target_dir.join("debug");
    fs::create_dir_all(&out_dir).unwrap();

    let lib_name = if cfg!(target_os = "windows") {
        "mockil2cpp.exe"
    } else {
        "mockil2cpp"
    };
    let lib_path = out_dir.join(lib_name);

    let mut compile_cmd = Command::new("cc");
    compile_cmd.arg("-o");
    compile_cmd.arg(lib_path.to_str().unwrap());
    compile_cmd.arg("tests/fixtures/mock_il2cpp.c");
    if cfg!(target_os = "macos") {
        compile_cmd.arg("-Wl,-no_fixup_chains");
    }

    let compile_output = compile_cmd.output().expect("Failed to run compiler (cc)");
    if !compile_output.status.success() {
        panic!(
            "Compilation of mock C library failed:\nSTDOUT:\n{}\nSTDERR:\n{}",
            String::from_utf8_lossy(&compile_output.stdout),
            String::from_utf8_lossy(&compile_output.stderr)
        );
    }

    // Read the compiled mock binary bytes.
    let exec_bytes = fs::read(&lib_path).unwrap();

    // 2. Programmatically generate mock global-metadata.dat bytes.
    let mut metadata_writer = Vec::new();
    let mut string_writer = Vec::new();

    // Write assembly, image, namespace, type, and method names to the string literal table.
    let mscorlib_dll_offset = write_string(&mut string_writer, "mscorlib.dll");
    let mscorlib_offset = write_string(&mut string_writer, "mscorlib");
    let test_namespace_offset = write_string(&mut string_writer, "TestNamespace");
    let test_class_offset = write_string(&mut string_writer, "TestClass");
    let test_method_offset = write_string(&mut string_writer, "TestMethod");

    // Build the header.
    // Offset tables list:
    let string_offset = 500; // Place string table at offset 500.
    let string_size = string_writer.len() as i32;

    let images_offset = string_offset + string_size as u32;
    let images_size = 40; // 1 image definition (size = 40 bytes in v29)

    let assemblies_offset = images_offset + images_size as u32;
    let assemblies_size = 64; // 1 assembly definition (size = 64 bytes in v29)

    let type_definitions_offset = assemblies_offset + assemblies_size as u32;
    let type_definitions_size = 88 * 2; // 2 type definitions (88 bytes each in v29)

    let methods_offset = type_definitions_offset + type_definitions_size as u32;
    let methods_size = 32 * 2; // 2 method definitions (32 bytes each in v29)

    // Write header fields sequentially:
    let mut header = Cursor::new(Vec::new());
    header.write_u32::<LittleEndian>(0xFAB11BAF).unwrap(); // sanity
    header.write_i32::<LittleEndian>(29).unwrap(); // version
    header.write_u32::<LittleEndian>(0).unwrap(); // string_literal_offset
    header.write_i32::<LittleEndian>(0).unwrap(); // string_literal_size
    header.write_u32::<LittleEndian>(0).unwrap(); // string_literal_data_offset
    header.write_i32::<LittleEndian>(0).unwrap(); // string_literal_data_size
    header.write_u32::<LittleEndian>(string_offset).unwrap();
    header.write_i32::<LittleEndian>(string_size).unwrap();
    header.write_u32::<LittleEndian>(0).unwrap(); // events_offset
    header.write_i32::<LittleEndian>(0).unwrap(); // events_size
    header.write_u32::<LittleEndian>(0).unwrap(); // properties_offset
    header.write_i32::<LittleEndian>(0).unwrap(); // properties_size
    header.write_u32::<LittleEndian>(methods_offset).unwrap();
    header.write_i32::<LittleEndian>(methods_size).unwrap();
    header.write_u32::<LittleEndian>(0).unwrap(); // parameter_default_values_offset
    header.write_i32::<LittleEndian>(0).unwrap(); // parameter_default_values_size
    header.write_u32::<LittleEndian>(0).unwrap(); // field_default_values_offset
    header.write_i32::<LittleEndian>(0).unwrap(); // field_default_values_size
    header.write_u32::<LittleEndian>(0).unwrap(); // field_and_parameter_default_value_data_offset
    header.write_i32::<LittleEndian>(0).unwrap(); // field_and_parameter_default_value_data_size
    header.write_i32::<LittleEndian>(0).unwrap(); // field_marshaled_sizes_offset
    header.write_i32::<LittleEndian>(0).unwrap(); // field_marshaled_sizes_size
    header.write_u32::<LittleEndian>(0).unwrap(); // parameters_offset
    header.write_i32::<LittleEndian>(0).unwrap(); // parameters_size
    header.write_u32::<LittleEndian>(0).unwrap(); // fields_offset
    header.write_i32::<LittleEndian>(0).unwrap(); // fields_size
    header.write_u32::<LittleEndian>(0).unwrap(); // generic_parameters_offset
    header.write_i32::<LittleEndian>(0).unwrap(); // generic_parameters_size
    header.write_u32::<LittleEndian>(0).unwrap(); // generic_parameter_constraints_offset
    header.write_i32::<LittleEndian>(0).unwrap(); // generic_parameter_constraints_size
    header.write_u32::<LittleEndian>(0).unwrap(); // generic_containers_offset
    header.write_i32::<LittleEndian>(0).unwrap(); // generic_containers_size
    header.write_u32::<LittleEndian>(0).unwrap(); // nested_types_offset
    header.write_i32::<LittleEndian>(0).unwrap(); // nested_types_size
    header.write_u32::<LittleEndian>(0).unwrap(); // interfaces_offset
    header.write_i32::<LittleEndian>(0).unwrap(); // interfaces_size
    header.write_u32::<LittleEndian>(0).unwrap(); // vtable_methods_offset
    header.write_i32::<LittleEndian>(0).unwrap(); // vtable_methods_size
    header.write_i32::<LittleEndian>(0).unwrap(); // interface_offsets_offset
    header.write_i32::<LittleEndian>(0).unwrap(); // interface_offsets_size
    header
        .write_u32::<LittleEndian>(type_definitions_offset)
        .unwrap();
    header
        .write_i32::<LittleEndian>(type_definitions_size)
        .unwrap();
    header.write_u32::<LittleEndian>(images_offset).unwrap();
    header.write_i32::<LittleEndian>(images_size).unwrap();
    header.write_u32::<LittleEndian>(assemblies_offset).unwrap();
    header.write_i32::<LittleEndian>(assemblies_size).unwrap();
    header.write_u32::<LittleEndian>(0).unwrap(); // field_refs_offset
    header.write_i32::<LittleEndian>(0).unwrap(); // field_refs_size
    header.write_i32::<LittleEndian>(0).unwrap(); // referenced_assemblies_offset
    header.write_i32::<LittleEndian>(0).unwrap(); // referenced_assemblies_size
    header.write_u32::<LittleEndian>(0).unwrap(); // attribute_data_offset
    header.write_i32::<LittleEndian>(0).unwrap(); // attribute_data_size
    header.write_u32::<LittleEndian>(0).unwrap(); // attribute_data_range_offset
    header.write_i32::<LittleEndian>(0).unwrap(); // attribute_data_range_size
    header.write_i32::<LittleEndian>(0).unwrap(); // unresolved_virtual_call_parameter_types_offset
    header.write_i32::<LittleEndian>(0).unwrap(); // unresolved_virtual_call_parameter_types_size
    header.write_i32::<LittleEndian>(0).unwrap(); // unresolved_virtual_call_parameter_ranges_offset
    header.write_i32::<LittleEndian>(0).unwrap(); // unresolved_virtual_call_parameter_ranges_size
    header.write_i32::<LittleEndian>(0).unwrap(); // windows_runtime_type_names_offset
    header.write_i32::<LittleEndian>(0).unwrap(); // windows_runtime_type_names_size
    header.write_i32::<LittleEndian>(0).unwrap(); // windows_runtime_strings_offset
    header.write_i32::<LittleEndian>(0).unwrap(); // windows_runtime_strings_size
    header.write_i32::<LittleEndian>(0).unwrap(); // exported_type_definitions_offset
    header.write_i32::<LittleEndian>(0).unwrap(); // exported_type_definitions_size

    let header_bytes = header.into_inner();
    metadata_writer.extend(header_bytes);

    // Pad out to string table offset.
    while metadata_writer.len() < string_offset as usize {
        metadata_writer.push(0);
    }
    metadata_writer.extend(string_writer);

    // Pad to images offset.
    while metadata_writer.len() < images_offset as usize {
        metadata_writer.push(0);
    }
    // Write Image Definition (v29).
    let mut img = Cursor::new(Vec::new());
    img.write_u32::<LittleEndian>(mscorlib_dll_offset).unwrap(); // name_index
    img.write_i32::<LittleEndian>(0).unwrap(); // assembly_index
    img.write_i32::<LittleEndian>(0).unwrap(); // type_start
    img.write_u32::<LittleEndian>(2).unwrap(); // type_count
    img.write_i32::<LittleEndian>(0).unwrap(); // exported_type_start
    img.write_u32::<LittleEndian>(0).unwrap(); // exported_type_count
    img.write_i32::<LittleEndian>(0).unwrap(); // entry_point_index
    img.write_u32::<LittleEndian>(1).unwrap(); // token
    img.write_i32::<LittleEndian>(0).unwrap(); // custom_attribute_start
    img.write_u32::<LittleEndian>(0).unwrap(); // custom_attribute_count
    metadata_writer.extend(img.into_inner());

    // Pad to assemblies offset.
    while metadata_writer.len() < assemblies_offset as usize {
        metadata_writer.push(0);
    }
    // Write Assembly Definition (v29).
    let mut asm = Cursor::new(Vec::new());
    asm.write_i32::<LittleEndian>(0).unwrap(); // image_index
    asm.write_u32::<LittleEndian>(1).unwrap(); // token
    asm.write_i32::<LittleEndian>(0).unwrap(); // referenced_assembly_start
    asm.write_i32::<LittleEndian>(0).unwrap(); // referenced_assembly_count
    // aname:
    asm.write_u32::<LittleEndian>(mscorlib_offset).unwrap(); // name_index
    asm.write_u32::<LittleEndian>(0).unwrap(); // culture_index
    asm.write_u32::<LittleEndian>(0).unwrap(); // public_key_index
    asm.write_u32::<LittleEndian>(0).unwrap(); // hash_alg
    asm.write_i32::<LittleEndian>(0).unwrap(); // hash_len
    asm.write_u32::<LittleEndian>(0).unwrap(); // flags
    asm.write_i32::<LittleEndian>(1).unwrap(); // major
    asm.write_i32::<LittleEndian>(0).unwrap(); // minor
    asm.write_i32::<LittleEndian>(0).unwrap(); // build
    asm.write_i32::<LittleEndian>(0).unwrap(); // revision
    asm.write_all(&[0u8; 8]).unwrap(); // public_key_token
    metadata_writer.extend(asm.into_inner());

    // Pad to type definitions offset.
    while metadata_writer.len() < type_definitions_offset as usize {
        metadata_writer.push(0);
    }
    // Type 1: dummy <Module>
    let mut type1 = Cursor::new(Vec::new());
    type1.write_u32::<LittleEndian>(0).unwrap(); // name_index
    type1.write_u32::<LittleEndian>(0).unwrap(); // namespace_index
    type1.write_i32::<LittleEndian>(0).unwrap(); // byval_type_index
    type1.write_i32::<LittleEndian>(-1).unwrap(); // declaring_type_index
    type1.write_i32::<LittleEndian>(-1).unwrap(); // parent_index
    type1.write_i32::<LittleEndian>(-1).unwrap(); // element_type_index
    type1.write_i32::<LittleEndian>(-1).unwrap(); // generic_container_index
    type1.write_u32::<LittleEndian>(0).unwrap(); // flags
    // starts & counts:
    for _ in 0..8 {
        type1.write_i32::<LittleEndian>(0).unwrap(); // field_start, method_start etc
    }
    for _ in 8..16 {
        type1.write_u16::<LittleEndian>(0).unwrap(); // counts
    }
    type1.write_u32::<LittleEndian>(0).unwrap(); // bitfield
    type1.write_u32::<LittleEndian>(1).unwrap(); // token
    metadata_writer.extend(type1.into_inner());

    // Type 2: TestClass
    let mut type2 = Cursor::new(Vec::new());
    type2.write_u32::<LittleEndian>(test_class_offset).unwrap();
    type2
        .write_u32::<LittleEndian>(test_namespace_offset)
        .unwrap();
    type2.write_i32::<LittleEndian>(1).unwrap(); // byval_type_index
    type2.write_i32::<LittleEndian>(-1).unwrap(); // declaring_type_index
    type2.write_i32::<LittleEndian>(-1).unwrap(); // parent_index
    type2.write_i32::<LittleEndian>(-1).unwrap(); // element_type_index
    type2.write_i32::<LittleEndian>(-1).unwrap(); // generic_container_index
    type2
        .write_u32::<LittleEndian>(0x00000001 | 0x00100000)
        .unwrap(); // public, before-field-init flags
    // starts & counts:
    type2.write_i32::<LittleEndian>(0).unwrap(); // field_start
    type2.write_i32::<LittleEndian>(1).unwrap(); // method_start (starts at method index 1)
    type2.write_i32::<LittleEndian>(0).unwrap(); // event_start
    type2.write_i32::<LittleEndian>(0).unwrap(); // property_start
    type2.write_i32::<LittleEndian>(0).unwrap(); // nested_types_start
    type2.write_i32::<LittleEndian>(0).unwrap(); // interfaces_start
    type2.write_i32::<LittleEndian>(0).unwrap(); // vtable_start
    type2.write_i32::<LittleEndian>(0).unwrap(); // interface_offsets_start
    type2.write_u16::<LittleEndian>(1).unwrap(); // method_count (1 method)
    type2.write_u16::<LittleEndian>(0).unwrap(); // property_count
    type2.write_u16::<LittleEndian>(0).unwrap(); // field_count
    type2.write_u16::<LittleEndian>(0).unwrap(); // event_count
    type2.write_u16::<LittleEndian>(0).unwrap(); // nested_type_count
    type2.write_u16::<LittleEndian>(0).unwrap(); // vtable_count
    type2.write_u16::<LittleEndian>(0).unwrap(); // interfaces_count
    type2.write_u16::<LittleEndian>(0).unwrap(); // interface_offsets_count
    type2.write_u32::<LittleEndian>(0).unwrap(); // bitfield
    type2.write_u32::<LittleEndian>(2).unwrap(); // token
    metadata_writer.extend(type2.into_inner());

    // Pad to methods offset.
    while metadata_writer.len() < methods_offset as usize {
        metadata_writer.push(0);
    }
    // Method 1: dummy
    let mut method1 = Cursor::new(Vec::new());
    method1.write_u32::<LittleEndian>(0).unwrap(); // name_index
    method1.write_i32::<LittleEndian>(0).unwrap(); // declaring_type
    method1.write_i32::<LittleEndian>(0).unwrap(); // return_type
    method1.write_i32::<LittleEndian>(0).unwrap(); // parameter_start
    method1.write_u32::<LittleEndian>(0).unwrap(); // generic_container
    method1.write_u32::<LittleEndian>(0).unwrap(); // token
    method1.write_u16::<LittleEndian>(0).unwrap(); // flags
    method1.write_u16::<LittleEndian>(0).unwrap(); // iflags
    method1.write_u16::<LittleEndian>(0).unwrap(); // slot
    method1.write_u16::<LittleEndian>(0).unwrap(); // parameter_count
    metadata_writer.extend(method1.into_inner());

    // Method 2: TestMethod
    let mut method2 = Cursor::new(Vec::new());
    method2
        .write_u32::<LittleEndian>(test_method_offset)
        .unwrap();
    method2.write_i32::<LittleEndian>(1).unwrap(); // declaring_type = 1 (TestClass)
    method2.write_i32::<LittleEndian>(0).unwrap(); // return_type = void
    method2.write_i32::<LittleEndian>(0).unwrap(); // parameter_start
    method2.write_u32::<LittleEndian>(0).unwrap(); // generic_container
    method2.write_u32::<LittleEndian>(0).unwrap(); // token
    method2.write_u16::<LittleEndian>(0x0006).unwrap(); // public | virtual
    method2.write_u16::<LittleEndian>(0).unwrap(); // iflags
    method2.write_u16::<LittleEndian>(0).unwrap(); // slot
    method2.write_u16::<LittleEndian>(0).unwrap(); // parameter_count = 0
    metadata_writer.extend(method2.into_inner());

    // 3. Run dumper.
    let (dump_cs, _script_json) = run_dumper(&exec_bytes, &metadata_writer).unwrap();

    // 4. Assertions on generated C# dump.
    assert!(dump_cs.contains("Namespace: TestNamespace"));
    assert!(dump_cs.contains("public class TestClass"));
    assert!(dump_cs.contains("public void TestMethod()"));
}
