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
    writer.push(0); // Null terminator.
    offset
}

#[test]
fn test_end_to_end_mock() {
    // 1. Compile mock_il2cpp.c.
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
    // Offset tables list.
    let string_offset = 500; // Place string table at offset 500.
    let string_size = string_writer.len() as i32;

    let images_offset = string_offset + string_size as u32;
    let images_size = 40; // 1 image definition (size = 40 bytes in v29).

    let assemblies_offset = images_offset + images_size as u32;
    let assemblies_size = 64; // 1 assembly definition (size = 64 bytes in v29).

    let type_definitions_offset = assemblies_offset + assemblies_size as u32;
    let type_definitions_size = 88 * 2; // 2 type definitions (88 bytes each in v29).

    let methods_offset = type_definitions_offset + type_definitions_size as u32;
    let methods_size = 32 * 2; // 2 method definitions (32 bytes each in v29).

    // Write header fields sequentially.
    let mut header = Cursor::new(Vec::new());
    header.write_u32::<LittleEndian>(0xFAB11BAF).unwrap(); // Sanity.
    header.write_i32::<LittleEndian>(29).unwrap(); // Version.
    header.write_u32::<LittleEndian>(0).unwrap(); // String literal offset.
    header.write_i32::<LittleEndian>(0).unwrap(); // String literal size.
    header.write_u32::<LittleEndian>(0).unwrap(); // String literal data offset.
    header.write_i32::<LittleEndian>(0).unwrap(); // String literal data size.
    header.write_u32::<LittleEndian>(string_offset).unwrap();
    header.write_i32::<LittleEndian>(string_size).unwrap();
    header.write_u32::<LittleEndian>(0).unwrap(); // Events offset.
    header.write_i32::<LittleEndian>(0).unwrap(); // Events size.
    header.write_u32::<LittleEndian>(0).unwrap(); // Properties offset.
    header.write_i32::<LittleEndian>(0).unwrap(); // Properties size.
    header.write_u32::<LittleEndian>(methods_offset).unwrap();
    header.write_i32::<LittleEndian>(methods_size).unwrap();
    header.write_u32::<LittleEndian>(0).unwrap(); // Parameter default values offset.
    header.write_i32::<LittleEndian>(0).unwrap(); // Parameter default values size.
    header.write_u32::<LittleEndian>(0).unwrap(); // Field default values offset.
    header.write_i32::<LittleEndian>(0).unwrap(); // Field default values size.
    header.write_u32::<LittleEndian>(0).unwrap(); // Field and parameter default value data offset.
    header.write_i32::<LittleEndian>(0).unwrap(); // Field and parameter default value data size.
    header.write_i32::<LittleEndian>(0).unwrap(); // Field marshaled sizes offset.
    header.write_i32::<LittleEndian>(0).unwrap(); // Field marshaled sizes size.
    header.write_u32::<LittleEndian>(0).unwrap(); // Parameters offset.
    header.write_i32::<LittleEndian>(0).unwrap(); // Parameters size.
    header.write_u32::<LittleEndian>(0).unwrap(); // Fields offset.
    header.write_i32::<LittleEndian>(0).unwrap(); // Fields size.
    header.write_u32::<LittleEndian>(0).unwrap(); // Generic parameters offset.
    header.write_i32::<LittleEndian>(0).unwrap(); // Generic parameters size.
    header.write_u32::<LittleEndian>(0).unwrap(); // Generic parameter constraints offset.
    header.write_i32::<LittleEndian>(0).unwrap(); // Generic parameter constraints size.
    header.write_u32::<LittleEndian>(0).unwrap(); // Generic containers offset.
    header.write_i32::<LittleEndian>(0).unwrap(); // Generic containers size.
    header.write_u32::<LittleEndian>(0).unwrap(); // Nested types offset.
    header.write_i32::<LittleEndian>(0).unwrap(); // Nested types size.
    header.write_u32::<LittleEndian>(0).unwrap(); // Interfaces offset.
    header.write_i32::<LittleEndian>(0).unwrap(); // Interfaces size.
    header.write_u32::<LittleEndian>(0).unwrap(); // Vtable methods offset.
    header.write_i32::<LittleEndian>(0).unwrap(); // Vtable methods size.
    header.write_i32::<LittleEndian>(0).unwrap(); // Interface offsets offset.
    header.write_i32::<LittleEndian>(0).unwrap(); // Interface offsets size.
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
    header.write_u32::<LittleEndian>(0).unwrap(); // Field refs offset.
    header.write_i32::<LittleEndian>(0).unwrap(); // Field refs size.
    header.write_i32::<LittleEndian>(0).unwrap(); // Referenced assemblies offset.
    header.write_i32::<LittleEndian>(0).unwrap(); // Referenced assemblies size.
    header.write_u32::<LittleEndian>(0).unwrap(); // Attribute data offset.
    header.write_i32::<LittleEndian>(0).unwrap(); // Attribute data size.
    header.write_u32::<LittleEndian>(0).unwrap(); // Attribute data range offset.
    header.write_i32::<LittleEndian>(0).unwrap(); // Attribute data range size.
    header.write_i32::<LittleEndian>(0).unwrap(); // Unresolved virtual call parameter types offset.
    header.write_i32::<LittleEndian>(0).unwrap(); // Unresolved virtual call parameter types size.
    header.write_i32::<LittleEndian>(0).unwrap(); // Unresolved virtual call parameter ranges offset.
    header.write_i32::<LittleEndian>(0).unwrap(); // Unresolved virtual call parameter ranges size.
    header.write_i32::<LittleEndian>(0).unwrap(); // Windows runtime type names offset.
    header.write_i32::<LittleEndian>(0).unwrap(); // Windows runtime type names size.
    header.write_i32::<LittleEndian>(0).unwrap(); // Windows runtime strings offset.
    header.write_i32::<LittleEndian>(0).unwrap(); // Windows runtime strings size.
    header.write_i32::<LittleEndian>(0).unwrap(); // Exported type definitions offset.
    header.write_i32::<LittleEndian>(0).unwrap(); // Exported type definitions size.

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
    // Write image definition (v29).
    let mut img = Cursor::new(Vec::new());
    img.write_u32::<LittleEndian>(mscorlib_dll_offset).unwrap(); // Name index.
    img.write_i32::<LittleEndian>(0).unwrap(); // Assembly index.
    img.write_i32::<LittleEndian>(0).unwrap(); // Type start.
    img.write_u32::<LittleEndian>(2).unwrap(); // Type count.
    img.write_i32::<LittleEndian>(0).unwrap(); // Exported type start.
    img.write_u32::<LittleEndian>(0).unwrap(); // Exported type count.
    img.write_i32::<LittleEndian>(0).unwrap(); // Entry point index.
    img.write_u32::<LittleEndian>(1).unwrap(); // Token.
    img.write_i32::<LittleEndian>(0).unwrap(); // Custom attribute start.
    img.write_u32::<LittleEndian>(0).unwrap(); // Custom attribute count.
    metadata_writer.extend(img.into_inner());

    // Pad to assemblies offset.
    while metadata_writer.len() < assemblies_offset as usize {
        metadata_writer.push(0);
    }
    // Write assembly definition (v29).
    let mut asm = Cursor::new(Vec::new());
    asm.write_i32::<LittleEndian>(0).unwrap(); // Image index.
    asm.write_u32::<LittleEndian>(1).unwrap(); // Token.
    asm.write_i32::<LittleEndian>(0).unwrap(); // Referenced assembly start.
    asm.write_i32::<LittleEndian>(0).unwrap(); // Referenced assembly count.
    // aname:
    asm.write_u32::<LittleEndian>(mscorlib_offset).unwrap(); // Name index.
    asm.write_u32::<LittleEndian>(0).unwrap(); // Culture index.
    asm.write_u32::<LittleEndian>(0).unwrap(); // Public key index.
    asm.write_u32::<LittleEndian>(0).unwrap(); // Hash alg.
    asm.write_i32::<LittleEndian>(0).unwrap(); // Hash len.
    asm.write_u32::<LittleEndian>(0).unwrap(); // Flags.
    asm.write_i32::<LittleEndian>(1).unwrap(); // Major.
    asm.write_i32::<LittleEndian>(0).unwrap(); // Minor.
    asm.write_i32::<LittleEndian>(0).unwrap(); // Build.
    asm.write_i32::<LittleEndian>(0).unwrap(); // Revision.
    asm.write_all(&[0u8; 8]).unwrap(); // Public key token.
    metadata_writer.extend(asm.into_inner());

    // Pad to type definitions offset.
    while metadata_writer.len() < type_definitions_offset as usize {
        metadata_writer.push(0);
    }
    // Type 1: dummy <Module>.
    let mut type1 = Cursor::new(Vec::new());
    type1.write_u32::<LittleEndian>(0).unwrap(); // Name index.
    type1.write_u32::<LittleEndian>(0).unwrap(); // Namespace index.
    type1.write_i32::<LittleEndian>(0).unwrap(); // Byval type index.
    type1.write_i32::<LittleEndian>(-1).unwrap(); // Declaring type index.
    type1.write_i32::<LittleEndian>(-1).unwrap(); // Parent index.
    type1.write_i32::<LittleEndian>(-1).unwrap(); // Element type index.
    type1.write_i32::<LittleEndian>(-1).unwrap(); // Generic container index.
    type1.write_u32::<LittleEndian>(0).unwrap(); // Flags.
    // Starts & counts.
    for _ in 0..8 {
        type1.write_i32::<LittleEndian>(0).unwrap(); // Field start, method start etc.
    }
    for _ in 8..16 {
        type1.write_u16::<LittleEndian>(0).unwrap(); // Counts.
    }
    type1.write_u32::<LittleEndian>(0).unwrap(); // Bitfield.
    type1.write_u32::<LittleEndian>(1).unwrap(); // Token.
    metadata_writer.extend(type1.into_inner());

    // Type 2: TestClass.
    let mut type2 = Cursor::new(Vec::new());
    type2.write_u32::<LittleEndian>(test_class_offset).unwrap();
    type2
        .write_u32::<LittleEndian>(test_namespace_offset)
        .unwrap();
    type2.write_i32::<LittleEndian>(1).unwrap(); // Byval type index.
    type2.write_i32::<LittleEndian>(-1).unwrap(); // Declaring type index.
    type2.write_i32::<LittleEndian>(-1).unwrap(); // Parent index.
    type2.write_i32::<LittleEndian>(-1).unwrap(); // Element type index.
    type2.write_i32::<LittleEndian>(-1).unwrap(); // Generic container index.
    type2
        .write_u32::<LittleEndian>(0x00000001 | 0x00100000)
        .unwrap(); // Public, before-field-init flags.
    // Starts & counts.
    type2.write_i32::<LittleEndian>(0).unwrap(); // Field start.
    type2.write_i32::<LittleEndian>(1).unwrap(); // Method start (starts at method index 1).
    type2.write_i32::<LittleEndian>(0).unwrap(); // Event start.
    type2.write_i32::<LittleEndian>(0).unwrap(); // Property start.
    type2.write_i32::<LittleEndian>(0).unwrap(); // Nested types start.
    type2.write_i32::<LittleEndian>(0).unwrap(); // Interfaces start.
    type2.write_i32::<LittleEndian>(0).unwrap(); // Vtable start.
    type2.write_i32::<LittleEndian>(0).unwrap(); // Interface offsets start.
    type2.write_u16::<LittleEndian>(1).unwrap(); // Method count (1 method).
    type2.write_u16::<LittleEndian>(0).unwrap(); // Property count.
    type2.write_u16::<LittleEndian>(0).unwrap(); // Field count.
    type2.write_u16::<LittleEndian>(0).unwrap(); // Event count.
    type2.write_u16::<LittleEndian>(0).unwrap(); // Nested type count.
    type2.write_u16::<LittleEndian>(0).unwrap(); // Vtable count.
    type2.write_u16::<LittleEndian>(0).unwrap(); // Interfaces count.
    type2.write_u16::<LittleEndian>(0).unwrap(); // Interface offsets count.
    type2.write_u32::<LittleEndian>(0).unwrap(); // Bitfield.
    type2.write_u32::<LittleEndian>(2).unwrap(); // Token.
    metadata_writer.extend(type2.into_inner());

    // Pad to methods offset.
    while metadata_writer.len() < methods_offset as usize {
        metadata_writer.push(0);
    }
    // Method 1: dummy.
    let mut method1 = Cursor::new(Vec::new());
    method1.write_u32::<LittleEndian>(0).unwrap(); // Name index.
    method1.write_i32::<LittleEndian>(0).unwrap(); // Declaring type.
    method1.write_i32::<LittleEndian>(0).unwrap(); // Return type.
    method1.write_i32::<LittleEndian>(0).unwrap(); // Parameter start.
    method1.write_u32::<LittleEndian>(0).unwrap(); // Generic container.
    method1.write_u32::<LittleEndian>(0).unwrap(); // Token.
    method1.write_u16::<LittleEndian>(0).unwrap(); // Flags.
    method1.write_u16::<LittleEndian>(0).unwrap(); // Iflags.
    method1.write_u16::<LittleEndian>(0).unwrap(); // Slot.
    method1.write_u16::<LittleEndian>(0).unwrap(); // Parameter count.
    metadata_writer.extend(method1.into_inner());

    // Method 2: TestMethod.
    let mut method2 = Cursor::new(Vec::new());
    method2
        .write_u32::<LittleEndian>(test_method_offset)
        .unwrap();
    method2.write_i32::<LittleEndian>(1).unwrap(); // Declaring type = 1 (TestClass).
    method2.write_i32::<LittleEndian>(0).unwrap(); // Return type = void.
    method2.write_i32::<LittleEndian>(0).unwrap(); // Parameter start.
    method2.write_u32::<LittleEndian>(0).unwrap(); // Generic container.
    method2.write_u32::<LittleEndian>(0).unwrap(); // Token.
    method2.write_u16::<LittleEndian>(0x0006).unwrap(); // Public | virtual.
    method2.write_u16::<LittleEndian>(0).unwrap(); // Iflags.
    method2.write_u16::<LittleEndian>(0).unwrap(); // Slot.
    method2.write_u16::<LittleEndian>(0).unwrap(); // Parameter count = 0.
    metadata_writer.extend(method2.into_inner());

    // 3. Run dumper.
    let (dump_cs, _script_json) = run_dumper(&exec_bytes, &metadata_writer).unwrap();

    // 4. Assertions on generated C# dump.
    assert!(dump_cs.contains("Namespace: TestNamespace"));
    assert!(dump_cs.contains("public class TestClass"));
    assert!(dump_cs.contains("public void TestMethod()"));
}
