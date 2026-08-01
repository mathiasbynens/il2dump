#![allow(dead_code, unused_variables, unused_imports, unused_mut)]

pub mod binary;
pub mod binary_reader;
pub mod config;
pub mod decompiler;
pub mod il2cpp_binary_structures;
pub mod il2cpp_executor;
pub mod metadata;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn dump_il2cpp_wasm(exec_bytes: &[u8], metadata_bytes: &[u8]) -> Result<JsValue, JsValue> {
    let metadata = metadata::Metadata::load(metadata_bytes.to_vec())
        .map_err(|e| JsValue::from_str(&format!("Metadata load error: {}", e)))?;

    let binary_file = binary::BinaryFile::parse(exec_bytes.to_vec())
        .map_err(|e| JsValue::from_str(&format!("Binary parse error: {}", e)))?;

    let version = metadata.version;

    // Run heuristic search to find CodeRegistration and MetadataRegistration
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
        return Err(JsValue::from_str(
            "Failed to locate registration structures via heuristic search",
        ));
    }

    let executor =
        il2cpp_executor::Il2CppExecutor::new(metadata, binary_file, code_reg, metadata_reg)
            .map_err(|e| JsValue::from_str(&format!("Executor init error: {}", e)))?;

    let decompiler = decompiler::Decompiler::new(&executor);
    let config = config::Config::default();

    let (dump_cs, script_json) = decompiler
        .decompile_to_memory(&config)
        .map_err(|e| JsValue::from_str(&format!("Decompilation error: {}", e)))?;

    // Return as a JS array/tuple of [dump_cs, script_json]
    let array = js_sys::Array::new();
    array.push(&JsValue::from_str(&dump_cs));
    array.push(&JsValue::from_str(&script_json));

    Ok(JsValue::from(array))
}
